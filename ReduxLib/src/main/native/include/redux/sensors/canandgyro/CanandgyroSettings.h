// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>
#include <unordered_map>
#include <optional>
#include <units/time.h>
#include <units/angle.h>
#include "redux/canand/CanandSettings.h"

namespace redux::sensors::canandgyro {

/**
 * The settings class for the Canandgyro.
 * 
 * <p>
 * This class holds settings values that can be used to reconfigure Canandgyro via Canandgyro::SetSettings
 * Additionally, objects of this class can be filled using Canandgyro.GetSettings which can be used to 
 * read the encoder's settings.
 * </p>
 * 
 * ```cpp
 * Canandgyro enc = Canandgyro{0};
 * 
 * // Only settings that are explicitly set here will be edited, so other settings 
 * // such as the status frame period will remain untouched.
 * CanandgyroSettings stg;
 * stg.SetAngularVelocityFramePeriod(0_ms); // disables angular velocity readings
 * stg.SetYawFramePeriod(5_ms); // sets the rate of yaw measurements to every 5 ms
 * 
 * enc.SetSettings(stg);
 *     
 * ```
 * Objects returned by the blocking Canandgyro::GetSettings() method will generally have all setting values populated and the getters will not return std::nullopt
 * assuming no timeout occured.
 * 
 * However, they may return std::nullopt if the object is either manually constructed and the corresponding getter hasn't been called yet
 * (e.g. calling GetStatusFramePeriod before SetStatusFramePeriod on an object you made)
 * or if the object comes from Canandgyro::GetSettingsAsync() and not all settings have been received (use AllSettingsReceived() to check if all are present)
 * 
 * <a href="https://en.cppreference.com/w/cpp/utility/optional">cppreference on std::optional</a> may be helpful here.
 * 
 * Example blocking fetch:
 * ```cpp
 * Canandgyro canandgyro{0};
 * CanandgyroSettings stg = canandgyro.GetSettings();
 * if (stg.AllSettingsReceived()) { // check for timeout
 *   fmt::print("status frame period: {}\n", *stg.GetStatusFramePeriod()); // print the status frame period (usually 1000 ms)
 * }
 * ```
 * 
 */
class CanandgyroSettings : public redux::canand::CanandSettings {
  public:
    CanandgyroSettings() = default;
    
    const std::vector<uint8_t>& SettingAddresses() const override;

    /**
    * Sets the dedicated yaw frame period in seconds.
    * 
    * By factory default, yaw frames are sent every 10 milliseconds (period = 0.010_s).
    * If 0 is passed in, yaw frames will be disabled and Canandgyro#getYaw} will not
    * return new values (unless configured to derive yaw from the angular position frame)
    * 
    * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
    */
    void SetYawFramePeriod(units::second_t period);

    /**
     * Sets the angular position frame period in seconds.
     * 
     * By factory default, angular position frames are sent every 20 milliseconds (period = 0.020_s).
     * If 0 is passed in, angular position frames will be disabled and methods returning angular
     * position data will not return new values.
     * 
     * <p>The one exception is Canandgyro.GetYaw which by default uses the yaw frame instead.</p>
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     */
    void SetAngularPositionFramePeriod(units::second_t period);

    /**
     * Sets the angular velocity frame period in seconds.
     * 
     * By factory default, angular velocity frames are sent every 100 milliseconds (period = 0.100_s).
     * If 0 is passed in, angular velocity frames will be disabled and methods returning angular 
     * velocity data will not return new values.
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     */
    void SetAngularVelocityFramePeriod(units::second_t period);

    /**
     * Sets the angular velocity frame period in seconds.
     * 
     * By factory default, acceleration frames are sent every 100 milliseconds (period = 0.100_s).
     * If 0 is passed in, acceleration frames will be disabled and methods returning acceleration
     * data will not return new values.
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     */
    void SetAccelerationFramePeriod(units::second_t period);

    /**
     * Sets the status frame period in seconds. 
     * 
     * By factory default, the device will broadcast 10 status messages every second (period=0.1_s). 
     * 
     * @param period the new period for status frames in seconds in range [0.001_s, 16.383_s].
     */
    void SetStatusFramePeriod(units::second_t period);

    /**
     * Gets the dedicated yaw frame period in seconds [0..65.535], or std::nullopt if the value has not 
     * been set on this object.
     * 
     * A value of 0 means yaw frames are disabled.
     * @return the frame period in seconds [0..65.535], or std::nullopt if the value has not been set on
     *     this object.
     */
    std::optional<units::second_t> GetYawFramePeriod();

    /**
     * Gets the angular position frame period in seconds [0..65.535], or std::nullopt if the value has 
     * not been set on this object.
     * 
     * A value of 0 means angular position frames are disabled.
     * @return the frame period in seconds [0..65.535], or std::nullopt if the value has not been set on
     *     this object.
     */
    std::optional<units::second_t> GetAngularPositionFramePeriod();

    /**
     * Gets the angular velocity frame period in seconds [0..65.535], or std::nullopt if the value has 
     * not been set on this object.
     * 
     * A value of 0 means angular velocity frames are disabled.
     * @return the frame period in seconds [0..65.535], or std::nullopt if the value has not been set on
     *     this object.
     */
    std::optional<units::second_t> GetAngularVelocityFramePeriod();

    /**
     * Gets the acceleration frame period in seconds [0..65.535], or std::nullopt if the value has not 
     * been set on this object.
     * 
     * A value of 0 means acceleration frames are disabled.
     * @return the frame period in seconds [0..65.535], or std::nullopt if the value has not been set on
     *     this object.
     */
    std::optional<units::second_t> GetAccelerationFramePeriod();

    /**
     * Gets the status frame period in seconds [0.001..16.383], or std::nullopt if the value has not beeno
     * set on this object.
     * @return the status frame period in seconds [0.001..16.383], or std::nullopt if the value has not been
     *     set on this object.
     */
    std::optional<units::second_t> GetStatusFramePeriod();

};
}