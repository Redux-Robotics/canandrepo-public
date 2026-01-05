// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>
#include <unordered_map>
#include <optional>
#include <units/time.h>
#include <units/angle.h>
#include "redux/canand/CanandSettings.h"

namespace redux::sensors::canandmag {

/**
 * The settings class for the Canandmag.
 * 
 * <p>
 * This class holds settings values that can be used to reconfigure Canandmag via Canandmag::SetSettings
 * Additionally, objects of this class can be filled using Canandmag.GetSettings which can be used to 
 * read the encoder's settings.
 * </p>
 * 
 * ```cpp
 * Canandmag enc = Canandmag{0};
 * 
 * // Only settings that are explicitly set here will be edited, so other settings 
 * // such as the status frame period will remain untouched.
 * CanandmagSettings stg;
 * stg.SetPositionFramePeriod(0_ms); // disables position readings
 * stg.SetVelocityFramePeriod(20_ms); // sets the rate of velocity measurements to every 20 ms
 * stg.SetInvertDirection(true); // inverts the encoder direction
 * 
 * enc.SetSettings(stg);
 *     
 * ```
 * Objects returned by the blocking Canandmag::GetSettings() method will always have all setting values populated and the getters will never return std::nullopt.
 * 
 * However, they may return std::nullopt if the object is either manually constructed and the corresponding getter hasn't been called yet
 * (e.g. calling GetStatusFramePeriod before SetStatusFramePeriod on an object you made)
 * or if the object comes from Canandmag::GetSettingsAsync() and not all settings have been received (use AllSettingsReceived() to check if all are present)
 * 
 * <a href="https://en.cppreference.com/w/cpp/utility/optional">cppreference on std::optional</a> may be helpful here.
 * 
 * Example blocking fetch:
 * ```cpp
 * Canandmag canandmag{0};
 * CanandmagSettings stg = canandmag.GetSettings();
 * if (stg.AllSettingsReceived()) { // check for timeout
 *   fmt::print("status frame period: {}\n", *stg.GetStatusFramePeriod()); // print the status frame period (usually 1000 ms)
 * }
 * ```
 * 
 */
class CanandmagSettings : public redux::canand::CanandSettings {
  public:
    CanandmagSettings() = default;
    const std::vector<uint8_t>& SettingAddresses() const override;

    /**
     * Sets the velocity filter width in milliseconds to sample over.
     * Velocity is computed by averaging all the points in the past widthMs milliseconds.
     * By factory default, the velocity filter averages over the past 25 milliseconds.
     * 
     * @param widthMs the new number of samples to average over. Minimum accepted is 0.25 milliseconds, maximum is 63.75 ms.
     */
    void SetVelocityFilterWidth(units::millisecond_t widthMs);

    /**
     * Sets the position frame period in seconds. 
     * By factory default, position frames are sent every 20 milliseconds (period=0.020_s)
     * If 0 is passed in, position frames will be disabled and the methods Canandmag::GetPosition() 
     * and Canandmag::GetAbsPosition() will not return new values.
     * 
     * @param period the new period for position frames in seconds in range [0_s, 65.535_s].
     */
    void SetPositionFramePeriod(units::second_t period);

    /**
     * Sets the velocity frame period in seconds. 
     * By factory default, velocity frames are sent every 20 milliseconds (period=0.020_s)
     * If 0 is passed in, velocity frames will be disabled and Canandmag::GetVelocity() will not return
     * new values.
     * 
     * @param period the new period for velocity frames in seconds in range [0_s, 65.535_s].
     */
    void SetVelocityFramePeriod(units::second_t period);

    /**
     * Sets the status frame period in seconds. 
     * By factory default, the encoder will broadcast 1 status message per second (period=1.000_s). 
     * 
     * @param period the new period for status frames in seconds in range [0.001_s, 16.383_s].
     */
    void SetStatusFramePeriod(units::second_t period);

    /**
     * Inverts the direction read from the sensor. By factory default, the sensor will read counterclockwise from its reading face
     * as positive (invert=false). 
     * @param invert whether to invert (negate) readings from the encoder
     */
    void SetInvertDirection(bool invert);

    /**
     * Sets whether or not the sensor should disallow zeroing and factory resets from the onboard button. 
     * By factory default, the sensor will allow the zero button to function when pressed (disable=false)
     * @param disable whether to disable the onboard zeroing button's functionality
     */
    void SetDisableZeroButton(bool disable);

    /**
     * Sets the zero offset of the encoder directly, rather than adjusting the zero offset 
     * relative to the currently read position.
     * 
     * <p>The zero offset is subtracted from the raw reading of the encoder's magnetic sensor to
     * get the adjusted absolute position as returned by Canandmag.GetAbsPosition().</p>
     * 
     * Users are encouraged to use Canandmag.SetAbsPosition instead.
     * 
     * @param offset the new offset in rotations [0..1)
     */
    void SetZeroOffset(units::turn_t offset);

    /**
     * Gets the velocity filter width in milliseconds [0.25..63.75], or std::nullopt if the value has not been set on this object.
     * @return the velocity filter width in milliseconds [0.25..63.75], or std::nullopt if the value has not been set on this object.
     */
    std::optional<units::millisecond_t> GetVelocityFilterWidth();

    /**
     * Gets the position frame period in seconds [0..65.535], or std::nullopt if the value has not been set on this object.
     * A value of 0 means position messages are disabled.
     * @return the position frame period in seconds [0..65.535], or std::nullopt if the value has not been set on this object.
     */
    std::optional<units::second_t> GetPositionFramePeriod();

    /**
     * Gets the velocity frame period in seconds [0..65.535], or std::nullopt if the value has not been set on this object.
     * A value of 0 means velocity messages are disabled.
     * @return the velocity frame period in seconds [0..65.535], or std::nullopt if the value has not been set on this object.
     */
    std::optional<units::second_t> GetVelocityFramePeriod();

    /**
     * Gets the status frame period in seconds [0.001..16.383], or std::nullopt if the value has not been set on this object.
     * A value of 0 means status messages are disabled.
     * @return the status frame period in seconds [0.001..16.383], or std::nullopt if the value has not been set on this object.
     */
    std::optional<units::second_t> GetStatusFramePeriod();

    /**
     * Gets whether or not the encoder has an inverted direction (0 for no, 1 for yes, std::nullopt for unset).
     * @return whether or not the encoder has an inverted direction (0 for no, 1 for yes, std::nullopt for unset).
     */
    std::optional<bool> GetInvertDirection();

    /**
     * Gets whether or not the sensor should disallow zeroing and factory resets from the onboard button (0 for allow, 1 for disallow, std::nullopt for unset).
     * @return whether or not the encoder has its onboard zero button's functionality disabled (0 for allow, 1 for disallow, std::nullopt for unset).
     */
    std::optional<bool> GetDisableZeroButton();

     /**
     * Gets the zero offset of the encoder.
     * 
     * The zero offset is subtracted from the raw reading of the encoder's magnetic sensor to
     * get the adjusted absolute position as returned by Canandmag.GetAbsPosition().
     * 
     * @return the zero offset [0..1), or std::nullopt if the value has not been set on this object.
     */
    std::optional<units::turn_t> GetZeroOffset();
};
}