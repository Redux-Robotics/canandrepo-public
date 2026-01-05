// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import java.util.HashMap;
import java.util.Optional;

import com.reduxrobotics.canand.CanandSettings;

/**
 * The settings class for the {@link Canandcolor}.
 * 
 * <p>
 * This class holds settings values that can be used to reconfigure Canandcolor via 
 * {@link Canandcolor#setSettings(CanandcolorSettings)}. 
 * Additionally, objects of this class are returned from {@link Canandcolor#getSettings()} and
 * {@link Canandcolor#getSettingsAsync()} which can be used to read the device's settings.
 * </p>
 * 
 * <pre>
 * // Object initialization
 * Canandcolor canandcolor = new Canandcolor(0);
 * 
 * // Only settings that are explicitly set here will be edited, so other settings 
 * // such as the status frame period will remain untouched.
 * // For example, canandcolor.setSettings(new CanandcolorSettings()); will be a no-op.
 * 
 * canandcolor.setSettings(new CanandcolorSettings()
 *     .setProximityFramePeriod(0) // disables proximity reading updates
 *     .setColorFramePeriod(0.10) // sets the rate of color measurements to every 20 ms
 *     .setColorIntegrationPeriod(ColorPeriod.k25ms) // sets the color integration period to 25 milliseconds 
 * );
 * // canandcolor.setSettings will block by default for up to 350 ms to confirm all values were set.
 * 
 * </pre>
 * 
 * <p>
 * To check if the settings fetch succeeded, one can use {@link #allSettingsReceived()} to check
 * all fields are populated. If that flag is true, all {@link Optional} values returned by the getters
 * can be assumed to <b>not</b> be {@link Optional#empty()}.
 * </p>
 * 
 * Example blocking fetch:
 * <pre>
 * // Object initialization
 * Canandcolor canandcolor = new Canandcolor(0);
 * 
 * // Robot code 
 * CanandcolorSettings stg = canandcolor.getSettings(0.5); // wait up to 500 ms
 * if (stg.allSettingsReceived()) {
 *     // print the status frame period (usually 100 ms)
 *     System.out.printf("status frame period: %d\n", stg.getStatusFramePeriod().get());
 * }
 * </pre>
 */
public class CanandcolorSettings extends CanandSettings {

    @Override
    protected int[] fetchSettingsAddresses() {
        return CanandcolorDetails.Stg.settingsAddresses;
    }

    /**
     * Instantiates a new {@link CanandcolorSettings} object that is "completely blank" -- 
     * holding no settings values at all.
     * 
     * Settings are only populated into the {@link CanandcolorSettings} object explicitly 
     * through the various setter methods -- running 
     * {@code canandcolor.setSetting(new CanandcolorSettings())} would not update the device 
     * at all.
     * 
     * To reset a device back to factory defaults, use {@link Canandcolor#resetFactoryDefaults()}
     */
    public CanandcolorSettings() {
        values = new HashMap<>();
    }

    /**
     * Instantiates a new {@link CanandcolorSettings} object that copies its settings from the
     * input instance.
     * 
     * @param toCopy the input settings object to copy
     */
    public CanandcolorSettings(CanandcolorSettings toCopy) {
        values = toCopy.getFilteredMap();
    }

