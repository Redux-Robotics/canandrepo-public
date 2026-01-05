// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.frames;

import java.util.Set;

import com.reduxrobotics.canand.CanandUtils;

import java.util.HashSet;

/**
 * Class representing periodic timestamped data received from CAN or other sources.
 * 
 * <p>
 * For applications like latency compensation, we often need both sensor/device data and a timestamp
 * of when the data was received. 
 * 
 * Frames provide this by holding the most recently received raw data
 * and the timestamps they were received at and allowing retrieval of both data and timestamps
 * in one {@link FrameData} object via {@link Frame#getFrameData()}, avoiding race conditions 
 * involving reading data and timestamp separately.
 * </p>
 * <p> Additionally, they allow for:
 * <ul>
 * <li> synchronous reads through {@link Frame#waitForFrames(double, Frame...)} by notifying when 
 * new data has been received. </li>
 * <li> asynchronous callbacks through {@link Frame#addCallback(FrameCallback)} that give 
 * {@link FrameData} directly to callback functions. </li>
 * </ul>
 * 
 * <p>
 * In Java, non-primitive data types cannot be easily passed by value -- instead the language 
 * prefers immutable record type objects for compound data structures.
 * 
 * However, in an FRC application, this would mean a ton of objects getting thrown to the garbage 
 * collector from every time a Frame gets updated rendering the previous value object stale. 
 * 
 * To get around this, implementing subclasses such as {@link DoubleFrame}, {@link LongFrame}, and 
 * {@link ByteArrayFrame} on update copy the new value to a primitive field (or in the case of 
 * ByteArrayFrame, an array of primitives).
 * 
 * As primitives in Java have value semantics, this avoids instantiation of new objects on every 
 * update, and the final object value is only instantiated on a {@link #getValue()} call via a 
 * constructor-supplied raw-to-finished data conversion function. 
 * 
 * Often, the conversion function is simply the constructor of the final object.
 * </p>
 * @param <T>the type of object the frame holds.
 */
public abstract class Frame<T> {

    private static class FrameListener {
        Frame<?>[] frames;
        FrameData<?>[] data;
        public FrameListener(Frame<?>[] frames) {
            this.frames = frames;
            this.data = new FrameData<?>[frames.length];
        }

        public synchronized void updateValue(Frame<?> frame) {
            for (int i = 0; i < frames.length; i++) {
                if (frame == frames[i]) {
                    data[i] = frame.getFrameData();
                }
            }
        }

        public synchronized void unregisterAll() {
            for (Frame<?> frame: frames) {
                frame.removeListener(this);
            }
        }

        public synchronized FrameData<?>[] getData() {
            return this.data;
        }

        public synchronized boolean hasAllData() {
            for (FrameData<?> fd: data) {
                if (fd == null) return false;
            }
            return true;
        }
    }

    /**
     * Functional interface for Frame callbacks.
     * @param <FC> the type of the frame's contained value.
     */
    public static interface FrameCallback<FC> {
        /**
         * The callback called.
         * @param frame The Frame called.
         */
        public void callback(Frame<FC> frame);

    }

    private double ts; 
    private Set<FrameListener> listeners;
    private Set<FrameCallback<T>> callbacks;

    /**
     * Constructs a new Frame object.
     * 
     * @param timestamp The initial timestamp at which the value was received in seconds.
     */
    public Frame(double timestamp) {
        this.ts = timestamp;
        this.listeners = new HashSet<>();
        this.callbacks = new HashSet<>();
    }

    /**
     * Updates the Frame's value, notifying any listeners of new data.
     * 
     * Implementing classes should call this function in a synch block in their own update methods.
     * 
     * @param timestamp the new timestamp of the received data, in seconds
     */
    protected synchronized void update(double timestamp) {
        this.ts = timestamp;
        if (callbacks.size() > 0) {
            for (FrameCallback<T> cb : callbacks) {
                cb.callback(this);
            }
        }
        for (FrameListener listener: listeners) {
            synchronized(listener) {
                listener.updateValue(this);
                listener.notifyAll();
            }
        }
    }

    /**
     * Add a callback that will be run whenever this Frame gets updated.
     * Example application:
     * <pre class="include-com_reduxrobotics_sensors_canandmag_Canandmag include-java_util_List include-java_util_ArrayList">
     * // Log Canandmag position FrameData.
     * List&lt;FrameData&lt;Double&gt;&gt; data = new ArrayList&lt;&gt;();
     * Canandmag enc0 = new Canandmag(0);
     * 
     * enc0.getPositionFrame().addCallback(frame -> {
     *     data.add(frame.getFrameData());
     * });
     * // Timestamped data is now streamed into the List.
     * 
     * </pre>
     * 
     * @param callback the callback function, taking a FrameData
     * @return true on success, false if the callback has already been added.
     */
    public synchronized boolean addCallback(FrameCallback<T> callback) {
        return callbacks.add(callback);
    }

