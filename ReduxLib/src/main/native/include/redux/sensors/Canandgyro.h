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
#include <units/force.h>
#include <units/time.h>
#include <units/temperature.h>
#include <frc/geometry/Quaternion.h>
#include <frc/geometry/Rotation3d.h>

#include "redux/canand/CanandDevice.h"
#include "redux/canand/CanandEventLoop.h"
#include "redux/canand/CanandSettingsManager.h"
#include "redux/canand/CooldownWarning.h"
#include "redux/frames/Frame.h"
#include "redux/sensors/canandgyro/CanandgyroDetails.h"
#include "redux/sensors/canandgyro/CanandgyroData.h"
#include "redux/sensors/canandgyro/CanandgyroFaults.h"
#include "redux/sensors/canandgyro/CanandgyroStatus.h"
#include "redux/sensors/canandgyro/CanandgyroSettings.h"

/**
 * Namespace for all classes relating to the Canandgyro
*/
namespace redux::sensors::canandgyro {

/**
 * Class for the CAN interface of the <a href="https://docs.reduxrobotics.com/canandgyro/index.html">canandgyro.</a>
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
 * Canandgyro canandgyro{0}; // instantiates with device id 0 
 * 
 * // Reading angular position
 * canandgyro.GetYaw(); // gets the yaw (Z-axis) value in units::turn_t [-180 deg inclusive..180 deg exclusive)
 *                      // This is probably what you want to use for robot heading.
 * canandgyro.GetMultiturnYaw(); // also gets yaw, except without a wraparound
 * canandgyro.GetPitch(); // pitch (Y-axis) value
 * canandgyro.GetRoll(); // roll (X-axis) value
 * canandgyro.GetRotation2d(); // Z-axis Rotation2d object
 * canandgyro.GetRotation3d(); // Full 3d rotation object
 * canandgyro.GetQuaternion(); // Raw rotation quaternion object
 * 
 * // Reading angular velocity (all in rotations per second)
 * canandgyro.GetAngularVelocityYaw();
 * canandgyro.GetAngularVelocityPitch();
 * canandgyro.GetAngularVelocityRoll();
 * 
 * // Linear acceleration (gravitational units)
 * canandgyro.GetAccelerationX();
 * canandgyro.GetAccelerationY();
 * canandgyro.GetAccelerationZ();
 * 
 * // Updating pose:
 * canandgyro.SetYaw(0.25_tr); // set yaw to 0.25 rotations positive
 * 
 * // set roll, pitch, yaw as 0.0, 0.1, and 0.25 rotations with 20 ms timeout
 * canandgyro.SetPoseRPY(0.0_tr, 0.1_tr, 0.25_tr, 20_ms);
 * // SetPose 
 * 
 * // Manually calibrating:
 * // The Canandgyro automatically calibrates on boot, but you may want to force a calibration.
 * // Calibration takes several seconds!!!
 * 
 * canandgyro.StartCalibration(); // begin calibration
 * canandgyro.IsCalibrating(); // check if the gyro is still calibrating
 * canandgyro.WaitForCalibrationToFinish(5_s); // wait up to 5 seconds for calibration to finish.
 * 
 * // Faults
 * canandgyro.ClearStickyFaults(); // clears all sticky faults (including the power cycle flag). 
 *                                  // This call does not block.
 * 
 * 
 * // this flag will always be true on boot until the sticky faults have been cleared, 
 * // so if this prints true the encoder has rebooted sometime between ClearStickyFaults and now.
 * CanandgyroFaults faults = canandgyro.GetStickyFaults(); // fetches faults
 * fmt::print("Device rebooted: {}\n", faults.powerCycle);
 * 
 * // Timestamped data
 * // gets current angular position + timestamp together
 * auto& quatFrameData = canandgyro.GetAngularPositionFrame();
 * quatFrameData.GetValue(); // fetched quaternion object
 * quatFrameData.GetValue().W(); // fetched quaternion W component
 * quatFrameData.GetTimestamp(); // timestamp of the quaternion data
 * ```
 * 
 */
class Canandgyro : public redux::canand::CanandDevice{
    protected:
    /** internal Frame variable used to track if the device is calibrating */
    redux::frames::Frame<bool> calibrating{false, 0_ms};

    /** internal Frame variable holding current yaw position state */
    redux::frames::Frame<units::turn_t> singleYaw{0.0_tr, 0_ms};

    /** internal Frame variable holding current yaw position state */
    redux::frames::Frame<units::turn_t> multiYaw{0.0_tr, 0_ms};

