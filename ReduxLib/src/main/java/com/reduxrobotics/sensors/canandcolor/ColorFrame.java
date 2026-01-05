// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import com.reduxrobotics.frames.Frame;

/**
 * Implements an object-holding Frame with considerations for color data.
 * 
 * This avoids creation of new objects by only converting to a {@link ColorData} when such an object is requested,
 * while also offering interfaces to efficiently retrieve components and HSV calculations without allocation.
 */
public class ColorFrame extends Frame<ColorData> {
    private double r;
    private double g;
    private double b;
    private ColorData defaultData;
    private ColorData cache;
    private ColorPeriod period;
    private boolean dataValid;

    /**
     * Instantiates a new ColorFrame 
     * 
     * @param timestamp initial timestamp
     * @param defaultData data to be returned when the frame value is not valid
     */
    public ColorFrame(double timestamp, ColorData defaultData) {
        super(timestamp);
        this.defaultData = defaultData;
        this.dataValid = false;
        this.cache = null;
    }

    @Override
    public synchronized ColorData getValue() {
        if (!dataValid) return defaultData;
        if (cache == null) cache = new ColorData(r, g, b);
        return cache;
    }

    /**
     * Retreives the frame's stored red value [0..1] inclusive.
     * @return red intensity
     */
    public synchronized double getRed() {
        return r;
    }

    /**
     * Retreives the frame's stored green value [0..1] inclusive.
     * @return green intensity
     */
    public synchronized double getGreen() {
        return g;
    }

    /**
     * Retreives the frame's stored blue value [0..1] inclusive.
     * @return blue intensity
     */
    public synchronized double getBlue() {
        return b;
    }

    /**
     * Computes the frame's HSV hue value [0..1] inclusive from the stored RGB data.
     * 
     * This function <b>does not</b> allocate objects!
     * @return hue
     */
    public synchronized double getHSVHue() {
        return ColorData.hsvHue(r, g, b);
    }


    /**
     * Computes the frame's HSV saturation value [0..1] inclusive from the stored RGB data.
     * 
     * This function <b>does not</b> allocate objects!
     * @return saturation 
     */
    public synchronized double getHSVSaturation() {
        return ColorData.hsvSaturation(r, g, b);
    }


    /**
     * Computes the frame's HSV value [0..1] inclusive from the stored RGB data.
     * 
     * This function <b>does not</b> allocate objects!
     * @return value
     */
    public synchronized double getHSVValue() {
        return ColorData.hsvValue(r, g, b);
    }

    /**
     * Retrieves the frame's stored color period.
     * @return color period
     */
    public synchronized ColorPeriod getColorPeriod() {
        return period;
    }

    @Override
    public boolean hasData() {
        return dataValid;
    }

    /**
     * Flag that this frame's data is not valid.
     */
    public synchronized void clearData() {
        this.dataValid = false;
    }

    /**
     * Update the Frame with new data.
     * @param data the new data to update with
     * @param timestamp the timestamp at which it occured
     */
    public synchronized void updateData(long data, double timestamp) {
        int rawRed = CanandcolorDetails.Msg.extractColorOutput_Red(data);
        int rawGreen = CanandcolorDetails.Msg.extractColorOutput_Green(data);
        int rawBlue = CanandcolorDetails.Msg.extractColorOutput_Blue(data);
        // we don't need to take into account the period, as the sensor firmware pre-shifts
        // the data fields here
        int rawPeriod = CanandcolorDetails.Msg.extractColorOutput_Period(data);
        final double FACTOR = 1.0 / ((1 << 20)-1);

        this.r = rawRed * FACTOR; 
        this.g = rawGreen * FACTOR;
        this.b = rawBlue * FACTOR;
        this.period = ColorPeriod.fromIndex(rawPeriod);
        this.dataValid = true;
        this.cache = null;

        update(timestamp);
    }
    
}
