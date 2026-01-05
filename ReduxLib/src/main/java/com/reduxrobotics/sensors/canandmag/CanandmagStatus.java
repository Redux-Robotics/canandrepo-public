// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag;

import com.reduxrobotics.sensors.canandmag.wpistruct.CanandmagStatusStruct;

import edu.wpi.first.util.struct.StructSerializable;

/**
 * Container record class representing a {@link Canandmag}'s status.
 * 
 * @param activeFaults the active faults of the encoder.
 * @param stickyFaults the sticky faults of the encoder.
 * @param temperature the temperature of the encoder's processor, in deg Celsius.
 * @param magnetInRange whether the encoder's magnet is in range
 */
public record CanandmagStatus(CanandmagFaults activeFaults, CanandmagFaults stickyFaults, 
    double temperature, boolean magnetInRange) implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final CanandmagStatusStruct struct = new CanandmagStatusStruct();

    /**
     * Return an invalid Status.
     * @return an invalid/default status
     */
    public static CanandmagStatus invalid() {
        return new CanandmagStatus(new CanandmagFaults((byte) 0, false), 
            new CanandmagFaults((byte) 0, false), 0, false);
    }

    /**
     * Deserializes a Status object from a byte array.
     * @param v byte array to deserialize from
     * @return new Status
     */
    public static CanandmagStatus fromByteArray(byte[] v) {
        return new CanandmagStatus( 
            new CanandmagFaults(v[0], true),  // active faults
            new CanandmagFaults(v[1], true),  // sticky faults
            v[2],                         // temp
            (v[0] & 0b100000) == 0);     // magnet status
    }
}