    /** internal Frame variable holding current angular position state */
    redux::frames::Frame<frc::Quaternion> quat{frc::Quaternion(), 0_ms};

    /** internal Frame variable holding current angular velocity state */
    redux::frames::Frame<AngularVelocity> vel{AngularVelocity{0_tps, 0_tps, 0_tps}, 0_ms};

    /** internal Frame variable holding current acceleration state */
    redux::frames::Frame<Acceleration> accel{Acceleration{0_SG, 0_SG, 0_SG}, 0_ms};

    /** internal Frame variable holding current status value state */
    redux::frames::Frame<CanandgyroStatus> status{CanandgyroStatus{0, 0, false, 30_degC}, 0_ms};

    /** internal settings manager */
    redux::canand::CanandSettingsManager<CanandgyroSettings> stg{*this};
    private:

    bool dataRecvOnce{false};
    bool useYawAngleFrame{true};
    units::second_t lastMessageTime{0_s};
    //redux::canand::CooldownWarning setAbsPositionWarning{1_s, 5};
    redux::canand::CanandAddress addr;

    public:
    /**
     * Constructor with the device's id. This object will be constant with respect to whatever CAN id assigned to it,
     * so if a device changes id it may change which device this object reads from.
     * @param canID the device id to use
     * @param bus the message bus to use. Defaults to "halcan".
    */
    Canandgyro(int canID, std::string bus = "halcan");
    inline ~Canandgyro() { redux::canand::RemoveCANListener(this); }

    /**
     * Gets a quaternion object of the gyro's 3d rotation from the zero point
     * @return a Quaternion of the current Canandgyro pose
     */
    inline frc::Quaternion GetQuaternion() { return quat.GetValue(); }

    /**
     * Gets an frc::Rotation3d object of the gyro's 3d rotation from the zero point
     * If you just want Z-axis rotation use GetYaw().
     * 
     * @return a Rotation3d of the current Canandgyro pose
     */
    inline frc::Rotation3d GetRotation3d() { return frc::Rotation3d{quat.GetValue()}; }

    /**
     * Gets an frc::Rotation2d object representing the rotation around the yaw axis from the zero point
     * If you just want Z-axis rotation use GetYaw().
     * @return a Rotation2d of the current Canandgyro yaw
     */
    inline frc::Rotation2d GetRotation2d() { return frc::Rotation2d(this->GetYaw()); }

    /**
     * Sets whether this object should use the dedicated yaw message for yaw angle instead of 
     * deriving it from the pose quaternion frame.
     * 
     * By default this is true, as the yaw angle frame is more precise and by default more frequent.
     * 
     * @param use use the yaw angle
     */
    inline void UseDedicatedYawAngleFrame(bool use) { this->useYawAngleFrame = use; }

    /**
     * Gets the yaw (Z-axis) rotation from [-0.5 rotations inclusive..0.5 exclusive).
     * 
     * This is probably the function you want to use for applications like field-centric control.
     * 
     * @return yaw in rotational units.
     */
    inline units::turn_t GetYaw() {
        if (this->useYawAngleFrame) {
            return this->singleYaw.GetValue();
        } else { 
            return this->GetRotation3d().Z(); 
        }
    }

    /**
     * Gets a multi-turn yaw (Z-axis) rotation that tracks to multiple continuous rotations.
     * 
     * Note that this relies on the dedicated multi-turn yaw packet so if it is disabled via
     * setYawFramePeriod it will not return fresh data.
     * 
     * @return multi-turn yaw in rotational units.
     */
    inline units::turn_t GetMultiturnYaw() {
        return this->multiYaw.GetValue();
    }

    /**
     * Gets the pitch (Y-axis) rotation from [-0.5 rotations inclusive..0.5 exclusive).
     * 
     * @return pitch in rotational units.
     */
    inline units::turn_t GetPitch() { return this->GetRotation3d().Y(); }

    /**
     * Gets the roll (X-axis) rotation from [-0.5 rotations inclusive..0.5 exclusive).
     * 
     * @return roll in rotational units.
    */
    inline units::turn_t GetRoll() { return this->GetRotation3d().X(); }


    /**
     * Gets the angular velocity along the roll (X) axis.
     * @return angular velocity
     */
    inline units::turns_per_second_t GetAngularVelocityRoll() { return vel.GetValue().Roll(); }

    /**
     * Gets the angular velocity along the pitch (Y) axis.
     * @return angular velocity
     */
    inline units::turns_per_second_t GetAngularVelocityPitch() { return vel.GetValue().Pitch(); }

