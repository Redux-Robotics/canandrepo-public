// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag;

import java.util.HashMap;
import java.util.Optional;

import com.reduxrobotics.canand.CanandSettings;

/**
 * The settings class for the {@link Canandmag}.
 * 
 * <p>
 * This class holds settings values that can be used to reconfigure Canandmag via 
 * {@link Canandmag#setSettings}. 
 * Additionally, objects of this class are returned from {@link Canandmag#getSettings} and
 * {@link Canandmag#getSettingsAsync} which can be used to read the devices's settings.
 * </p>
 * 
 * <pre>
 * // Object initialization
 * Canandmag enc = new Canandmag(0);
 * // ...
 * 
 * // Only settings that are explicitly set here will be edited, so other settings 
 * // such as the status frame period will remain untouched.
 * // For example, enc.setSettings(new CanandmagSettings()); will be a no-op.
 * 
 * enc.setSettings(new CanandmagSettings()
 *     .setPositionFramePeriod(0) // disables position readings
 *     .setVelocityFramePeriod(0.10) // sets the rate of velocity measurements to every 20 ms
 *     .setInvertDirection(true) // inverts the encoder direction
 * );
 * // enc.setSettings will block by default for up to 350 ms to confirm all values were set.
 * 
 * </pre>
 * 
 * <b> Objects returned by the blocking {@link Canandmag#getSettings} method, unlike v2023, 
 * may now return null on its getters.</b> This occurs when getSettings would've previously 
 * returned null -- however, this allows for partial settings fetches. 
 * To check if the settings fetch succeeded, one can use {@link #allSettingsReceived()} to check
 * all fields are populated.
 * 
 * 
 * Example blocking fetch:
 * <pre>
 * // Object initialization
 * Canandmag canandmag = new Canandmag(0);
 * // ...
 * 
 * // the actual fetch itself
 * CanandmagSettings stg = canandmag.getSettings(0.5); // wait up to 500 ms
 * if (stg.allSettingsReceived()) {
 *    // print the status frame period (1000 ms/1 second by default)
 *    System.out.printf("status frame period: %d\n", stg.getStatusFramePeriod()); 
 * }
 * </pre>
 */
public class CanandmagSettings extends CanandSettings {

    private static final int settingsAddresses[] = {
        CanandmagDetails.kStg_StatusFramePeriod,
        CanandmagDetails.kStg_ZeroOffset,
        CanandmagDetails.kStg_VelocityWindow,
        CanandmagDetails.kStg_VelocityFramePeriod,
        CanandmagDetails.kStg_PositionFramePeriod,
        CanandmagDetails.kStg_InvertDirection,
        CanandmagDetails.kStg_DisableZeroButton,
    };

    @Override
    protected int[] fetchSettingsAddresses() {
        return settingsAddresses;
    }

    /**
     * Instantiates a new {@link CanandmagSettings} object that is "completely blank" -- 
     * holding no settings values at all.
     * 
     * Settings are only populated into the {@link CanandmagSettings} object explicitly 
     * through the various setter methods -- running 
     * {@code canandmag.setSetting(new CanandmagSettings())} would not update the device 
     * at all.
     * 
     * To reset a device back to factory defaults, use {@link Canandmag#resetFactoryDefaults}
     */
    public CanandmagSettings() {
        values = new HashMap<>();
    }

    /**
     * Instantiates a new {@link CanandmagSettings} object that copies its settings from the 
     * input instance.
     * 
     * @param toCopy the input settings object to copy
     */
    public CanandmagSettings(CanandmagSettings toCopy) {
        values = toCopy.getFilteredMap();
    }

    /**
     * Sets the velocity filter width in milliseconds to sample over.
     * Velocity is computed by averaging all the points in the past {@code widthMs} milliseconds.
     * By factory default, the velocity filter averages over the past 25 milliseconds.
     * 
     * @param widthMs the new number of samples to average over. Minimum accepted is 0.25 
     *     milliseconds, maximum is 63.75 ms.
     * @return the calling object, so these calls can be chained
     */
    public CanandmagSettings setVelocityFilterWidth(double widthMs) {
        values.put(CanandmagDetails.kStg_VelocityWindow, 
            checkBounds("velocity filter width widthMs", widthMs, 1, 255, 4));
        return this;
    }