    /**
     * Remove a registered callback by the handle.
     * @param callback the callback function, taking a FrameData
     * @return true on success, false if the callback has already been added.
     */
    public synchronized boolean removeCallback(FrameCallback<T> callback) {
        return callbacks.remove(callback);
    }

    synchronized boolean addListener(FrameListener listener) {
        return listeners.add(listener);
    }

    synchronized boolean removeListener(FrameListener listener) {
        return listeners.remove(listener);
    }

    /**
     * Returns an immutable FrameData&lt;T&gt; class containing both value and timestamp.
     * 
     * This bypasses the race condition possible via calling {@link #getValue} followed by 
     * {@link #getTimestamp} individually.
     * @return frame data
     */
    public synchronized FrameData<T> getFrameData() {
        return new FrameData<T>(getValue(), ts);
    }

    /**
     * Returns the value of the data frame.
     * @return the value the data frame holds.
     */
    public abstract T getValue();

    /**
     * Returns if this frame has data.
     * @return if this frame's data can be considered valid
     */
    public abstract boolean hasData();

    /**
     * Gets the timestamp in seconds of when this value was updated.
     * The time base is relative to the FPGA timestamp.
     * @return the timestamp in seconds.
     */
    public synchronized double getTimestamp() {
        return ts;
    }

    /**
     * Waits for all Frames to have transmitted a value. 
     * Either returns an array of {@link FrameData} representing the data from corresponding frames 
     * passed in (in the order they are passed in) or null if timeout or interrupt is hit.
     * <pre class="include-com_reduxrobotics_sensors_canandmag_Canandmag">
     * // Keep in mind this code sample will likely cause timing overruns 
     * // if on the main thread of your robot code.
     * 
     * // some definitions for reference:
     * Canandmag enc = new Canandmag(0);
     * Canandmag enc1 = new Canandmag(1);
     * 
     * // in your side thread function:
     * 
     * // wait for up to 40 ms for position and velocity to come in from two Canandmags
     * FrameData&lt;?&gt;[] data = Frame.waitForFrames(0.040, enc.getPositionFrame(), enc.getVelocityFrame(), enc1.getPositionFrame());
     * if (data == null) {
     *   System.out.printf("waitForFrames timed out before receiving all data\n");
     * } else {
     *   // blind casts are needed to pull the data out of the array
     *   FrameData&lt;Double&gt; posFrame = (FrameData&lt;Double&gt;) data[0];
     *   FrameData&lt;Double&gt; velFrame = (FrameData&lt;Double&gt;) data[1];
     *   FrameData&lt;Double&gt; posFram1 = (FrameData&lt;Double&gt;) data[2];
     *
     *   // fetches the maximum timestamp across all received timestamps (the "latest" value)
     *   double latest = Frame.maxTimestamp(data);
     *
     *   // prints the received frame value and how far behind the latest received CAN timestamp it was
     *   System.out.printf("posFrame: %.3f, %.3f ms\n", posFrame.getValue(), (latest - posFrame.getTimestamp()) * 1000);
     *   System.out.printf("velFrame: %.3f, %.3f ms\n", velFrame.getValue(), (latest - velFrame.getTimestamp()) * 1000);
     *   System.out.printf("posFram1: %.3f, %.3f ms\n", posFram1.getValue(), (latest - posFram1.getTimestamp()) * 1000);
     * } 
     * </pre>
     * 
     * @param timeout maximum seconds to wait for before giving up
     * @param frames {@link Frame} handles to wait on. Position in argument list corresponds to 
     *     position in the returned FrameData array.
     * @return an array of {@link FrameData}; representing the data from  corresponding frames 
     *     passed in or null if timeout or interrupt is hit.
     */
    public static FrameData<?>[] waitForFrames(double timeout, Frame<?>... frames) {
        Frame.FrameListener listener = new FrameListener(frames);
        for (Frame<?> frame: frames) {
            frame.addListener(listener);
        }
        try {
            double before = CanandUtils.getFPGATimestamp();
            synchronized(listener) {
                while (!listener.hasAllData()) {
                    // timeout check
                    if (CanandUtils.getFPGATimestamp() - before > timeout) {
                        listener.unregisterAll();
                        return null;
                    }
                    listener.wait((long) (timeout * 1000));
                }
            }
        } catch (InterruptedException e) { 
            listener.unregisterAll();
            return null; 
        }
        listener.unregisterAll();
        return listener.getData();
    }

    /**
     * Returns the max timestamp from a tuple of {@link FrameData} objects.
     * Most useful for getting the "latest" CAN timestamp from a result of {@link #waitForFrames}.
     * @param data frame data array from {@link #waitForFrames}
     * @return the maximum timestamp
    */
    public static double maxTimestamp(FrameData<?>[] data) {
        double v = 0;
        for (FrameData<?> frame: data) {
            v = Math.max(v, frame.getTimestamp());
        }
        return v;
    }
}