    /**
     * Sets the status frame period in seconds. 
     * 
     * By factory default, the device will broadcast 10 status messages per second (period=0.1). 
     * 
     * @param period the new period for status frames in seconds in range [0.001_s, 16.383_s].
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setStatusFramePeriod(double period) {
        values.put(CanandcolorDetails.Stg.kStatusFramePeriod, 
            checkBounds("status frame period", period, 1, 16383, 1000));
        return this;
    }

    /**
     * Sets the proximity frame period in seconds. 
     * 
     * <p>
     * By factory default, proximity frames are broadcast every 20 milliseconds (period=0.1). 
     * If 0 is passed in, proximity frames will be disabled and {@link Canandcolor#getProximity()}
     * will not return new values.
     * </p>
     * 
     * <p>Note that if {@link #setAlignProximityFramesToIntegrationPeriod(boolean)} is enabled,
     * then this becomes the <i>maximum</i> period between two proximity frames, as proximity frames
     * may get scheduled to broadcast early at the rate of the {@link ProximityPeriod proximity integration period}
     * instead to minimize latency; but setting this option to zero still disables all proximity frames.
     * </p>
     * 
     * @param period the new period for proximity frames in seconds in range [0_s, 65.535_s].
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setProximityFramePeriod(double period) {
        values.put(CanandcolorDetails.Stg.kDistanceFramePeriod, 
            checkBounds("proximity frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the color frame period in seconds. 
     * 
     * <p>
     * By factory default, color frames are broadcast every 25 milliseconds (period=0.15). 
     * If 0 is passed in, color frames will be disabled and {@link Canandcolor#getColor()} and
     *  other color-reading methods will not return new values.
     * </p>
     * 
     * <p>Note that if {@link #setAlignColorFramesToIntegrationPeriod(boolean)} is enabled,
     * then this becomes the <i>maximum</i> period between two proximity frames, as proximity frames
     * may get scheduled to broadcast early at the rate of the {@link ColorPeriod color integration period}
     * instead to minimize latency; but setting this option to zero still disables all color frames.
     * </p>
     * 
     * @param period the new period for color frames in seconds in range [0_s, 65.535_s].
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setColorFramePeriod(double period) {
        values.put(CanandcolorDetails.Stg.kColorFramePeriod, 
            checkBounds("color frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the digital output (digout) frame period in seconds. 
     * 
     * <p>
     * By factory default, digout frames are broadcast every 100 milliseconds (period=0.10). 
     * If 0 is passed in, digout frames will be disabled and {@link Canandcolor#getDigoutState()} 
     * and {@link DigoutChannel#getValue()}/{@link DigoutChannel#getStickyValue()} will not 
     * return new values.
     * </p>
     * 
     * @param period the new period for digout frames in seconds in range [0_s, 65.535_s].
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setDigoutFramePeriod(double period) {
        values.put(CanandcolorDetails.Stg.kDigoutFramePeriod, 
            checkBounds("digout frame period", period, 0, 65535, 1000));
        return this;
    }

    /**
     * Sets the brightness of the onboard lamp LED.
     * 
     * <p>
     * The LED can also be physically turned off regardless of setting with the onboard switch.
     * </p>
     * By factory default this setting is set to max brightness (1.0)
     * @param brightness scaled brightness from 0.0 (off) to 1.0 (max brightness)
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setLampLEDBrightness(double brightness) {
        values.put(CanandcolorDetails.Stg.kLampBrightness, 
            checkBounds("lamp brightness", brightness, 0, 36000, 36000));
        return this;
    }

    /**
     * Sets the sampling/integration period for the color sensor.
     * 
     * <p>
     * If {@link #setAlignColorFramesToIntegrationPeriod(boolean)} is set, this also effectively
     * determines the frame period of the color frame as well.
     * </p>
     * 
     * @param period the {@link ColorPeriod} to apply
     * @return the calling object, so these calls can be chained
     * @see ColorPeriod
     */
    public CanandcolorSettings setColorIntegrationPeriod(ColorPeriod period) {
        values.put(CanandcolorDetails.Stg.kColorIntegrationPeriod, (long) period.getIndex());
        return this;
    }

    /**
     * Sets the integration period/multipulse configuration for the proximity sensor.
     * <p>
     * If {@link #setAlignProximityFramesToIntegrationPeriod(boolean)} is set, this also effectively
     * determines the frame period of the proximity frame as well.
     * </p>
     * @param period the {@link ProximityPeriod} to apply
     * @return the calling object, so these calls can be chained
     * @see ProximityPeriod
     */
    public CanandcolorSettings setProximityIntegrationPeriod(ProximityPeriod period) {
        values.put(CanandcolorDetails.Stg.kDistanceIntegrationPeriod, (long) period.getIndex());
        return this;
    }

