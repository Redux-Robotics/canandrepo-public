// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include "CanandSettings.h"
#include "CanandDevice.h"
#include "CanandUtils.h"
#include <units/time.h>
#include <mutex>
#include <condition_variable>
#include <vector>
#include <concepts>
#include <frc/Errors.h>

namespace redux::canand {

/**
 * Setting result codes.
 * 
 * Positive indicates codes returnable from the "report setting" packet, while negative 
 * indicates codes returnable from other causes (e.g. timeouts)
 */
class SettingResultCode {
  public:
    enum : int16_t {
        /** General invalid data */
        kInvalid = -1,
        /** Operation timeout */
        kTimeout = -2,
        /** General error returned by device */
        kError = 0,
        /** Success */
        kOk = 1,
    };
};

/**
 * Result type of an individual settings operation outcome.
 */
class SettingResult {
  public:
    /**
     * Constructor.
     * @param value the value (only lower 48 bits matter)
     * @param result the result code
     */
    constexpr SettingResult(uint64_t value, int16_t result) : value{value}, result{result} {};

    /**
     * The setting value.
     */
    uint64_t value{0xffffffff'ffffffff};

    /**
     * The result code.
     */
    int16_t result{SettingResultCode::kInvalid};
    /**
     * Returns true if the setting result is valid.
     * 
     * @return true if the setting result is valid/ok
     */
    constexpr bool IsValid() { return result == SettingResultCode::kOk; }
};

/**
 * Settings flags that can be sent to the device in settings sets.
 */
class SettingFlags {
  public:
    /** Specifies that the setting is to be set ephemeral and will not persist in flash. */
    static constexpr uint8_t kEphemeral = 1;
};
/**
 * Common logic for settings management for CanandDevices.
 * 
 * This class holds a CanandSettings cache of known settings received from the CAN bus,
 * and offers a series of helper functions that provide common logic for bulk settings operations.
 */
template<class T>
requires std::is_base_of<CanandSettings, T>::value
class CanandSettingsManager {
  public:
    /**
     * Constructor.
     * @param dev the CanandDevice to associate with.
     */
    CanandSettingsManager(CanandDevice& dev) : dev{dev} {};

    /**
     * Fetches the device's current configuration in a blocking manner.
     * 
     * This function will block for at least `timeout` seconds waiting for the device to reply, so 
     * it is best to put this in a teleop or autonomous init function, rather than the main loop.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up
     * @param missingAttempts if >0, lists how many times to attempt fetching missing settings.
     * @return CanandSettings representing what has been received of the device's configuration.
     */
    inline T GetSettings(units::second_t timeout, units::second_t missingTimeout, uint32_t missingAttempts) {

        {
            std::unique_lock<std::mutex> lock(knownSettingsLock);
            if (timeout > 0_ms) {
                knownSettings.GetMap().clear();
                // send setting command
                SendSettingCommand(details::SettingCommand::kFetchSettings);
                // wait all settings
                knownSettingsCV.wait_for(lock, utils::toChronoSeconds(timeout),
                    [&]{return knownSettings.AllSettingsReceived();});
            }
            if (missingAttempts < 1 || missingTimeout <= 0_ms) { return T{knownSettings}; }
        }
        FetchMissingSettings(missingTimeout, missingAttempts);
        return T{knownSettings};
    }

