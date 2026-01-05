// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.frames;

/**
 * Implements an object-holding Frame backed by a double.
 * 
 * This avoids creation of new objects by only converting to objects when the value is requested
 * @param <T> the type of object the frame holds.
 */
public class DoubleFrame<T> extends Frame<T> {

    /**
     * Functional interface for a function mapping a double data value to the final frame type.
     * @param <T> the type of object the frame holds.
     */
    public static interface DoubleToType<T> {
        /**
         * Conversion function from double to the type parameter.
         * @param data The double data. 
         * @return a new object.
         */
        T convert(double data);
    }

    private double data;
    private DoubleToType<T> conv;
    private T defaultData;
    private T cache;
    private boolean dataValid;

    /**
     * Instantiates a new DoubleFrame.
     * 
     * @param initialData The initial double data for the frame to hold.
     * @param timestamp The timestamp the update happened at.
     * @param defaultData An instance of the object to return before the first update happens.
     * @param conversion A function that takes in a double and converts it to the final datatype.
     */
    public DoubleFrame(double initialData, double timestamp, T defaultData, DoubleToType<T> conversion) {
        super(timestamp);
        this.data = initialData;
        this.conv = conversion;
        this.defaultData = defaultData;
        this.dataValid = false;
        this.cache = null;
    }

    @Override
    public synchronized T getValue() {
        if (!dataValid) return defaultData;
        if (cache == null) cache = conv.convert(data);
        return cache;
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
     * Gets the underlying double data.
     * @return the double data as a type.
     */
    public synchronized double getData() {
        return data;
    }

    /**
     * Update the DoubleFrame with new double-backed data.
     * @param data the new data to update with
     * @param timestamp the timestamp at which it occured
     */
    public synchronized void updateData(double data, double timestamp) {
        this.data = data;
        this.dataValid = true;
        this.cache = null;
        update(timestamp);
    }
}
