// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <optional>
#include <mutex>
#include <condition_variable>
#include <concepts>
#include <limits>
#include <utility>
#include <span>
#include <inttypes.h>
#include "redux/canand/CanandAddress.h"
#include "redux/canand/CanandFirmwareVersion.h"
#include "units/time.h"
#include <string>
#include <vector>

namespace redux::canand {

/**
 * The base class for all CAN-communicating Redux Robotics device classes.
 * The ReduxLib vendordep does all CAN message parsing in the Java/C++ classes themselves. CanandDevice provides
 * general facilities for sending and receiving CAN messages (abstracted from the actual underlying buses) as well as 
 * helper functions and constants common for all Redux CAN products.
 * 
 * Classes implementing CanandDevice need to do the following:
 * <ul>
 * <li> Implement CanandDevice::getAddress (usually by instantiating a CanandAddress in the constructor and returning it)</li>
 * <li> Implement CanandDevice::HandleMessage which will be called asynchronously whenever new CAN messages matching the object's CanandAddress get received by the robot </li>
 * <li> Run redux::canand::AddCANListener(this) in the constructor so HandleMessage actually gets called at runtime </li>
 * <li> Run redux::canand::RemoveCANListener(this) in the destructor. </li>
 * </ul>
*/
class CanandDevice {
  public:
    /** 
     * A callback called when a Redux CAN message is received and should be parsed.
     * Subclasses of CanandDevice should override this to update their internal state accordingly.
     * 
     * <p>
     * HandleMessage will be called on all Redux CAN packets received by the vendordep that match the CanandAddress 
     * returned by CanandDevice::GetAddress().
     * </p>
     * 
     * @param msg a reference to a CanandMessage representing the received message. The message may not have lifetime outside the function call.
     * 
    */
    virtual void HandleMessage(CanandMessage& msg) = 0;

    /**
     * Returns the reference to the CanandAddress representing the combination of CAN bus and 
     * CAN device ID that this CanandDevice refers to.
     * 
     * <p>
     * Implementing device subclasses should likely construct a new CanandAddress
     * in their constructor and return it here.
     * </p>
     * @return a reference to the CanandAddress for the device.
     */
    virtual CanandAddress& GetAddress() = 0;
 
    /**
     * Checks whether or not the device has sent a message within the last timeout seconds.
     * @param timeout window to check for message updates in seconds. Default 2
     * @return true if there has been a message within the last timeout seconds, false if not
     */
    bool IsConnected(units::second_t timeout = 2_s);

    /**
     * Returns a canonical class-wide device name.
     * @return std::string of a device type name
     */
    virtual inline std::string GetDeviceClassName() { return "CanandDevice"; }

    /**
     * Returns a nicely formatted name of the device, specific to its device address.
     * @return std::string of a specific device name
    */
    std::string GetDeviceName();

    /**
     * Called before HandleMessage gets called to run some common logic.
     * (Namely, handling setting receives and last message times)
     * 
     * This function can be overridden to change or disable its logic.
     * 
     * @param msg a CanandMessage representing the received message.
     */
    virtual void PreHandleMessage(CanandMessage& msg);

    /**
     * Checks the received firmware version.
     * 
     * <p>
     * If no firmware version has been received, complain to the driver station about potentially 
     * missing devices from the bus.
     * </p>
     * <p>
     * If the reported firmware version is too old, also complain to the driver station.
     * </p>
     */
    virtual void CheckReceivedFirmwareVersion();

    /**
     * Returns the minimum firmware version this vendordep requires.
     * @return minimum firmware version
     */
    inline virtual CanandFirmwareVersion GetMinimumFirmwareVersion() { return CanandFirmwareVersion{0, 0, 0}; }

    /**
     * Sends a CAN message to the CanandAddress.
     * 
     * @param apiIndex the API index the message should have (between 0-31 inclusive)
     * @param data 1-8 bytes of payload, as an array or pointer
     * @param length the length of the the payload buffer
     * @return if the operation was successful
     */
    inline bool SendCANMessage(uint8_t apiIndex, uint8_t* data, uint8_t length) {
        return GetAddress().SendCANMessage(apiIndex, data, length);
    }

    /**
     * Send a CAN message directly to the device, but properly length checked.
     * 
     * @param msgId the individual API index to value to send
     * @param data 1-8 byte payload std:span of std:byte
    */
    template<std::size_t len> requires(len < 8U) 
    void SendCANMessage(uint8_t msgId, std::span<std::byte,len> data) {
        SendCANMessage(msgId, std::as_bytes(data), data.size_bytes());
    }

  private:
    std::mutex settingRecvLock;
    std::optional<CanandFirmwareVersion> receivedFirmwareVersion{std::nullopt};

  /**
   * The last received message timestamp.
   * This is timed with respect to the FPGA timer and is updated before HandleMessage gets called
   * (it's updated in preHandleMessage).
  */
    std::optional<units::second_t> lastMessageTs{std::nullopt};
};


/**
 * Constants common to all CanandDevices
 */
namespace details {

/** Message IDs common to all devices */
class Message {
  public:
    /** Message id for setting control command */
    static constexpr uint8_t kSettingCommand     = 0x2;
    /** Message id for update setting on device */
    static constexpr uint8_t kSetSetting         = 0x3;
    /** Message id for setting value report from device */
    static constexpr uint8_t kReportSetting      = 0x4;
    /** Message id for clear device sticky faults */
    static constexpr uint8_t kClearStickyFaults  = 0x5;
    /** Message id for status frames */
    static constexpr uint8_t kStatus             = 0x6;
    /** Message id for party mode */
    static constexpr uint8_t kPartyMode          = 0x7;
};

/** Setting command IDs common to all devices */
class SettingCommand {
  public:
    /** Setting command id for Fetch all settings from device */
    static constexpr uint8_t kFetchSettings        = 0x0;
    /** Setting command id for Reset everything to factory default */
    static constexpr uint8_t kResetFactoryDefault = 0x1;
    /** setting command for Fetch individual setting */
    static constexpr uint8_t kFetchSettingValue = 0x2;
};

/** Setting indexes common to all devices */
class Setting {
  public:
    /** Setting index for Status frame period (ms) */
    static constexpr uint8_t kStatusFramePeriod   = 0x4;
    /** Setting index for Serial number */
    static constexpr uint8_t kSerialNumber        = 0x5;
    /** Setting index for Firmware version */
    static constexpr uint8_t kFirmwareVersion     = 0x6;
};

/** List of settings as a std::vector for CanandSettings. */
const std::vector<uint8_t> VDEP_SETTINGS = {
  Setting::kStatusFramePeriod,
  Setting::kSerialNumber,
  Setting::kFirmwareVersion
};

}
}