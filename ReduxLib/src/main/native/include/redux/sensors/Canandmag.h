// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>
#include <unordered_map>
#include <mutex>
#include <condition_variable>
#include <optional>
#include <units/angle.h>
#include <units/angular_velocity.h>
#include <units/time.h>
#include <units/temperature.h>
#include "redux/canand/CanandDevice.h"
#include "redux/canand/CanandEventLoop.h"
#include "redux/canand/CanandSettingsManager.h"
#include "redux/canand/CooldownWarning.h"
#include "redux/canand/MessageBus.h"
#include "redux/frames/Frame.h"

#include "redux/sensors/canandmag/CanandmagDetails.h"
#include "redux/sensors/canandmag/CanandmagFaults.h"
#include "redux/sensors/canandmag/CanandmagSettings.h"
#include "redux/sensors/canandmag/CanandmagStatus.h"

/**
 * Namespace for all classes relating to the Canandmag
*/
namespace redux::sensors::canandmag {

/**
 * Class for the CAN interface of the <a href="https://docs.reduxrobotics.com/canandmag/index.html">Canandmag.</a>
 * 
 * <p>
 * If you are using a Canandmag with Spark Max or Talon with the PWM output, see 
 * <a href="https://docs.reduxrobotics.com/canandmag/spark-max.html">our Spark Max docs</a>
 * or
 * <a href="https://docs.reduxrobotics.com/canandmag/talon-srx.html">our Talon SRX docs</a>
 * for information on how to use the encoder with the Rev and CTRE APIs.
 * </p>
 * 
 * <p>
 * The C++ vendordep uses the <a href="https://docs.wpilib.org/en/stable/docs/software/basic-programming/cpp-units.html">units library</a> 
 * for all dimensioned values, including settings.
 * </p>
 * 
 * <p>
 * Operations that receive data from the device (position, velocity, faults, temperature) generally do not block. 
 * The object receives data asynchronously from the CAN packet receive thread and reads thus return the last data received.
 * </p>
 * <p>
 * Operations that set settings or change offsets will generally wait for up to 20ms by default as they will usually
 * wait for a confirmation packet to be received in response -- unless the blocking timeout is set to zero, in which case
 * the operation swill not block.
 * </p>
 * 
 * Example code:
 * ```cpp
 * Canandmag canandmag{0}; // instantiates with encoder id 0 
 * 
 * // Reading the Canandmag
 * canandmag.GetPosition(); // returns a multi-turn relative position, in rotations (turns)
 * canandmag.GetAbsPosition(); // returns an absolute position bounded from [0..1) over one rotation
 * canandmag.GetVelocity(); // returns measured velocity in rotations per second
 * 
 * // Updating position
 * canandmag.SetPosition(-3.5_tr); // sets the relative position to -3.5 turns (does not persist on reboot)
 * canandmag.SetAbsPosition(330_deg, 0_s); // sets the absolute position to 330 degrees without blocking for confirmation (persists on reboot)
 * canandmag.ZeroAll(); // sets both the relative and absolute position to zero
 * 
 * // Changing configuration
 * CanandmagSettings settings;
 * settings.SetVelocityFilterWidth(25_ms); // sets the velocity filter averaging period to 25 ms
 * settings.SetInvertDirection(true); // make positive be clockwise instead of ccw opposite the sensor face
 * canandmag.SetSettings(settings, 20_ms); // apply the new settings to the device, with maximum 20 ms timeout per settings operation
 * 
 * // Faults
 * canandmag.ClearStickyFaults(); // clears all sticky faults (including the power cycle flag). This call does not block.
 * 
 * // this flag will always be true on boot until the sticky faults have been cleared, 
 * // so if this prints true the encoder has rebooted sometime between ClearStickyFaults and now.
 * CanandmagFaults faults = canandmag.GetStickyFaults(); // fetches faults
 * fmt::print("Encoder rebooted: {}\n", faults.powerCycle);
 * 
 * // Timestamped data
 * redux::frames::FrameData<units::turn_t> posFrameData = canandmag.GetPositionFrame().GetFrameData(); // gets current position + timestamp together
 * posFrameData.GetValue(); // fetched position in rotations
 * posFrameData.GetTimestamp(); // timestamp of the previous position
 * ```
 * 
 */
class Canandmag : public redux::canand::CanandDevice{
    public:
    /**
     * Constructor with the device's id. This object will be constant with respect to whatever CAN id assigned to it,
     * so if a device changes id it may change which device this object reads from.
     * @param canID the device id to use
     * @param bus the message bus to use. Defaults to "halcan"
    */
    Canandmag(int canID, std::string bus = "halcan");
    inline ~Canandmag() {redux::canand::RemoveCANListener(this);}
    // functions related to core functionality