    /**
     * Configures the physical GPIO pin associated with a given digital output channel.
     * 
     * <p>Note that these pin outputs are independent of the actual digital output channel's value,
     * which is always continuously calcuated from digout slots.
     * </p>
     * 
     * <p>
     * These pins can be set into one of two or three modes:
     * </p>
     * 
     * <ul>
     * <li>output disabled, by passing in {@link DigoutPinConfig#kDisabled}</li> 
     * <li>output the value from the digout channel by passing in {@link DigoutPinConfig#kDigoutLogicActiveHigh}
     * or {@link DigoutPinConfig#kDigoutLogicActiveLow} </li>
     * <li>(only supported on {@link DigoutChannel.Index#kDigout2 kDigout2}) a duty cycle (PWM) output of values 
     * from either the color or proximity sensor, by passing in a {@link DataSource}) object </li>
     * </ul>
     * See {@link DigoutChannel#configureOutputPin(DigoutPinConfig)} for how to pass in a {@link DigoutPinConfig}.
     * 
     * @param digout the channel to configure (either {@link DigoutChannel.Index#kDigout1} or 
     *               {@link DigoutChannel.Index#kDigout2}).
     *               Note that channel 1 does not support duty cycle output.
     * @param config a {@link DigoutPinConfig} specifying how the digout should output if at all.
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setDigoutPinConfig(DigoutChannel.Index digout, DigoutPinConfig config) {
        int idx = (digout == DigoutChannel.Index.kDigout1) ? CanandcolorDetails.Stg.kDigout1OutputConfig
                                                          : CanandcolorDetails.Stg.kDigout2OutputConfig;
        if (config instanceof DataSource && digout == DigoutChannel.Index.kDigout1) {
            throw new IllegalArgumentException("Digout 1 does not support duty cycle GPIO output!");
        }
        values.put(idx, config.toOutputSettingData());
        return this;
    }

    /**
     * Sets digout message triggers which control if the Canandcolor should send digout messages on state change.
     * 
     * <p>These triggers can function even if digout output for the corresponding {@link DigoutChannel.Index} via 
     * {@link #setDigoutPinConfig(DigoutChannel.Index, DigoutPinConfig)} is disabled or set to duty cycle output.
     * This allows user code to quickly react to digout logic purely over CAN without needing direct GPIO digital I/O connections.
     * </p>
     * 
     * @param digout the digout to configure (either {@link DigoutChannel.Index#kDigout1} or 
     *     {@link DigoutChannel.Index#kDigout2})
     * @param trg digout message trigger
     * @return the calling object, so these calls can be chained
     * @see DigoutFrameTrigger
     */
    public CanandcolorSettings setDigoutFrameTrigger(DigoutChannel.Index digout, DigoutFrameTrigger trg) {
        int idx = (digout == DigoutChannel.Index.kDigout1) ? CanandcolorDetails.Stg.kDigout1MessageOnChange
                                                          : CanandcolorDetails.Stg.kDigout2MessageOnChange;
        values.put(idx, (long) trg.getIndex());
        return this;
    }

    /**
     * Sets whether or not to align the transmission of proximity frames to the integration period of the proximity sensor.
     * 
     * <p>
     * This setting makes the Cannadcolor transmit proximity frames whenever it finishes processing new proximity data. 
     * </p>
     * <p>
     * For example, if the proximity frame period is normally set to 200 ms, and the configured {@link ProximityPeriod} is 10 ms, 
     * passing true to this method would make the sensor emit a new proximity frame every 10 ms when the proximity sensor updates.
     * </p>
     * 
     * 
     * @param align true if to enable this feature
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setAlignProximityFramesToIntegrationPeriod(boolean align) {
        values.put(CanandcolorDetails.Stg.kDistanceExtraFrameMode, (align) ? 1L : 0L);
        return this;
    }

    /**
     * Sets whether or not to align the transmission of color frames to the integration period of the color sensor.
     * 
     * <p>
     * This setting makes the Cannadcolor transmit color frames whenever it finishes processing new color data. 
     * </p>
     * <p>
     * For example, if the color frame period is normally set to 200 ms, and the configured {@link ColorPeriod} is 25ms, 
     * passing true to this method would make the sensor emit a new color frame every 25ms when the color sensor updates.
     * </p>
     * 
     * @param align true if to enable this feature
     * @return the calling object, so these calls can be chained
     */
    public CanandcolorSettings setAlignColorFramesToIntegrationPeriod(boolean align) {
        values.put(CanandcolorDetails.Stg.kColorExtraFrameMode, (align) ? 1L : 0L);
        return this;
    }


    // digout slots are going to be handled outside the settings tree.

    /**
     * Gets the status frame period in seconds [0.001..65.535], or an empty {@link Optional} if
     * the value has not been set on this object.
     * 
     * @return the status frame period in seconds [0.001..65.535], or an empty {@link Optional}
     * if the value has not been set on this object.
     */
    public Optional<Double> getStatusFramePeriod() {
        return getIntAsDouble(CanandcolorDetails.Stg.kStatusFramePeriod, 1000);
    }

    /**
     * Gets the proximity frame period in seconds [0..65.535], or an empty {@link Optional} if
     * the value has not been set on this object.
     * 
     * A value of 0 means proximity messages are disabled.
     * @return the frame period in seconds [0..65.535], or an empty {@link Optional} if the
     * value has not been set on this object.
     */
    public Optional<Double> getProximityFramePeriod() {
        return getIntAsDouble(CanandcolorDetails.Stg.kDistanceFramePeriod, 1000);
    }

    /**
     * Gets the color frame period in seconds [0..65.535], or an empty {@link Optional} if the
     * value has not been set on this object.
     * 
     * A value of 0 means color messages are disabled.
     * @return the frame period in seconds [0..65.535], or an empty {@link Optional} if the
     * value has not been set on this object.
     */
    public Optional<Double> getColorFramePeriod() {
        return getIntAsDouble(CanandcolorDetails.Stg.kColorFramePeriod, 1000);
    }

