// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Interface that holds the output configuration for a physical digital output pin (digout)
 * 
 * <p>Digout pins can be set into one of three modes:</p>
 * <ul>
 * <li>disabled (via {@link #kDisabled}) </li> 
 * <li>outputting high or low depending on the values of digout slots that combine to form a boolean expression 
 * (via {@link #kDigoutLogicActiveLow} and {@link #kDigoutLogicActiveHigh} -- for more information, see {@link DigoutChannel})  </li>
 * <li>a duty cycle/PWM output of values from either the color or proximity sensor via {@link DataSource} (only works on digout 2) </li>
 * </ul>
 * 
 * See {@link DigoutChannel#configureOutputPin(DigoutPinConfig)} for example usage.
 * 
 */
public sealed interface DigoutPinConfig permits DataSource, DigoutPinConfig.RawConfigValue {

    /** Disables all output on this digout GPIO pin. */
    public static final DigoutPinConfig kDisabled = new RawConfigValue(
        CanandcolorDetails.Stg.constructDigout2OutputConfig(CanandcolorDetails.Enums.DigoutOutputConfig.kDisabled, 0)
    );

    /** Sets the digout GPIO pin to use the associated {@link DigoutChannel digout channel's} value, with 3.3v as true and 0v as false. */
    public static final DigoutPinConfig kDigoutLogicActiveHigh = new RawConfigValue(
        CanandcolorDetails.Stg.constructDigout2OutputConfig(CanandcolorDetails.Enums.DigoutOutputConfig.kDigoutLogicActiveHigh, 0)
    );

    /** Sets the digout GPIO pin to use the associated {@link DigoutChannel digout channel's} value, with 0v as true and 3.3v as false. */
    public static final DigoutPinConfig kDigoutLogicActiveLow = new RawConfigValue(
        CanandcolorDetails.Stg.constructDigout2OutputConfig(CanandcolorDetails.Enums.DigoutOutputConfig.kDigoutLogicActiveLow, 0)
    );

    /**
     * Represents a raw config value.
     * Users should not need to instantiate this class.
     * @param stgData raw setting data.
     */
    static record RawConfigValue(long stgData) implements DigoutPinConfig {
        @Override
        public long toOutputSettingData() {
            return stgData;
        }

        @Override
        public boolean equals(DigoutPinConfig other) {
            return other instanceof RawConfigValue && ((RawConfigValue) other).stgData == stgData;
        }
    }

    /**
     * Serializes the digout config into a value writeable to device settings.
     * @return 48-bit long
     */
    long toOutputSettingData();

    /**
     * Returns if two configurations are equivalent.
     * @param other other config
     * @return true if equal
     */
    boolean equals(DigoutPinConfig other);

    /**
     * Unserializes the digout config from setting data to an object.
     * @param value setting data
     * @return config object
     */
    public static DigoutPinConfig fromSettingData(long value) {
        return switch (CanandcolorDetails.Stg.extractDigout2OutputConfig_OutputConfig(value)) {
            case CanandcolorDetails.Enums.DigoutOutputConfig.kDisabled -> kDisabled;
            case CanandcolorDetails.Enums.DigoutOutputConfig.kDigoutLogicActiveHigh-> kDigoutLogicActiveHigh;
            case CanandcolorDetails.Enums.DigoutOutputConfig.kDigoutLogicActiveLow -> kDigoutLogicActiveLow;
            case CanandcolorDetails.Enums.DigoutOutputConfig.kDutyCycleOutput -> DataSource.fromIndex(
                CanandcolorDetails.Stg.extractDigout2OutputConfig_PwmDataSource(value)
            );
            default -> kDisabled;
        };
    }
}
