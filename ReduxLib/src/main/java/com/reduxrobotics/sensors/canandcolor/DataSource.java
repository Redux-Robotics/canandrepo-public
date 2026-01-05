// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Data sources that digout channels can use to perform comparisons or output data directly as a 
 * duty cycle on GPIO pin DIG-2.
 * 
 * <p>
 * All values are typically considered to be scaled between 0.0 and 1.0. For proximity, red, 
 * green, blue, white, and HSV, the values that digout slots consider (and what the Rio would 
 * read if it read the PWM outputs), they are generally equivalent to the values read through 
 * functions like {@link Canandcolor#getProximity()} or {@link Canandcolor#getColor()}.
 * </p>
 * 
 */
public enum DataSource implements DigoutPinConfig {
    /** Zero value (always reads zero) */
    kZero(0),
    /** Proximity value */
    kProximity(1),
    /** Red reading */
    kRed(2),
    /** Green reading */
    kGreen(3),
    /** Blue reading */
    kBlue(4),
    /** Hue reading */
    kHue(5),
    /** Saturation reading */
    kSaturation(6),
    /** Value reading */
    kValue(7);
    private int index;
    DataSource(int index) { this.index = index; } 

    /**
     * Returns the associated index number for the enum (used in serialization)
     * @return the associated index number for the enum.
     */
    public int getIndex() { return index; }
    private static final DataSource map[] = {
        kZero,
        kProximity, 
        kRed, 
        kGreen, 
        kBlue, 
        kHue,
        kSaturation,
        kValue};

    /**
     * Fetches the enum associated with the index value.
     * @param v int containing index value -- if index > max value, the modulo is used
     * @return corresponding enum
     */
    public static DataSource fromIndex(int v) {
        return map[v % map.length];
    }

    @Override
    public long toOutputSettingData() {
        return CanandcolorDetails.Stg.constructDigout2OutputConfig(
            CanandcolorDetails.Enums.DigoutOutputConfig.kDutyCycleOutput,
            index
        );
    }

    @Override
    public boolean equals(DigoutPinConfig other) {
        return (other instanceof DataSource) && ((DataSource) other).getIndex() == getIndex();
    }
}