    /**
     * Gets the angular velocity along the yaw (Z) axis.
     * @return angular velocity
     */
    inline units::turns_per_second_t GetAngularVelocityYaw() { return vel.GetValue().Yaw(); }

    /**
     * Gets the linear acceleration along the X axis.
     * @return linear acceleration
     */
    inline units::standard_gravity_t GetAccelerationX() { return accel.GetValue().X(); }

    /**
     * Gets the linear acceleration along the Y axis.
     * @return linear acceleration in Gs
     */
    inline units::standard_gravity_t GetAccelerationY() { return accel.GetValue().Y(); }

    /**
     * Gets the linear acceleration along the Z axis.
     * @return linear acceleration
     */
    inline units::standard_gravity_t GetAccelerationZ() { return accel.GetValue().Z(); }

    /**
     * Begins calibration on the Canandgyro.
     * 
     * This takes several seconds. To check the state of calibration, use IsCalibrating or
     * WaitForCalibrationToFinish.
     */
    void StartCalibration();

    /**
     * Returns if the Canandgyro is known to be currently calibrating.
     * @return if the Canandgyro is calibrating
     */
    inline bool IsCalibrating() {
        return calibrating.GetValue();
    }

    /**
     * Blocks the current thread until the Canandgyro has finished calibrating or until a timeout is reached.
     * 
     * @param timeout the timeout in seconds to wait for a calibration confirmation.
     * @return true if the calibration has finished within the timeout, false if not.
     */
    bool WaitForCalibrationToFinish(units::second_t timeout);

    /**
     * Sets a new angular position pose without recalibrating with a given roll/pitch/yaw.
     * If you just want to set yaw, use SetYaw.
     * 
     * @param newRoll new roll (x) pose
     * @param newPitch new pitch (y) pose
     * @param newYaw new yaw (z) pose
     * @param timeout the timeout in seconds to wait for a pose set confirmation.
     *                Set to 0 to not check (always return true.)
     * @param retries the number of retries for the set pose operation.
     * @return true if a pose set confirmation was received (or if timeout is zero)
     */
    inline bool SetPoseRPY(units::turn_t newRoll, units::turn_t newPitch, units::turn_t newYaw, units::second_t timeout = 20_ms, uint32_t retries = 5) {
        return SetPose(frc::Rotation3d(newRoll, newPitch, newYaw).GetQuaternion(), timeout, retries);
    }

    /**
     * Sets a new angular position without recalibrating with an frc::Rotation3d.
     * 
     * @param newPose new rotation3d pose
     * @param timeout the timeout in seconds to wait for a pose set confirmation.
     *                Set to 0 to not check (always return true.)
     * @param retries the number of retries for the set pose operation.
     * @return true if a pose set confirmation was received (or if timeout is zero)
     */
    inline bool SetPoseR3D(frc::Rotation3d newPose, units::second_t timeout = 20_ms, uint32_t retries = 5) {
        return SetPose(newPose.GetQuaternion(), timeout);
    }

    /**
     * Sets a new pose without recalibrating with an frc::Quaternion.
     * 
     * @param newPose new quaternion pose
     * @param timeout the timeout in seconds to wait for a pose set confirmation.
     *                Set to 0 to not check (always return true.)
     * @param retries the number of retries for the set pose operation.
     * @return true if a pose set confirmation was received (or if timeout is zero)
     */
    bool SetPose(frc::Quaternion newPose, units::second_t timeout = 20_ms, uint32_t retries = 5);

    /**
     * Sets a new yaw without recalibrating the Canandgyro.
     * Blocks for up to 50 milliseconds by default to confirm the transaction.
     * @param yaw new yaw angle in rotations
     * @param timeout the timeout in seconds to block to confirm the transaction (set 0 to not block)
     * @param retries the number of retries for the set pose operation.
     * @return true if a confirmation was received or the timeout is zero
     */
    bool SetYaw(units::turn_t yaw, units::second_t timeout = 20_ms, uint32_t retries = 5);


    // functions related to diagonstic data

    /**
     * Fetches sticky faults.
     * Sticky faults are the active faults, except once set they do not become unset until ClearStickyFaults() is called.
     * 
     * @return canandgyroFaults of the sticky faults
     */
    inline CanandgyroFaults GetStickyFaults() { return status.GetValue().stickyFaults; }

    /**
     * Fetches active faults.
     * Active faults are only active for as long as the error state exists.
     * 
     * @return canandgyroFaults of the active faults
     */
    CanandgyroFaults GetActiveFaults() { return status.GetValue().activeFaults; }

