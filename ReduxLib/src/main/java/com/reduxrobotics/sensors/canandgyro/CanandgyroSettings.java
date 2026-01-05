// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro;

import java.util.HashMap;
import java.util.Optional;

import com.reduxrobotics.canand.CanandSettings;

/**
 * The settings class for the {@link Canandgyro}.
 * 
 * <p>
 * This class holds settings values that can be used to reconfigure Canandgyro via 
 * {@link Canandgyro#setSettings}. 
 * Additionally, objects of this class are returned from {@link Canandgyro#getSettings} and
 * {@link Canandgyro#getSettingsAsync} which can be used to read the device's settings.
 * </p>
 * 
 * <pre>
 * // Object initialization
 * Canandgyro canandgyro = new Canandgyro(0);
 * 
 * // Only settings that are explicitly set here will be edited, so other settings 
 * // such as the status frame period will remain untouched.
 * // For example, canandgyro.setSettings(new CanandgyroSettings()); will be a no-op.
 * 
 * canandgyro.setSettings(new CanandgyroSettings()
 *     .setYawFramePeriod(0.005) // sets the rate of gyro measurements to every 5 ms
 *     .setAngularPositionFramePeriod(0) // disables angular position updates. 
 *                                       // Yaw uses a separate frame and is unaffected.
 *     .setAngularVelocityFramePeriod(0.01) // set angular velocity to every 10 ms
 *     .setAccelerationFramePeriod(0) // disable acceleration frames
 * );
 * // canandgyro.setSettings will block by default for up to 350 ms to confirm all values were set.
 * 
 * </pre>
 * 
 * 
 * Objects returned by the blocking {@link Canandgyro#getSettings} method may return {@link Optional#empty()} on its
 * getters. This occurs when getSettings would've previously returned {@link Optional#empty()} -- however, this 
 * allows for partial settings fetches. This scenario is unlikely (assuming the device is 
 * attached) as failed settings fetches are by default retried up to 3 times.
 * To check if the settings fetch succeeded, one can use {@link #allSettingsReceived()} to check
 * all fields are populated.
 * 
 * Example blocking fetch:
 * <pre>
 * // Object initialization
 * Canandgyro canandgyro = new Canandgyro(0);
 * 
 * // Robot code 
 * CanandgyroSettings stg = canandgyro.getSettings(0.5); // wait up to 500 ms
 * if (stg.allSettingsReceived()) {
 *     // print the status frame period (usually 100 ms)
 *     System.out.printf("status frame period: %d\n", stg.getStatusFramePeriod());
 * }
 * </pre>
 */
public class CanandgyroSettings extends CanandSettings {
    /**
     * Instantiates a blank {@link CanandgyroSettings} object with no settings to be set.
     */
    public CanandgyroSettings() { 
        values = new HashMap<>();
    }

    /**
     * Instantiates a new {@link CanandgyroSettings} object that copies its settings from the
     * input instance.
     * 
     * @param toCopy the input settings object to copy
     */
    public CanandgyroSettings(CanandgyroSettings toCopy) {
        values = toCopy.getFilteredMap();
    }

    @Override
    protected int[] fetchSettingsAddresses() {
        return CanandgyroDetails.Stg.settingsAddresses;
    }