    /**
     * Attempt to fill out the known settings with the set of settings it is missing.
     * 
     * @param timeout maximum timeout per setting index (seconds)
     * @param attempts number of attempts to fetch a setting index (should be at least 1)
     * @return a std::vector of setting indexes that were not able to be received despite attempts/timeout
     */
    inline std::vector<uint8_t> FetchMissingSettings(units::second_t timeout, uint32_t attempts) {
        std::vector<uint8_t> missingNow;
        std::vector<uint8_t> missingFinal;
        {
            // Synch snapshot of missing keys.
            std::unique_lock<std::mutex> lock(knownSettingsLock);
            if (knownSettings.AllSettingsReceived()) return missingFinal;
            for (uint8_t addr : knownSettings.SettingAddresses()) {
                if (!knownSettings.GetMap().contains(addr)) {
                    missingNow.push_back(addr);
                }
            }
        }

        for (uint8_t addr : missingNow) {
            SettingResult value{0, SettingResultCode::kInvalid};
            for (uint32_t i = 0; i < attempts && !value.IsValid(); i++) {
                value = FetchSetting(addr, timeout);
            }
            if (!value.IsValid()) {
                missingFinal.push_back(addr);
            }
        }
        return missingFinal;
    }

    /**
     * Tells the device to begin transmitting its settings. 
     * Once they are all transmitted (typically after ~200-300ms), 
     * the values can be retrieved from GetKnownSettings()
     */
    inline void StartFetchSettings() {
        std::unique_lock<std::mutex> lock(knownSettingsLock);
        SendSettingCommand(details::SettingCommand::kFetchSettings);
        knownSettings.GetMap().clear();
    }

    /**
     * Applies the settings from a CanandSettings to the device, with fine
     * grained control over failure-handling.
     * 
     * This overload allows specifiyng the number of retries per setting as well as the confirmation
     * timeout. Additionally, it returns a CanandSettings object of settings that 
     * were not able to be successfully applied.
     * 
     * @param settings the CanandSettings to update the device with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @param attempts the maximum number of attempts to write each individual setting
     * @return a CanandSettings object of unsuccessfully set settings.
     */
    inline T SetSettings(T& settings, units::second_t timeout, uint32_t attempts) {
        T missed_settings;
        std::unordered_map<uint8_t, uint64_t> values = settings.FilteredMap();
        int flags = 0;
        if (settings.IsEphemeral()) {
            flags |= SettingFlags::kEphemeral;
        }
        for (auto& it: values) {
            uint8_t addr = it.first;
            {
                std::lock_guard<std::mutex> guard(knownSettingsLock);
                knownSettings.GetMap().erase(addr);
            }
            bool success = false;
            for (uint32_t i = 0; i < attempts && !success; i++) {
                success = ConfirmSetSetting(addr, (uint8_t*) &it.second, 6, timeout, flags).IsValid();
            }
            if (!success) {
                // Add the missed setting to the missed settings map
                missed_settings.GetMap()[addr] = values[addr];
            }
        }

        return missed_settings;
    }

    /**
     * Applies the settings from a CanandSettings to the device. 
     * 
     * @param settings the CanandSettings to update the device with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @return true if successful, false if a setting operation failed
     */
    inline bool SetSettings(T& settings, units::second_t timeout) {
        T missed = SetSettings(settings, timeout, 3);
        if (!missed.IsEmpty()) {
            FRC_ReportError(frc::err::Error, "{} settings could not be applied to {}", 
                missed.GetMap().size(), dev.GetDeviceName());
            return false;
        }
        return true;
    }

    /**
     * Runs a setting command that may mutate all settings and trigger a response.
     * 
     * Typically used with "reset factory default" type commands
     * @param cmd setting index
     * @param timeout total timeout for all settings to be returned
     * @param clearKnown whether to clear the set of known settings
     * @return the set of known settings.
     */
    inline T SendReceiveSettingCommand(uint8_t cmd, units::second_t timeout, bool clearKnown) {
        std::unique_lock<std::mutex> guard(knownSettingsLock);
        if (clearKnown) knownSettings.GetMap().clear();
        SendSettingCommand(cmd);
        if (timeout > 0_ms) {
            knownSettingsCV.wait_for(guard, utils::toChronoSeconds(timeout),
                [&]{return knownSettings.AllSettingsReceived();});
        }
        return T{knownSettings};
    }


