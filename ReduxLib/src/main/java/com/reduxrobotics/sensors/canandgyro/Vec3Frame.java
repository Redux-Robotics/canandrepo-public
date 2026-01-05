// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro;

import com.reduxrobotics.frames.Frame;

import edu.wpi.first.math.VecBuilder;
import edu.wpi.first.math.Vector;
import edu.wpi.first.math.numbers.N3;

/**
 * Implements an object-holding Frame with considerations for 3-vectors.
 * 
 * This avoids creation of new objects by only converting to a {@link Vector} when the value is 
 * requested, while also offering interfaces to efficiently retrieve components without allocation.
 */
public class Vec3Frame extends Frame<Vector<N3>> {

    private double x;
    private double y;
    private double z;
    private Vector<N3> defaultData;
    private Vector<N3> cache;
    private boolean dataValid;
    private double scaleFactor;

    /**
     * Instantiates a new Vec3Frame.
     * 
     * @param timestamp The timestamp the update happened at.
     * @param defaultData An instance of the object to return before the first update happens.
     * @param scaleFactor The conversion factor between one LSB and one vector unit.
     */
    public Vec3Frame(double timestamp, Vector<N3> defaultData, double scaleFactor) {
        super(timestamp);
        this.defaultData = defaultData;
        this.dataValid = false;
        this.scaleFactor = scaleFactor;
        this.cache = null;
    }

    @Override
    public synchronized Vector<N3> getValue() {
        if (!dataValid) return defaultData;
        if (cache == null) cache = VecBuilder.fill(x, y, z);
        return cache;
    }

    /**
     * The x (first) component.
     * @return x value
     */
    public synchronized double getX() {
        return x;
    }

    /**
     * The y (second) component.
     * @return x value
     */
    public synchronized double getY() {
        return y;
    }

    /**
     * The z (third) component.
     * @return z value
     */
    public synchronized double getZ() {
        return z;
    }

    /**
     * Returns if this frame has data.
     * @return if this frame's data can be considered valid
     */
    public synchronized boolean hasData() { return dataValid; }

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
    public synchronized void updateData(byte[] data, double timestamp) {
        z = ((short)((data[0] & 0xFF) | ((data[1] & 0xFF) << 8))) * scaleFactor;
        y = ((short)((data[2] & 0xFF) | ((data[3] & 0xFF) << 8))) * scaleFactor;
        x = ((short)((data[4] & 0xFF) | ((data[5] & 0xFF) << 8))) * scaleFactor;

        this.dataValid = true;
        this.cache = null;
        update(timestamp);
    }
}
