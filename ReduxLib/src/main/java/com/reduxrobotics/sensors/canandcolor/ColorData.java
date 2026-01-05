// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import com.reduxrobotics.sensors.canandcolor.wpistruct.CanandcolorColorDataStruct;

import edu.wpi.first.util.struct.StructSerializable;
import edu.wpi.first.wpilibj.util.Color;

/**
 * Record class to hold detected color values.
 * 
 * @param red the amount of red detected between 0 to 1
 * @param green the amount of green detected between 0 to 1
 * @param blue the amount of blue detected between 0 to 1
 */
public record ColorData(double red, double green, double blue) implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final CanandcolorColorDataStruct struct = new CanandcolorColorDataStruct();
    /**
     * Converts to a WPILib {@link Color} object.
     * @return a Color with the rgb value.
     */
    public Color toWpilibColor() {
        return new Color(red, green, blue);
    }

    /**
     * Returns hue [0..1] inclusive in HSV colorspace.
     * @return hue
     */
    public double hue() {
        return hsvHue(red, green, blue);
    }

    /**
     * Returns saturation [0..1] in HSV colorspace.
     * @return sat
     */
    public double saturation() {
        return hsvSaturation(red, green, blue);
    }

    /**
     * Returns value [0..1] inclusive in HSV color space.
     * Not to be confused with the white value
     * @return value
     */
    public double value() {
        return hsvValue(red, green, blue);
    }

    static double hsvHue(double red, double green, double blue) {
        double Xmax = Math.max(red, Math.max(green, blue));
        double Xmin = Math.min(red, Math.min(green, blue));
        double C = Xmax - Xmin;

        if (C == 0.0) return 0;
        if (Xmax == red) { return (((green - blue) / C) % 6) / 6.0; }
        if (Xmax == green) { return (((blue - red) / C) + 2) / 6.0; }
        if (Xmax == blue) { return (((red - green) / C) + 4) / 6.0; }
        return 0;
    }

    static double hsvSaturation(double red, double green, double blue) {
        double Xmax = Math.max(red, Math.max(green, blue));
        double Xmin = Math.min(red, Math.min(green, blue));
        double C = Xmax - Xmin;

        if (Xmax == 0) return 0;
        
        return C / Xmax;
    }

    static double hsvValue(double red, double green, double blue) {
        return Math.max(red, Math.max(green, blue));
    }

    /**
     * Extracts a new ColorData object from a 64-bit data field.
     * @param data data to extraact from
     * @return new color data
     */
    public static ColorData fromColorMessage(long data) {
        int rawRed = CanandcolorDetails.Msg.extractColorOutput_Red(data);
        int rawGreen = CanandcolorDetails.Msg.extractColorOutput_Green(data);
        int rawBlue = CanandcolorDetails.Msg.extractColorOutput_Blue(data);
        // we don't need to take into account the period, as the sensor firmware pre-shifts
        // the data fields here
        //int period = CanandcolorDetails.Msg.extractColorOutput_Period(data);
        final double FACTOR = 1.0 / ((1 << 20)-1);

        return new ColorData(rawRed * FACTOR, rawGreen * FACTOR, rawBlue * FACTOR);
    }
}