    /**
     * Return a CanandSettings of known settings.
     * The object returned is a copy of this object's internal copy.
     * @return known settings
     */
    inline T GetKnownSettings() {
        // construct a blank object and switch out backing map for a clone of knownSettings
        return T{knownSettings};
    }

    /**
     * Setting handler to put in CanandDevice::HandleMessage
     * 
     * @param msg the CanandMessage containing settings data.
     */
    inline void HandleSetting(CanandMessage& msg) {
        uint8_t flags = 0;
        uint8_t* data = msg.GetData();
        uint32_t dataLength = msg.GetLength();
        uint64_t settingValue = 0;
        if (dataLength < 7) return;
        else if (dataLength >= 8) {
            flags = data[7];
        }
        memcpy(&settingValue, data + 1, 6);
        // process knownSettings
        bool allSettingsFound = false;
        {
            std::lock_guard<std::mutex> getSettingsGuard(knownSettingsLock);
            knownSettings.GetMap()[data[0]] = settingValue;
            allSettingsFound = knownSettings.AllSettingsReceived();
        }
        if (allSettingsFound) knownSettingsCV.notify_all();

        // process settings recv
        {
            std::lock_guard<std::mutex> settingRecvGuard(settingRecvLock);
            settingRecvCtr++;
            settingRecvIdx = data[0];
            settingRecvCode = flags;
            settingRecvValue = 0;
            memcpy(&settingRecvValue, data + 1, 6);
        }
        settingRecvCV.notify_all();     
    }

    /**
     * Directly sends a CAN message to the associated CanandDevice to set a setting by index.
     * This function does not block nor check if a report settings message is sent in response.
     * 
     * <p>
     * Device subclasses will usually have a more user-friendly settings interface, 
     * eliminating the need to call this function directly in the vast majority of cases.
     * </p>
     * 
     * @param settingId the setting id
     * @param value the raw numerical value. Only the first 6 bytes will be used.
     * @param length the length of the buffer specified.
     * @param flags optional flags to send to the device specifying how the setting will be set.
    */
    inline void SetSettingById(uint8_t settingId, uint8_t* value, uint8_t length, uint8_t flags) {
        uint8_t data[8] = { 0 };
        data[0] = settingId;
        data[7] = flags;
        if (length > 6) length = 6;
        memcpy(data + 1, value, length);
        dev.SendCANMessage(details::Message::kSetSetting, data, 8);
    }

    /**
     * 
     * Directly sends a CAN message to the associated CanandDevice to set a setting by index.
     * This function does not block nor check if a report settings message is sent in response.
     * 
     * <p>
     * Device subclasses will usually have a more user-friendly settings interface, 
     * eliminating the need to call this function directly in the vast majority of cases.
     * </p>
     * 
     * @param settingId setting id to use
     * @param value 48-bit long
     * @param flags flags
     */
    inline void SetSettingById(uint8_t settingId, uint64_t value, uint8_t flags) {
        // something soemthing undefined behavior
        SetSettingById(settingId, (uint8_t*) &value, 6, flags);
    }

    /**
     * Potentially blocking operation to send a setting and wait for a report setting message to be 
     * received to confirm the operation.
     * 
     * @param settingIdx Setting index to set and listen for
     * @param payload the bytes to send.
     * @param length the length of the payload.
     * @param timeout the timeout to wait before giving up in seconds. Passing in 0 will return 
     *     instantly (not block)
     * @param flags optional flags to send to the device specifying how the setting will be set.
     * @return the value received by the report setting packet if existent or kTimeout otherwise. 
     *     If timeout = 0, return "payload" (assume success)
     */
    inline SettingResult ConfirmSetSetting(uint8_t settingIdx, uint8_t* payload, uint8_t length, 
        units::second_t timeout, uint8_t flags) {

        std::unique_lock<std::mutex> lock(settingRecvLock);
        SetSettingById(settingIdx, payload, length, flags);
        if (timeout <= 0_ms) {
            uint64_t longPayload = 0;
            memcpy(&longPayload, payload, std::min((uint8_t) 6, length));
            return SettingResult{longPayload, SettingResultCode::kOk}; 
        }
        uint32_t prevCtr = settingRecvCtr;
        if (!settingRecvCV.wait_for(lock, utils::toChronoSeconds(timeout), [&]{
            // checks that the recv is both the correct idx and fresh
            return this->settingRecvIdx == settingIdx && this->settingRecvCtr != prevCtr;
        })) {
            // timeout
            return SettingResult{0, SettingResultCode::kTimeout};
        }

        return SettingResult{settingRecvValue, settingRecvCode};
    }