    /**
     * Get onboard encoder temperature readings in degrees Celsius.
     * @return temperature in degrees Celsius
     */
    units::celsius_t GetTemperature() { return status.GetValue().temperature; }

    /**
     * Get the contents of the previous status packet, which includes active faults, sticky faults, and temperature.
     * @return device status as a status struct
     */
    inline CanandgyroStatus GetStatus() { return status.GetValue(); }
    
    /**
     * Clears sticky faults.
     * 
     * <p>It is recommended to clear this during initialization, so one can check if the encoder loses power during operation later. </p>
     * <p>This call does not block, so it may take up to the next status frame (default every 1000 ms) for the sticky faults to be updated.</p>
     */
    void ClearStickyFaults();

    /**
     * Controls "party mode" -- an encoder identification tool that blinks the onboard LED
     * various colors if level != 0.
     * 
     * This function does not block.
     * 
     * @param level the party level value to set. 
     */
    void SetPartyMode(uint8_t level);

    // functions relating to settings
    
    /**
     * Fetches the device's current configuration in a blocking manner.
     * <p>This method works by requesting the device first send back all settings, and then waiting
     * for up to a specified timeout for all settings to be received by the robot controller. 
     * If the timeout is zero, this step is skipped. 
     * 
     * <p>If there are settings that were not received by the timeout, then this function will 
     * attempt to individually fetched each setting for up to a specified number of attempts. 
     * If the fresh argument is true and the timeout argument is 0, then only this latter step runs,
     * which can be used to only fetch settings that are missing from the known settings cache 
     * returned by GetSettingsAsync.
     * 
     * <p>The resulting set of known (received) settings is then returned, complete or not.
     * 
     * <p>This function blocks, so it is best to put this in init routines rather than a main loop.
     * 
     * ```cpp
     * // device declaration 
     * Canandgyro canandgyro{0};
     * 
     * // in your init/other sequence
     * CanandgyroSettings stg = canandgyro.GetSettings();
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
     * Canandgyro canandgyro{0};
     * enc.StartFetchSettings(); // send a "fetch settings command"
     * 
     * // wait some amount of time
     * CanandgyroSettings stg = enc.GetSettingsAsync();
     * stg.AllSettingsReceived(); // may or may not be true
     * 
     * stg = enc.GetSettings(0_ms, 20_ms, 3); // Retry getitng the missing settings.
     * stg.AllSettingsReceived(); // far more likely to be true
     * ```
     * 
     * @param timeout maximum number of seconds to wait for a settings operation before timing out (default 350_ms)
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up
     * @param attempts number of attempts to try and fetch values missing from the first pass
     * @return Received set of CanandgyroSettings of device configuration.
     */
    inline CanandgyroSettings GetSettings(units::second_t timeout = 350_ms, units::second_t missingTimeout = 20_ms, uint32_t attempts = 3) { 
        return stg.GetSettings(timeout, missingTimeout, attempts); 
    }

    /**
     * Tells the Canandgyro to begin transmitting its settings; once they are all transmitted (after ~200-300ms),
     * the values can be retrieved through the Canandgyro::GetSettingsAsync() function call
     */
    inline void StartFetchSettings() { stg.StartFetchSettings(); }

    /**
     * Non-blockingly returns a CanandgyroSettings object of the most recent known settings values received from the encoder.
     * 
     * <p> <b>Most users will probably want to use canandgyro::GetSettings() instead. </b> </p> 
     * 
     * One can call this after a Canandgyro::StartFetchSettings() call, and use CanandgyroSettings::AllSettingsReceived()
     * to check if/when all values have been seen. As an example:
     * 
     * ```cpp
     * 
     * // device declaration
     * canandgyro enc{0};
     * 
     * // somewhere in an init function
     * enc.StartFetchSettings();
     * 
     * // ...
     * // somewhere in a loop function
     * 
     * CanandgyroSettings stg = enc.GetSettingsAsync();
     * if (stg.AllSettingsReceived()) {
     *   // do something with the returned settings
     *   fmt::print("Device status frame period: {}\n", *stg.GetStatusFramePeriod());
     * }
     * ```
     * 
     * 
     * If this is called after Canandgyro::SetSettings(), this method will return a settings object where only
     * the fields where the device has echoed the new values back will be populated. To illustrate this, consider the following:
     * ```cpp
     * // device declaration
     * Canandgyro enc{0};
     * 
     * // somewhere in a loop 
     * CanandgyroSettings stg_set;
     * stg_set.SetStatusFramePeriod(100_ms);
     * enc.SetSettings(stg_set);
     * CanandgyroSettings stg_get = enc.GetSettingsAsync();
     * 
     * // will likely return std::nullopt, as the device likely hasn't already responded to the settings set request
     * stg_get.GetStatusFramePeriod();
     * 
     * // after up to 100 ms...
     * stg_get = enc.GetSettingsAsync();
     * 
     * // will likely be a value equivalent to 100_ms, may still be std::nullopt if the device is disconnected, so be careful of blind dereferences
     * stg_get.GetStatusFramePeriod();
     * ```
     * 
     * @return CanandgyroSettings of currently known settings
     */
    inline CanandgyroSettings GetSettingsAsync() { return stg.GetKnownSettings(); }

