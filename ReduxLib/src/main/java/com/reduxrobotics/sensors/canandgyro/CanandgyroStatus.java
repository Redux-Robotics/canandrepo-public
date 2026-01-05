// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro;

import com.reduxrobotics.sensors.canandgyro.wpistruct.CanandgyroStatusStruct;

import edu.wpi.first.util.struct.StructSerializable;

/**
 * Container record class representing a {@link Canandgyro}'s status.
 * 
 * @param valid true if this status object has valid data.
 * @param activeFaults the active faults of the device.
 * @param stickyFaults the sticky faults of the device.
 * @param temperature the temperature of the device's processor, in deg Celsius.
 */
public record CanandgyroStatus(boolean valid, CanandgyroFaults activeFaults, CanandgyroFaults stickyFaults, double temperature) implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final CanandgyroStatusStruct struct = new CanandgyroStatusStruct();

    /**
     * Returns a default Status record for frame initializion
     * @return invalid Status record
     */
    public static CanandgyroStatus invalid() {
        return new CanandgyroStatus(false, new CanandgyroFaults(0, false), new CanandgyroFaults(0, false), 0);
    }

    /**
     * Deserializes a Status object from a byte array.
     * @param v byte array to deserialize from
     * @return new Status
     */
    public static CanandgyroStatus fromByteArray(byte[] v) {
        return new CanandgyroStatus( 
            true,
            new CanandgyroFaults((int) v[0] & 0xff, true),  // active faults
            new CanandgyroFaults((int) v[1] & 0xff, true),  // sticky faults
            (double) (((int) v[3]) << 8 | (((int) v[2]) & 0xff)) / 256.0); // temp
    }

    /**
     * Deserializes a Status object from a long.
     * @param v long data
     * @return new Status
     */
    public static CanandgyroStatus fromLong(long v) {
        return new CanandgyroStatus(true,
            new CanandgyroFaults(CanandgyroDetails.Msg.extractStatus_Faults(v), true),
            new CanandgyroFaults(CanandgyroDetails.Msg.extractStatus_StickyFaults(v), true),
            ((double) CanandgyroDetails.Msg.extractStatus_Temperature(v)) / 256.0
        );
    }
}