    /**
     * Sets the dedicated yaw frame period in seconds.
     * 
     * By factory default, yaw frames are sent every 10 milliseconds (period = 0.010).
     * If 0 is passed in, yaw frames will be disabled and {@link Canandgyro#getYaw} will not
     * return new values (unless configured to derive yaw from the angular position frame)
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     * @return the calling object, so these calls can be chained
     */
    public CanandgyroSettings setYawFramePeriod(double period) {
        values.put(CanandgyroDetails.Stg.kYawFramePeriod, 
            checkBounds("yaw frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the angular position frame period in seconds.
     * 
     * By factory default, angular position frames are sent every 20 milliseconds (period = 0.10).
     * If 0 is passed in, angular position frames will be disabled and methods returning angular
     * position data will not return new values.
     * 
     * <p>The one exception is {@link Canandgyro#getYaw} which by default uses the yaw frame instead.</p>
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     * @return the calling object, so these calls can be chained
     */
    public CanandgyroSettings setAngularPositionFramePeriod(double period) {
        values.put(CanandgyroDetails.Stg.kAngularPositionFramePeriod, 
            checkBounds("angular position frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the angular velocity frame period in seconds.
     * 
     * By factory default, angular velocity frames are sent every 100 milliseconds (period = 0.100).
     * If 0 is passed in, angular velocity frames will be disabled and methods returning angular 
     * velocity data will not return new values.
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     * @return the calling object, so these calls can be chained
     */
    public CanandgyroSettings setAngularVelocityFramePeriod(double period) {
        values.put(CanandgyroDetails.Stg.kAngularVelocityFramePeriod, 
            checkBounds("angular velocity frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the angular velocity frame period in seconds.
     * 
     * By factory default, acceleration frames are sent every 100 milliseconds (period = 0.100).
     * If 0 is passed in, acceleration frames will be disabled and methods returning acceleration
     * data will not return new values.
     * 
     * @param period the new frame period in seconds [0_s, 65.535_s] inclusive
     * @return the calling object, so these calls can be chained
     */
    public CanandgyroSettings setAccelerationFramePeriod(double period) {
        values.put(CanandgyroDetails.Stg.kAccelerationFramePeriod, 
            checkBounds("angular velocity frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the status frame period in seconds. 
     * 
     * By factory default, the device will broadcast 10 status messages every second (period=0.1). 
     * 
     * @param period the new period for status frames in seconds in range [0.001_s, 16.383_s].
     * @return the calling object, so these calls can be chained
     */
    public CanandgyroSettings setStatusFramePeriod(double period) {
        values.put(CanandgyroDetails.Stg.kStatusFramePeriod, 
            checkBounds("status frame period", period, 1, 16383, 1000));
        return this;
    }

    /**
     * Gets the dedicated yaw frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has not 
     * been set on this object.
     * 
     * A value of 0 means yaw frames are disabled.
     * @return the frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has not been set on
     *     this object.
     */
    public Optional<Double> getYawFramePeriod() {
        return getIntAsDouble(CanandgyroDetails.Stg.kYawFramePeriod, 1000);
    }

    /**
     * Gets the angular position frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has 
     * not been set on this object.
     * 
     * A value of 0 means angular position frames are disabled.
     * @return the frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has not been set on
     *     this object.
     */
    public Optional<Double> getAngularPositionFramePeriod() {
        return getIntAsDouble(CanandgyroDetails.Stg.kAngularPositionFramePeriod, 1000);
    }

    /**
     * Gets the angular velocity frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has 
     * not been set on this object.
     * 
     * A value of 0 means angular velocity frames are disabled.
     * @return the frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has not been set on
     *     this object.
     */
    public Optional<Double> getAngularVelocityFramePeriod() {
        return getIntAsDouble(CanandgyroDetails.Stg.kAngularVelocityFramePeriod, 1000);
    }

    /**
     * Gets the acceleration frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has not 
     * been set on this object.
     * 
     * A value of 0 means acceleration frames are disabled.
     * @return the frame period in seconds [0..65.535], or {@link Optional#empty()} if the value has not been set on
     *     this object.
     */
    public Optional<Double> getAccelerationFramePeriod() {
        return getIntAsDouble(CanandgyroDetails.Stg.kAccelerationFramePeriod, 1000);
    }

    /**
     * Gets the status frame period in seconds [0.001..65.535], or {@link Optional#empty()} if the value has not beeno
     * set on this object.
     * @return the status frame period in seconds [0.001..65.535], or {@link Optional#empty()} if the value has not been
     *     set on this object.
     */
    public Optional<Double> getStatusFramePeriod() {
        return getIntAsDouble(CanandgyroDetails.Stg.kStatusFramePeriod, 1000);
    }

}