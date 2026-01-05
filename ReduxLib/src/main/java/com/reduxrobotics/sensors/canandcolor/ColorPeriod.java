// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Enum representing the internal configuration of the Canandcolor's color IC.
 * Passed to {@link CanandcolorSettings#setColorIntegrationPeriod(ColorPeriod)} which may be then used with 
 * {@link Canandcolor#setSettings(CanandcolorSettings)} to configure the device.
 * 
 * <p>
 * The color sensor IC supports a single parameter that serves as both the integration and 
 * sampling period, which can be adjusted from 25 millisecondss to 400 milliseconds.
 * This period functions as a kind of "exposure time" -- longer periods mean colors at lower 
 * light can be read more accurately, but also mean they saturate faster in higher light 
 * conditions and update less frequently, regardless of the device frame period.
 * </p>
 * 
 * <p>
 * The rate at which the color sensor produces new values once every integration period, so {@link #k400ms} will 
 * produce a new color reading every 400 milliseconds, and {@link #k25ms} will produce a new reading every 25 milliseconds.
 * </p>
 * 
 * The factory default is {@link #k25ms}.
 */
public enum ColorPeriod {
    /** Sample the color sensor every 400 ms. */
    k400ms(CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod400MsResolution20Bit),
    /** Sample the color sensor every 200 ms. */
    k200ms(CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod200MsResolution19Bit),
    /** Sample the color sensor every 100 ms. */
    k100ms(CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod100MsResolution18Bit),
    /** Sample the color sensor every 50 ms. */
    k50ms(CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod50MsResolution17Bit),
    /** Sample the color sensor every 25 ms. */
    k25ms(CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod25MsResolution16Bit);
    private int index;
    ColorPeriod(int index) { this.index = index; } 

    /**
     * Returns a corresponding color config from the index.
     * @param idx the index to fetch.
     * @return a valid color config. If the index value is invalid it will return the default.
     */
    public static ColorPeriod fromIndex(int idx) { 
        switch (idx) {
            // If you know the color sensor IC, you may be wondering why we didn't include 3.125 ms/13 bit color;
            // 3.125ms resolution, which technically settable on the sensor IC, in practice is strictly worse 
            // than using the 25 ms period because the sensor IC only yields new readings every 25 ms at best.
            // Using it is a footgun, which is why it's not supported.
            case CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod25MsResolution16Bit: { return ColorPeriod.k25ms; }
            case CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod50MsResolution17Bit: { return ColorPeriod.k50ms; }
            case CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod100MsResolution18Bit: { return ColorPeriod.k100ms; }
            case CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod200MsResolution19Bit: { return ColorPeriod.k200ms; }
            case CanandcolorDetails.Enums.ColorIntegrationPeriod.kPeriod400MsResolution20Bit: { return ColorPeriod.k400ms; }
            default: { return ColorPeriod.k25ms; }
        }
    }

    /**
     * Gets the corresponding index for the value in question.
     * @return the index value for the enum (used internally)
     */
    public int getIndex() { return index; }
}