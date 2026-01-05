// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.frames;

/**
 * Implements an object-holding Frame backed by a byte array.
 * 
 * This avoids creation of new objects by only converting to objects when the value is requested.
 * @param <T> the type of object the frame holds.
 */
public class ByteArrayFrame<T> extends Frame<T> {

    /**
     * Functional interface for a function mapping a double data value to the final frame type.
     * For byte arrays this should probably ensure no references to the original array are present 
     * (as they update)
     * @param <T>the type of object the to convert to
     */
    public static interface ByteArrayToType<T> {
        /**
         * Converison function from byte[] to the type parameter.
         * @param data The byte array data. If preserved in an output object, it should be cloned 
         *    first to avoid data getting overwritten on next update.
         * @return a new object.
         */
        T convert(byte[] data);
    }

    private byte[] data;
    private ByteArrayToType<T> conv;
    private T defaultData;
    private T cache;
    private boolean dataValid;

    /**
     * Instantiates a new ByteArrayFrame.
     * 
     * @param capacity The capacity of the byte array value held.
     * @param timestamp The timestamp the update happened at.
     * @param defaultData An instance of the object to return before the first update happens.
     * @param conversion A function that takes in a double and converts it to the final datatype.
     */
    public ByteArrayFrame(int capacity, double timestamp, T defaultData, ByteArrayToType<T> conversion) {
        super(timestamp);
        this.data = new byte[capacity];
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
     * Gets the underlying byte array data.
     * @return the byte array data as a type.
     */
    public synchronized byte[] getData() {
        return data;
    }

    /**
     * Update the ByteArrayFrame with new byte array data.
     * @param data the new data to update with
     * @param timestamp the timestamp at which it occured
     */
    public synchronized void updateData(byte[] data, double timestamp) {
        System.arraycopy(data, 0, this.data, 0, Math.min(data.length, this.data.length));
        this.dataValid = true;
        this.cache = null;
        update(timestamp);
    }
}
