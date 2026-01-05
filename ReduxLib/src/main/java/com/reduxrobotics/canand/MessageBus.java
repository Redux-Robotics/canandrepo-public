// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

/**
 * Class representing CAN buses that may exist on a robot.
 * 
 * <b>Currently, only the Rio's onboard CAN bus is supported.</b>
 * 
 */
public class MessageBus {
    private int fd;

    private static MessageBus rioBus = null;
    MessageBus(int busDescriptor) {
        fd = busDescriptor;
    }

    /**
     * Returns an object representing the roboRIO's onboard bus.
     * @return an object representing the Rio's bus (descriptor = 0)
     */
    public static synchronized MessageBus getRioBus() {
        if (rioBus == null) {
            rioBus = new MessageBus(ReduxJNI.openBusById(0));
        }
        return rioBus;
    } 

    /**
     * Returns an object representing the specified SystemCore bus index.
     * Do not use this on a roboRIO.
     * 
     * @param idx bus index.
     * @return object
     */
    public static MessageBus getSystemCoreBus(int idx) {
        if (idx > 0 && idx < 5) {
            return new MessageBus(idx);
        } else {
            throw new IllegalArgumentException("bus index must be between 0 and 4 inclusive");
        }
    }

    /**
     * Returns an object corresponding to a specific bus descriptor value.
     * 
     * It is generally preferable to use this function instead of the constructor to avoid object 
     * creation churn.
     * @param busDescriptor the descriptor value
     * @return a new bus object associated with that bus descriptor. Equality can be checked between two bus objects with {@link #equals}
     */
    public static MessageBus byDescriptor(int busDescriptor) {
        if (busDescriptor == 0) {
            return getRioBus();
        }
        return new MessageBus(busDescriptor);
    }

    /**
     * Gets a MessageBus instance from a bus string.
     * @param busString bus string.
     * @return bus object
     */
    public static MessageBus byBusString(String busString) {
        return new MessageBus(ReduxJNI.openBusByString(busString));
    }

    /**
     * Returns the descriptor ID associated with the CAN bus object. 
     * Generally not needed to be used directly.
     * @return the descriptor ID.
     */
    public int getDescriptor() {
        return fd;
    }

    /**
     * Returns whether two bus objects refer to the same bus.
     * @param other other bus object to compare against
     * @return whether or not they refer to the same bus
     */
    public boolean equals(Object other) {
        return other instanceof MessageBus && getDescriptor() == ((MessageBus) other).getDescriptor();
    }


}