    /**
     * Gets the current integrated position in rotations. 
     * 
     * <p> This value does not wrap around, so turning a sensed axle multiple rotations will return multiple sensed rotations of position. 
     * By default, positive is in the counter-clockwise direction from the sensor face.
     * </p>
     * <p> On encoder power-on, unlike the absolute value, this value will always initialize to zero. </p>
     * @return signed relative position in rotations (range [-131072.0..131071.999938396484])
     */
    units::turn_t GetPosition();

    /**
     * Gets the current absolute position of the encoder, in a scaled value from 0 inclusive to 1 exclusive.
     * By default, higher values are in the counter-clockwise direction from the sensor face.
     * <p> This value will persist across encoder power cycles making it appropriate for swerves/arms/etc. </p>
     * @return absolute position in fraction of a rotation [0..1)
     */
    units::turn_t GetAbsPosition();

    /**
     * Sets the new relative (multi-turn) position of the encoder to the given value.
     * 
     * <p>
     * Note that this does not update the absolute position, and this value is lost on a power cycle. To update the absolute position,
     * use Canandmag::SetAbsPosition
     * </p>
     * @param newPosition new position in rotations
     * @param timeout maximum time to wait for the operation to be confirmed (default 0.020 seconds). Set to 0 to not check (and not block).
     * @return true on success, false on timeout
     */
    bool SetPosition(units::turn_t newPosition, units::second_t timeout = 20_ms);

    /**
     * Sets the new absolute position value for the encoder which will (by default) persist across reboots.
     * 
     * @param newPosition new absolute position in fraction of a rotation (acceptable range [0..1))
     * @param timeout maximum time to wait for the operation to be confirmed (default 0.020 seconds). Set to 0 to not check (and not block).
     * @param ephemeral if true, set the setting ephemerally -- the new zero offset will not persist on power cycle.
     * @return true on success, false on timeout
     */
    bool SetAbsPosition(units::turn_t newPosition, units::second_t timeout = 20_ms, bool ephemeral = false);

    /**
     * Sets both the current absolute and relative encoder position to 0 -- generally equivalent to pressing the physical zeroing button on the encoder.
     * @param timeout maximum time in seconds to wait for each operation to be confirmed (there are 2 ops for zeroing both absolute and relative positions, 
     * so the wait is up to 2x timouet). Set to 0 to not check (and not block).
     * @return true on success, false on timeout
     */
    bool ZeroAll(units::second_t timeout = 20_ms);

    /**
     * Returns the measured velocity in rotations per second.
     * @return velocity, in rotations (turns) per second
     */
    units::turns_per_second_t GetVelocity();

    /**
     * Returns whether the encoder magnet is in range of the sensor or not.
     * This can be seen visually on the sensor -- a green LED is in range, whereas 
     * a red LED is out of range.
     * 
     * @return whether the output shaft magnet is in range.
     */
    bool MagnetInRange();

    // functions related to diagonstic data

    /**
     * Fetches sticky faults.
     * Sticky faults are the active faults, except once set they do not become unset until ClearStickyFaults() is called.
     * 
     * @return CanandmagFaults of the sticky faults
     */
    CanandmagFaults GetStickyFaults();

    /**
     * Fetches active faults.
     * Active faults are only active for as long as the error state exists.
     * 
     * @return CanandmagFaults of the active faults
     */
    CanandmagFaults GetActiveFaults();

    /**
     * Get onboard encoder temperature readings in degrees Celsius.
     * @return temperature in degrees Celsius
     */
    units::celsius_t GetTemperature();

    /**
     * Get the contents of the previous status packet, which includes active faults, sticky faults, and temperature.
     * @return device status as a status struct
     */
    inline CanandmagStatus GetStatus() { return status.GetValue(); }
    
    /**
     * Clears sticky faults.
     * 
     * <p>It is recommended to clear this during initialization, so one can check if the encoder loses power during operation later. </p>
     * <p>This call does not block, so it may take up to the next status frame (default every 1000 ms) for the sticky faults to be updated.</p>
     */
    void ClearStickyFaults();

    /**
     * Controls "party mode" -- an encoder identification tool that blinks the onboard LED
     * various colors at a user-specified strobe period.
     * The strobe period of the LED will be (50 milliseconds * level). Setting this to 0 disables party mode.
     * 
     * This function does not block.
     * 
     * @param level the party level value to set. 
     */
    void SetPartyMode(uint8_t level);