    /**
     * Sets the position frame period in seconds. 
     * By factory default, position frames are sent every 20 milliseconds (period=0.10)
     * If 0 is passed in, position frames will be disabled and the methods 
     * {@link Canandmag#getPosition()} and {@link Canandmag#getAbsPosition()} will not 
     * return new values.
     * 
     * @param period the new period for position frames in seconds in range [0_ms, 65.535_s]
     * @return the calling object, so these calls can be chained
     */
    public CanandmagSettings setPositionFramePeriod(double period) {
        values.put(CanandmagDetails.kStg_PositionFramePeriod, 
            checkBounds("position frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the velocity frame period in seconds. 
     * By factory default, velocity frames are sent every 20 milliseconds (period=0.10)
     * If 0 is passed in, velocity frames will be disabled and {@link Canandmag#getVelocity()}
     * will not return new values.
     * 
     * @param period the new period for velocity frames in seconds in range [0_ms, 65.535_s]
     * @return the calling object, so these calls can be chained
     */
    public CanandmagSettings setVelocityFramePeriod(double period) {
        values.put(CanandmagDetails.kStg_VelocityFramePeriod, 
            checkBounds("velocity frame period", period, 0, 65535, 1000));
        return this;
    }


    /**
     * Sets the status frame period in seconds. 
     * By factory default, the encoder will broadcast 1 status message per second (period=1.0). 
     * 
     * @param period the new period for status frames in seconds in range [1.0_s, 16.383_s].
     * @return the calling object, so these calls can be chained
     */
    public CanandmagSettings setStatusFramePeriod(double period) {
        values.put(CanandmagDetails.kStg_StatusFramePeriod, 
            checkBounds("status frame period", period, 1, 16383, 1000));
        return this;
    }

    /**
     * Inverts the direction read from the sensor. By factory default, the sensor will read 
     * counterclockwise from its reading face as positive (invert=false). 
     * 
     * @param invert whether to invert (negate) readings from the encoder
     * @return the calling object
     */
    public CanandmagSettings setInvertDirection(boolean invert) {
        values.put(CanandmagDetails.kStg_InvertDirection, invert ? 1L : 0);
        return this;
    }

    /**
     * Sets whether or not the sensor should disallow zeroing and factory resets from the 
     * onboard button. 
     * 
     * By factory default, the sensor will allow the zero button to function when pressed 
     * (disable=false)
     * @param disable whether to disable the onboard zeroing button's functionality
     * @return the calling object
     */
    public CanandmagSettings setDisableZeroButton(boolean disable) {
        values.put(CanandmagDetails.kStg_DisableZeroButton, disable ? 1L : 0);
        return this;
    }

    /**
     * Sets the zero offset of the encoder directly, rather than adjusting the zero offset 
     * relative to the currently read position.
     * 
     * <p>The zero offset is subtracted from the raw reading of the encoder's magnetic sensor to
     * get the adjusted absolute position as returned by {@link Canandmag#getAbsPosition()}.
     * 
     * Users are encouraged to use {@link Canandmag#setAbsPosition} instead.
     * 
     * @param offset the new offset in rotations [0..1)
     * @return the calling object
     */
    public CanandmagSettings setZeroOffset(double offset) {
        values.put(CanandmagDetails.kStg_ZeroOffset, 
            checkBounds("zero offset", offset, 0, 16383, Canandmag.kCountsPerRotation));
        return this;
    }

    // Getters

    /**
     * Gets the velocity filter width in milliseconds [0.25..63.75], or null if the value has 
     * not been set on this object.
     * @return the velocity filter width in milliseconds [0.25..63.75], or null if the value has
     *     not been set on this object.
     */
    public Optional<Double> getVelocityFilterWidth() {
        return getIntAsDouble(CanandmagDetails.kStg_VelocityWindow, 4);
    }

    /**
     * Gets the position frame period in seconds [0..65.535], or null if the value has not been 
     * set on this object.
     * A value of 0 means position messages are disabled.
     * @return the position frame period in seconds [0..65.535], or null if the value has not 
     *     been set on this object.
     */
    public Optional<Double> getPositionFramePeriod() {
        return getIntAsDouble(CanandmagDetails.kStg_PositionFramePeriod, 1000);
    }

    /**
     * Gets the velocity frame period in seconds [0..65.535], or null if the value has not been 
     * 
     * set on this object.
     * A value of 0 means velocity messages are disabled.
     * @return the velocity frame period in seconds [0..65.535], or null if the value has not 
     *     been set on this object.
     */
    public Optional<Double> getVelocityFramePeriod() {
        return getIntAsDouble(CanandmagDetails.kStg_VelocityFramePeriod, 1000);
    }

    /**
     * Gets the status frame period in seconds [0.001..65.535], or null if the value has not beeno
     * set on this object.
     * A value of 0 means status messages are disabled.
     * @return the status frame period in seconds [0.001..65.535], or null if the value has not been
     *     set on this object.
     */
    public Optional<Double> getStatusFramePeriod() {
        return getIntAsDouble(CanandmagDetails.kStg_StatusFramePeriod, 1000);
    }

    /**
     * Gets whether or not the encoder has an inverted direction (false for no, true for yes, 
     * null for unset).
     * 
     * @return whether or not the encoder has an inverted direction (false for no, true for yes,
     *     null for unset).
     */
    public Optional<Boolean> getInvertDirection() {
        return getBool(CanandmagDetails.kStg_InvertDirection);
    }


    /**
     * Gets whether or not the sensor should disallow zeroing and factory resets from the 
     * onboard button (0 for allow, 1 for disallow, -1 for unset).
     * 
     * @return whether or not the encoder has its onboard zero button's functionality disabled 
     * (false for allow, true for disallow, null for unset).
     */
    public Optional<Boolean> getDisableZeroButton() {
        return getBool(CanandmagDetails.kStg_DisableZeroButton);
    }

    /**
     * Gets the zero offset of the encoder.
     * 
     * The zero offset is subtracted from the raw reading of the encoder's magnetic sensor to
     * get the adjusted absolute position as returned by {@link Canandmag#getAbsPosition()}.
     * 
     * @return the zero offset [0..1), or null if the value has not been set on this object.
     */
    public Optional<Double> getZeroOffset() {
        return getIntAsDouble(CanandmagDetails.kStg_ZeroOffset, Canandmag.kCountsPerRotation);
    }

}