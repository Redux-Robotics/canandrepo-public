// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include <vector>
#include <thread>
#include <mutex>
#include <functional>
#include <cstdlib>
#include <string.h>
#include <exception>
#include "redux/canand/CanandEventLoop.h"
#include "redux/canand/CanandMessage.h"
#include "ReduxCore.h"
#include "ReduxFIFO.h"
#include <units/time.h>
#include "frc/Notifier.h"
#include "frc/Errors.h"
#include "frc/Timer.h"
#include "hal/Threads.h"
#include "fmt/format.h"
#include <atomic>

/** Supported driver year */
constexpr int DRIVER_YEAR = 2024;

/** Supported driver major version */
constexpr int DRIVER_MAJOR_VERSION = 2;

/** Supported driver minor version */
constexpr int DRIVER_MINOR_VERSION = 0;

constexpr int DRIVER_NUMER = (DRIVER_YEAR << 16) | (DRIVER_MAJOR_VERSION << 8) | (DRIVER_MINOR_VERSION);

static std::thread run_thread;
static bool running = false;
static std::mutex thread_lock;
static bool enable_device_presence_warnings = true;

enum CheckState {
    kUnchecked,
    kDoNotCheck,
    kWaitingOnFirmwareVersion,
    kConnected,
    kDisconnected
};

struct DeviceEntry {
    redux::canand::CanandDevice* device;
    CheckState state{kUnchecked};
    bool enabled{true};
    units::second_t presence_threshold{2_s};
    uint8_t repeatTimeout = 20;
    DeviceEntry(redux::canand::CanandDevice* device) : device{device} {};
    virtual ~DeviceEntry() = default;
};

class CanandEventLoop {
public:
    std::vector<DeviceEntry> listeners;

    void run() {
        int32_t status = 0;
        HAL_SetCurrentThreadPriority(true, 30, &status);
        fmt::print("[ReduxLib] CanandEventLoop started.\n");
        struct ReduxFIFO_Message* msgbuf = ReduxCore_AllocateBuffer(32);
        size_t messages_read = 0;
        ReduxCore_OpenBusById(0);

        while (shouldRun.load()) {
            if (ReduxCore_BatchWaitForCANMessages(msgbuf, sizeof(msgbuf), &messages_read) == -1) break;

            for (size_t i = 0; i < messages_read; i++) {
                auto& rmsg = msgbuf[i];
                redux::canand::CanandMessage msg{rmsg.bus_id, rmsg.message_id, rmsg.timestamp, rmsg.data_size, rmsg.data};
                {
                    std::lock_guard<std::mutex> guard(thread_lock);
                    for (auto& dev : listeners) {
                        try {
                            redux::canand::CanandDevice& device = *dev.device;
                            if (device.GetAddress().MsgMatches(msg)) {
                                device.PreHandleMessage(msg);
                                device.HandleMessage(msg);
                            }
                        } catch (std::exception& exc) {
                            FRC_ReportError(frc::err::Error, "Exception in CanandEventLoop message listener:\n{}", exc.what());
                        }
                    }
                }
            }
        }
        ReduxCore_DeallocateBuffer(msgbuf, 32);
        fmt::print("[ReduxLib] CanandEventLoop exit.\n");
    }       

    void shutdown() {
        shouldRun.store(true);
    }
private:
    std::atomic<bool> shouldRun{true};
};

static CanandEventLoop event_loop{};