    // functions relating to settings
    
    /**
     * Fetches the Canandmag's current configuration in a blocking manner.
     * This function will need to block for at least 0.2-0.3 seconds waiting for the encoder to reply, so it is best
     * to put this in an init function, rather than the main loop.
     * 
     * <p> <b>Note that unlike v2023, this function will always return a settings object, 
     * but they may be incomplete settings!</b> </p>
     * 
     * You will need to do something like this to unwrap/verify the result:
     * 
     * ```cpp
     * // device declaration 
     * Canandmag canandmag{0};
     * 
     * // in your init/other sequence
     * CanandmagSettings stg = canandmag.GetSettings();
     * if (stg.AllSettingsReceived()) {
     *     // do your thing here
     * } else {
     *     // handle missing settings
     * }
     * ```
     * 
     * <p>Advanced users can use this function to retry settings missed from StartFetchSettings: </p>
     * ```cpp
     * // device declaration 
     * Canandmag canandmag{0};
     * enc.StartFetchSettings(); // send a "fetch settings command"
     * 
     * // wait some amount of time
     * CanandmagSettings stg = enc.GetSettingsAsync();
     * stg.AllSettingsReceived(); // may or may not be true
     * 
     * stg = enc.GetSettings(0_ms, 20_ms, 3); // Retry getitng the missing settings.
     * stg.AllSettingsReceived(); // far more likely to be true
     * ```
     * 
     * @param timeout maximum number of seconds to wait for a settings operation before timing out (default 350_ms)
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up
     * @param attempts number of attempts to try and fetch values missing from the first pass
     * @return Received set of CanandmagSettings of device configuration.
     */
    inline CanandmagSettings GetSettings(units::second_t timeout = 350_ms, units::second_t missingTimeout = 20_ms, uint32_t attempts = 3) { 
        return stg.GetSettings(timeout, missingTimeout, attempts); 
    }

    /**
     * Tells the Canandmag to begin transmitting its settings; once they are all transmitted (after ~200-300ms),
     * the values can be retrieved through the Canandmag::GetSettingsAsync() function call
     */
    inline void StartFetchSettings() { stg.StartFetchSettings(); }

    /**
     * Non-blockingly returns a {@link CanandmagSettings} object of the most recent known settings values received from the encoder.
     * 
     * <p> <b>Most users will probably want to use Canandmag::GetSettings() instead. </b> </p> 
     * 
     * One can call this after a Canandmag::StartFetchSettings() call, and use CanandmagSettings::AllSettingsReceived()
     * to check if/when all values have been seen. As an example:
     * 
     * ```cpp
     * 
     * // device declaration
     * Canandmag enc{0};
     * 
     * // somewhere in an init function
     * enc.StartFetchSettings();
     * 
     * // ...
     * // somewhere in a loop function
     * 
     * CanandmagSettings stg = enc.GetSettingsAsync();
     * if (stg.AllSettingsReceived()) {
     *   // do something with the returned settings
     *   fmt::print("Encoder velocity frame period: {}\n", *stg.GetVelocityFramePeriod());
     * }
     * ```
     * 
     * 
     * If this is called after Canandmag::SetSettings(), this method will return a settings object where only
     * the fields where the encoder has echoed the new values back will be populated. To illustrate this, consider the following:
     * ```cpp
     * // device declaration
     * Canandmag enc{0};
     * 
     * // somewhere in a loop 
     * CanandmagSettings stg_set;
     * stg_set.SetVelocityFramePeriod(100_ms);
     * enc.SetSettings(stg_set);
     * CanandmagSettings stg_get = enc.GetSettingsAsync();
     * 
     * // will likely return std::nullopt, as the device likely hasn't already responded to the settings set request
     * stg_get.GetVelocityFramePeriod();
     * 
     * // after up to 100 ms...
     * stg_get = enc.GetSettingsAsync();
     * 
     * // will likely be a value equivalent to 100_ms, may still be std::nullopt if the device is disconnected, so be careful of blind dereferences
     * stg_get.GetVelocityFramePeriod();
     * ```
     * 
     * @return CanandmagSettings of currently known settings
     */
    inline CanandmagSettings GetSettingsAsync() { return stg.GetKnownSettings(); }

