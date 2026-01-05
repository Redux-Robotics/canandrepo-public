package com.reduxrobotics.canand;

/**
 * A class representing a message repeater.
 * 
 * Message repeaters live in the Redux driver and repeat messages for some fixed period for some fixed number of repetitions until
 * their configuration gets updated.
 */
public class Repeater {
    private long handle;
    private boolean active;

    /**
     * Allocates a new message repeater.
     */
    public Repeater() {
        this.handle = ReduxJNI.newRepeater();
        this.active = true;
    }

    /**
     * Updates the message repeater.
     * This will also immediately send out the updated message.
     * 
     * @param busId the bus ID to send to
     * @param messageId the message ID to send to
     * @param data the data as a 64-bit long
     * @param length the length code
     * @param periodMs the period to send in milliseconds
     * @param times the number of times to repeat the message
     */
    public synchronized void update(int busId, int messageId, long data, int length, int periodMs, int times) {
        if (active) {
            ReduxJNI.updateRepeater(handle, busId, messageId, data, length, periodMs, times);
        }
    }

    /**
     * Permanently disarms the {@link Repeater}. 
     */
    public synchronized void disarm() {
        active = false;
        ReduxJNI.deallocateRepeater(handle);
    }
}