static void report_missing_device(redux::canand::CanandDevice& device) {
    FRC_ReportError(frc::warn::Warning, 
                        "{} possibly disconnected from bus -- check robot wiring and/or frame periods!",
                        device.GetDeviceName());
}
static void device_checker_task() {
    std::lock_guard<std::mutex> guard(thread_lock);
    if (frc::Timer::GetFPGATimestamp() < 2_s) { return; }
    for (auto& ent : event_loop.listeners) {
        redux::canand::CanandDevice& device = *ent.device;
        switch (ent.state) {
            case kUnchecked: {
                uint8_t data[] = {redux::canand::details::SettingCommand::kFetchSettingValue, 
                                  redux::canand::details::Setting::kFirmwareVersion};
                device.GetAddress().SendCANMessage(redux::canand::details::Message::kSettingCommand, data, 2);
                ent.state = CheckState::kWaitingOnFirmwareVersion;
                break;
            }
            case kWaitingOnFirmwareVersion: {
                device.CheckReceivedFirmwareVersion();
                ent.state = device.IsConnected() ? kConnected : kDisconnected;
                break;
            }
            case kConnected: {
                if (!device.IsConnected(ent.presence_threshold) && enable_device_presence_warnings) {
                    report_missing_device(device);
                    ent.state = kDisconnected;
                }
                break;
            }
            case kDisconnected: {
                if (device.IsConnected(ent.presence_threshold)) {
                    ent.state = CheckState::kConnected;
                    ent.repeatTimeout = 20;
                } else if(ent.repeatTimeout-- <= 0) {
                    report_missing_device(device);
                    ent.repeatTimeout = 20;
                }
                break;
            }
            case kDoNotCheck: {
                break;
            }
            default: break;
        }
    }
}

frc::Notifier& device_checker_notifier() {
    static frc::Notifier device_checker{1, device_checker_task};
    return device_checker;
}

static void CanandEventLoop_shutdownHook() {
    event_loop.shutdown();
    ReduxCore_StopServer();
    if(run_thread.joinable()) {
        // no messages could hang the shutdown
        run_thread.join();
    }
}

static void CanandEventLoop_ensureRunning() {
    // not thread safe. assumes you're holding thread_lock.
    if (!running) {
        int ver = ReduxCore_GetVersion();
        int yearVer = ((ver >> 16) & 0xffff);
        int majorVer = ((ver >> 8) & 0xff);
        int minorVer = (ver & 0xff);
        if (ver != DRIVER_NUMER) {
            FRC_ReportError(frc::err::Error, "Fatal Error: ReduxCore version v{}.{}.{} does not match vendordep version v{}.{}.{}",
                yearVer, majorVer, minorVer,
                DRIVER_YEAR, DRIVER_MAJOR_VERSION, DRIVER_MINOR_VERSION
            );
            std::exit(1);
        }

        ReduxCore_InitServer();
        running = true;
        run_thread = std::thread(std::bind(&::CanandEventLoop::run, &event_loop));
        std::atexit(CanandEventLoop_shutdownHook);
        device_checker_notifier().StartPeriodic(0.5_s);
    }
}

// NOT thread-safe. Use the external lock_guard.
static DeviceEntry* get_device_entry_if_exists(const redux::canand::CanandDevice& device) {
    for (auto& ent : event_loop.listeners) {
        if (ent.device == &device) {
            return &ent;
        }
    }
    return nullptr;
}

namespace redux::canand {
    void AddCANListener(CanandDevice* device) {
        std::lock_guard<std::mutex> guard(thread_lock);
        CanandEventLoop_ensureRunning();
        event_loop.listeners.push_back(DeviceEntry{device});
    }

    void RemoveCANListener(CanandDevice* device) {
        std::lock_guard<std::mutex> guard(thread_lock);
        bool found = false;
        std::vector<DeviceEntry>::size_type idx;
        for (std::vector<DeviceEntry>::size_type i = 0; i < event_loop.listeners.size(); i++) {
            if (event_loop.listeners[i].device == device) {
                idx = i;
                found = true;
                break;
            }
        }
        if (found) {
            event_loop.listeners.erase(event_loop.listeners.begin() + idx);
        }
    }

    void EnsureCANLinkServer() {
        std::lock_guard<std::mutex> guard(thread_lock);
        CanandEventLoop_ensureRunning();
    }

    void SetGlobalDevicePresenceWarnings(bool enabled) {
        std::lock_guard<std::mutex> guard(thread_lock);
        enable_device_presence_warnings = enabled;
    }

    void SetDevicePresenceWarnings(const CanandDevice& device, bool enabled) {
        std::lock_guard<std::mutex> guard(thread_lock);
        DeviceEntry* ent = get_device_entry_if_exists(device);
        if (ent == nullptr) return;
        ent->enabled = enabled;
    }

    void SetDevicePresenceThreshold(const CanandDevice& device, units::second_t threshold) {
        std::lock_guard<std::mutex> guard(thread_lock);
        DeviceEntry* ent = get_device_entry_if_exists(device);
        if (ent == nullptr) return;
        ent->presence_threshold = threshold;
    }
    
}