    /**
     * Applies the settings from a CanandgyroSettings object to the Canandgyro. 
     * For more information, see the CanandgyroSettings class documentation.
     * 
     * Example:
     * ```cpp
     * CanandgyroSettings stg;
     * Canandgyro enc{0};
     * // After configuring the settings object...
     * 
     * CanandgyroSettings failed = enc.SetSettings(stg); 
     * if (failed.IsEmpty()) {
     *     // success
     * } else {
     *     // handle failed settings
     * }
     * ```
     * 
     * @param settings the CanandgyroSettings to update the encoder with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. (default 0.020s, set to 0 to not check and not block).
     * @param attempts the maxinum number of attempts to write each individual settings
     * @return CanandgyroSettings object of unsuccessfully set settings. 
     */
    inline CanandgyroSettings SetSettings(CanandgyroSettings& settings, units::second_t timeout = 20_ms, uint32_t attempts = 3) {
        return stg.SetSettings(settings, timeout, attempts);
    }

    /**
     * Resets the encoder to factory defaults, and then wait for all settings to be broadcasted 
     * back.
     * @param timeout how long to wait for the new settings to be confirmed by the encoder in 
     *     seconds (suggested at least 0.35 seconds)
     * @return CanandgyroSettings object of received settings. 
     *     Use CanandgyroSettings.AllSettingsReceived() to verify success.
     */
    inline CanandgyroSettings ResetFactoryDefaults(units::second_t timeout = 350_ms) {
        return stg.SendReceiveSettingCommand(details::types::SettingCommand::kResetFactoryDefault, timeout, true);
    }

    /**
     * Returns the CanandSettingsManager associated with this device.
     * 
     * The CanandSettingsManager is an internal helper object. 
     * Teams are typically not expected to use it except for advanced cases (e.g. custom settings
     * wrappers)
     * @return internal settings manager handle
     */
    inline redux::canand::CanandSettingsManager<CanandgyroSettings>& GetInternalSettingsManager() {
        return stg;
    }

    /**
     * Returns the current single-turn yaw frame, which includes CAN timestamp data.
     * redux::canand::FrameData objects are immutable.
     * @return the current yaw frame
     */
    inline redux::frames::Frame<units::turn_t>& GetYawFrame() { return singleYaw; }

    /**
     * Returns the current multi-turn yaw frame, which includes CAN timestamp data.
     * redux::canand::FrameData objects are immutable.
     * @return the current yaw frame
     */
    inline redux::frames::Frame<units::turn_t>& GetMultiturnYawFrame() { return multiYaw; }

    /**
     * Returns the current angular position frame, which includes CAN timestamp data.
     * @return the current angular position frame
     */
    inline redux::frames::Frame<frc::Quaternion>& GetAngularPositionFrame() { return quat; }

    /**
     * Returns the current angular velocity frame, which includes CAN timestamp data.
     * @return the current angular velocity frame
     */
    inline redux::frames::Frame<AngularVelocity>& GetVelocityFrame() { return vel; }

    /**
     * Returns the current acceleration frame, which includes CAN timestamp data.
     * @return the current acceleration frame
     */
    inline redux::frames::Frame<Acceleration>& GetAccelerationFrame() { return accel; }

    /**
     * Returns a handle to the current status frame, which includes CAN timestamp data.
     * @return the current status frame, as a CanandgyroStatus record.
     */
    inline redux::frames::Frame<CanandgyroStatus>& GetStatusFrame() { return status; }


    // functions that directly modify settings
    void HandleMessage(redux::canand::CanandMessage& msg) override;
    redux::canand::CanandAddress& GetAddress() override;
    inline std::string GetDeviceClassName() override { return "Canandgyro"; };
    inline redux::canand::CanandFirmwareVersion GetMinimumFirmwareVersion() override { 
        return redux::canand::CanandFirmwareVersion{2024, 0, 0};
    }

};


}