    /**
     * Potentially blocking operation to send a setting and wait for a report setting message to be 
     * received to confirm the operation.
     * 
     * @param settingIdx Setting index to set and listen for
     * @param payload the 48 bits to send.
     * @param timeout the timeout to wait before giving up in seconds. Passing in 0 will return 
     *     instantly (not block)
     * @param flags optional flags to send to the device specifying how the setting will be set.
     * @return the value received by the report setting packet if existent or kTimeout otherwise. 
     *     If timeout = 0, return "payload" (assume success)
     */
    inline SettingResult ConfirmSetSetting(uint8_t settingIdx, uint64_t payload,
        units::second_t timeout, uint8_t flags) {

        std::unique_lock<std::mutex> lock(settingRecvLock);
        SetSettingById(settingIdx, payload, flags);
        if (timeout <= 0_ms) { 
            return SettingResult{payload, SettingResultCode::kOk}; 
        }
        uint32_t prevCtr = settingRecvCtr;
        if (!settingRecvCV.wait_for(lock, utils::toChronoSeconds(timeout), [&]{
            // checks that the recv is both the correct idx and fresh
            return this->settingRecvIdx == settingIdx && this->settingRecvCtr != prevCtr;
        })) {
            // timeout
            return SettingResult{0, SettingResultCode::kTimeout};
        }

        return SettingResult{settingRecvValue, settingRecvCode};
    }

    /**
     * Fetches a setting from the device and returns the received result.
     * @param settingIdx Setting index to fetch
     * @param timeout timeout to wait before giving up in seconds. Passing in 0 will return a timeout.
     * @return SettingResult representing the setting result.
     */
    inline SettingResult FetchSetting(uint8_t settingIdx, units::second_t timeout) {
        std::unique_lock<std::mutex> lock(settingRecvLock);
        uint8_t buf[] = {details::SettingCommand::kFetchSettingValue, settingIdx};
        dev.SendCANMessage(details::Message::kSettingCommand, buf, 2);

        if (timeout <= 0_ms) { return SettingResult{0, SettingResultCode::kInvalid}; }
        uint32_t prevCtr = settingRecvCtr;
        if (!settingRecvCV.wait_for(lock, utils::toChronoSeconds(timeout), [&]{
            // checks that the recv is both the correct idx and fresh
            return this->settingRecvIdx == settingIdx && this->settingRecvCtr != prevCtr;
        })) {
            // timeout
            return SettingResult{0, SettingResultCode::kTimeout};
        }

        return SettingResult{settingRecvValue, settingRecvCode};
    }

    /**
     * Sends a setting command with no arguments.
     * @param settingCmdIdx the index of the setting command to send.
     */
    inline void SendSettingCommand(uint8_t settingCmdIdx) {
        dev.SendCANMessage(details::Message::kSettingCommand, &settingCmdIdx, 1);
    }

  private:
    T knownSettings;
    std::mutex knownSettingsLock;
    std::condition_variable knownSettingsCV;

    std::mutex settingRecvLock;
    std::condition_variable settingRecvCV;
    uint32_t settingRecvCtr = 0;
    uint8_t settingRecvIdx = 0;
    uint8_t settingRecvCode = 0;
    uint64_t settingRecvValue = 0;

    CanandDevice& dev;
};


}