    /**
     * Gets the digout status frame period in seconds [0..65.535], or an empty {@link Optional}
     * if the value has not been set on this object.
     * 
     * A value of 0 means digout status messages are disabled.
     * @return the frame period in seconds [0..65.535], or an empty {@link Optional} if the
     * value has not been set on this object.
     */
    public Optional<Double> getDigoutFramePeriod() {
        return getIntAsDouble(CanandcolorDetails.Stg.kDigoutFramePeriod, 1000);
    }

    /**
     * Gets the lamp LED's brightness, scaled from 0.0 (off) to 1.0 (max)
     * @return the brightness factor [0.0..1.0] or an empty {@link Optional} if unset
     */
    public Optional<Double> getLampLEDBrightness() {
        return getIntAsDouble(CanandcolorDetails.Stg.kLampBrightness, 36000);
    }

    /**
     * Gets the sampling/integration period for the color sensor, if set on this object.
     * @return {@link ColorPeriod} enum or {@link Optional#empty} if unset
     */
    public Optional<ColorPeriod> getColorIntegrationPeriod() {
        if (!values.containsKey(CanandcolorDetails.Stg.kColorIntegrationPeriod)) { return Optional.empty(); }
        long data = values.get(CanandcolorDetails.Stg.kColorIntegrationPeriod);
        return Optional.of(ColorPeriod.fromIndex((int) data));
    }

    /**
     * Gets the sampling/integration period for the proximity sensor, if set on this object.
     * @return {@link ProximityPeriod} or {@link Optional#empty} if unset
     */
    public Optional<ProximityPeriod> getProximityIntegrationPeriod() {
        if (!values.containsKey(CanandcolorDetails.Stg.kDistanceIntegrationPeriod)) { return Optional.empty(); }
        long data = values.get(CanandcolorDetails.Stg.kDistanceIntegrationPeriod);
        return Optional.of(ProximityPeriod.fromIndex((int) data));
    }

    /**
     * Gets the output configuration for a physical GPIO pin associated with the given
     * digout channel.
     * @param digout the digital output config to fetch
     * @return the {@link DigoutPinConfig} or {@link Optional#empty} if unset
     */
    public Optional<DigoutPinConfig> getDigoutPinConfig(DigoutChannel.Index digout) {
        Long data = values.get(switch (digout) {
            case kDigout1 -> CanandcolorDetails.Stg.kDigout1OutputConfig;
            case kDigout2 -> CanandcolorDetails.Stg.kDigout2OutputConfig;
        }); 
        if (data == null) return Optional.empty();
        return Optional.of(DigoutPinConfig.fromSettingData(data));
    }

    /**
     * Gets the frame trigger configuration for a digital output.
     * @param digout the digital output config to fetch
     * @return the {@link DigoutFrameTrigger} or {@link Optional#empty} if unset
     */
    public Optional<DigoutFrameTrigger> getDigoutFrameTrigger(DigoutChannel.Index digout) {
        Long data = values.get(switch (digout) {
            case kDigout1 -> CanandcolorDetails.Stg.kDigout1MessageOnChange;
            case kDigout2 -> CanandcolorDetails.Stg.kDigout2MessageOnChange;
        }); 
        if (data == null) return Optional.empty();
        return Optional.of(DigoutFrameTrigger.fromIndex(data.intValue()));
    }

    /**
     * Gets the config for proximity frame period alignment with integration period.
     * @return the config state (true if enabled) or {@link Optional#empty} if unset
     * @see #setAlignProximityFramesToIntegrationPeriod(boolean)
     */
    public Optional<Boolean> getAlignProximityFramesToIntegrationPeriod() {
        if (!values.containsKey(CanandcolorDetails.Stg.kDistanceExtraFrameMode)) { return Optional.empty(); }
        long data = values.get(CanandcolorDetails.Stg.kDistanceExtraFrameMode);
        return Optional.of(data != 0);
    }

    /**
     * Gets the config for color frame period alignment with integration period.
     * @return the config state (true if enabled) or {@link Optional#empty} if unset
     * @see #setAlignColorFramesToIntegrationPeriod(boolean)
     */
    public Optional<Boolean> getAlignColorFramesToIntegrationPeriod() {
        if (!values.containsKey(CanandcolorDetails.Stg.kColorExtraFrameMode)) { return Optional.empty(); }
        long data = values.get(CanandcolorDetails.Stg.kColorExtraFrameMode);
        return Optional.of(data != 0);
    }

}