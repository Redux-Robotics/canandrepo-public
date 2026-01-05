// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.frames;

/**
 * Immutable container class for timestamped values.
 * @param <T>the type of the contained value.
 */
public class FrameData<T> {
    private double ts; 
    private T value;

    /**
     * Constructs a new FrameData object.
     * 
     * @param value The value to hold.
     * @param timestamp The timestamp at which the value was received in seconds.
     */
    public FrameData(T value, double timestamp) {
        this.value = value;
        this.ts = timestamp;
    }

    /**
     * Returns the value of the data frame.
     * @return the value the data frame holds.
     */
    public T getValue() {
        return value;
    }

    /**
     * Gets the timestamp in seconds of when this value was updated.
     * @return the timestamp in seconds.
     */
    public double getTimestamp() {
        return ts;
    }

    /**
     * Fetches the maximum CAN timestamp out of an array of FrameData objects
     * @param dataArray array value likely begotten from {@link Frame#waitForFrames}
     * @return the max timestamp value
     */
    public static double maxTimestamp(FrameData<?>[] dataArray) {
        double max = 0;
        for (FrameData<?> fd: dataArray) {
            max = Math.max(max, fd.getTimestamp());
        }
        return max;

    }
}