    /**
     * Applies the settings from a CanandmagSettings object to the Canandmag. 
     * For more information, see the CanandmagSettings class documentation.
     * 
     * Example:
     * ```cpp
     * CanandmagSettings stg;
     * Canandmag enc{0};
     * // After configuring the settings object...
     * 
     * CanandmagSettings failed = enc.SetSettings(stg); 
     * if (failed.IsEmpty()) {
     *     // success
     * } else {
     *     // handle failed settings
     * }
     * ```
     * 
     * @param settings the CanandmagSettings to update the encoder with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. (default 0.020s, set to 0 to not check and not block).
     * @param attempts the maxinum number of attempts to write each individual settings
     * @return CanandmagSettings object of unsuccessfully set settings. 
     */
    inline CanandmagSettings SetSettings(CanandmagSettings& settings, units::second_t timeout = 20_ms, uint32_t attempts = 3) {
        return stg.SetSettings(settings, timeout, attempts);
    }

    /**
     * Resets the encoder to factory defaults, and then wait for all settings to be broadcasted 
     * back.
     * @param clearZero whether to clear the zero offset from the encoder's memory as well
     * @param timeout how long to wait for the new settings to be confirmed by the encoder in 
     *     seconds (suggested at least 0.35 seconds)
     * @return CanandmagSettings object of received settings. 
     *     Use CanandmagSettings.AllSettingsReceived() to verify success.
     */
    inline CanandmagSettings ResetFactoryDefaults(bool clearZero = false, units::second_t timeout = 350_ms) {
        uint8_t val = ((clearZero) ? details::SettingCommand::kResetFactoryDefault 
                                   : details::SettingCommand::kResetFactoryDefaultKeepZero);
        return stg.SendReceiveSettingCommand(val, timeout, true);
    }

    /**
     * Returns the CanandSettingsManager associated with this device.
     * 
     * The CanandSettingsManager is an internal helper object. 
     * Teams are typically not expected to use it except for advanced cases (e.g. custom settings
     * wrappers)
     * @return internal settings manager handle
     */
    inline redux::canand::CanandSettingsManager<CanandmagSettings>& GetInternalSettingsManager() {
        return stg;
    }

    /**
     * Returns the current relative position frame, which includes CAN timestamp data.
     * redux::canand::FrameData objects are immutable.
     * @return the current position frame, which will hold the current position in the same units as Canandmag::GetPosition()
     */
    inline redux::frames::Frame<units::turn_t>& GetPositionFrame() { return position; }

    /**
     * Returns the current absolute position frame, which includes CAN timestamp data.
     * @return the current position frame, which will hold the current position in the same units as Canandmag::getAbsPosition()
     */
    inline redux::frames::Frame<units::turn_t>& GetAbsPositionFrame() { return absPosition; }

    /**
     * Returns the current velocity frame, which includes CAN timestamp data.
     * @return the current velocity frame, which will hold the current velocity in the same units as Canandmag::getVelocity()
     */
    inline redux::frames::Frame<units::turns_per_second_t>& GetVelocityFrame() { return velocity; }

    /**
     * Returns a handle to the current status frame, which includes CAN timestamp data.
     * @return the current status frame, as a CanandmagStatus record.
     */
    inline redux::frames::Frame<CanandmagStatus>& GetStatusFrame() { return status; }


    // functions that directly modify settings
    void HandleMessage(redux::canand::CanandMessage& msg) override;
    redux::canand::CanandAddress& GetAddress() override;
    inline std::string GetDeviceClassName() override { return "Canandmag"; };
    inline redux::canand::CanandFirmwareVersion GetMinimumFirmwareVersion() override { 
        return redux::canand::CanandFirmwareVersion{2024, 2, 0};
    }

    /** number of encoder ticks per rotation */
    static constexpr double kCountsPerRotation = 16384;

    /** number of velocity ticks per rotation per second */
    static constexpr double kCountsPerRotationPerSecond = 1024;

    protected:

    /** internal Frame variable holding current relative position state */
    redux::frames::Frame<units::turn_t> position{0.0_tr, 0_ms};

    /** internal Frame variable holding current absolute position state */
    redux::frames::Frame<units::turn_t> absPosition{0.0_tr, 0_ms};

    /** internal Frame variable holding current velocity state */
    redux::frames::Frame<units::turns_per_second_t> velocity{0_tps, 0_ms};

    /** internal Frame variable holding current status value state */
    redux::frames::Frame<CanandmagStatus> status{CanandmagStatus{0, 0, false, 30_degC, false}, 0_ms};

    /** internal settings manager */
    redux::canand::CanandSettingsManager<CanandmagSettings> stg{*this};
    private:

    bool dataRecvOnce{false};
    units::second_t lastMessageTime{0_s};
    redux::canand::CooldownWarning setAbsPositionWarning{1_s, 5};
    redux::canand::CanandAddress addr;

};


}