// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro;

import com.reduxrobotics.frames.Frame;

import edu.wpi.first.math.geometry.Quaternion;

/**
 * Implements an object-holding Frame with considerations for quaternions.
 * 
 * This avoids creation of new objects by only converting to a {@link Quaternion} when the value is requested,
 * while also offering interfaces to efficiently retrieve components without allocation.
 */
public class QuaternionFrame extends Frame<Quaternion> {
    private final static double TAU = Math.PI * 2;

    private double w;
    private double x;
    private double y;
    private double z;
    private Quaternion defaultData;
    private Quaternion cache;
    private boolean dataValid;

    /**
     * Instantiates a new QuaternionFrame.
     * 
     * @param timestamp The timestamp the update happened at.
     * @param defaultData An instance of the object to return before the first update happens.
     */
    public QuaternionFrame(double timestamp, Quaternion defaultData) {
        super(timestamp);
        this.defaultData = defaultData;
        this.dataValid = false;
        this.cache = null;
    }

    @Override
    public synchronized Quaternion getValue() {
        if (!dataValid) return defaultData;
        if (cache == null) cache = new Quaternion(w, x, y, z);
        return cache;
    }

    /**
     * Gets the W term of the current quaternion, normalized from [-1.0..1.0] inclusive.
     * @return quaternion term value
     */
    public synchronized double getW() {
        return w;
    }
    
    /**
     * Gets the X term of the current quaternion, normalized from [-1.0..1.0] inclusive.
     * @return quaternion term value
     */
    public synchronized double getX() {
        return x;
    }

    /**
     * Gets the Y term of the current quaternion, normalized from [-1.0..1.0] inclusive.
     * @return quaternion term value
     */
    public synchronized double getY() {
        return y;
    }

    /**
     * Gets the Z term of the current quaternion, normalized from [-1.0..1.0] inclusive.
     * @return quaternion term value
     */
    public synchronized double getZ() {
        return z;
    }

    // these implementations are borrowed from WPILib's Rotation3d class.
    // range [-0.5..0.5) rotations

    /**
     * Gets the Euler roll value <b>in rotations</b> from [-0.5 inclusive..0.5 exclusive).
     * @return roll value in rotations
     */
    public synchronized double getRoll() {
        final var cxcy = 1.0 - 2.0 * (x * x + y * y);
        final var sxcy = 2.0 * (w * x + y * z);
        final var cy_sq = cxcy * cxcy + sxcy * sxcy;
        if (cy_sq > 1e-20) {
            return Math.atan2(sxcy, cxcy) / TAU;
        }
        return 0.0;
    }

    /**
     * Gets the Euler pitch value <b>in rotations</b> from [-0.5 inclusive..0.5 exclusive).
     * @return pitch value in rotations
     */
    public synchronized double getPitch() {
        double ratio = 2.0 * (w * y - z * x);
        if (Math.abs(ratio) >= 1.0) {
            return Math.copySign(Math.PI / 2.0, ratio) / TAU;
        }
        return Math.asin(ratio) / TAU;
    }

    /**
     * Gets the Euler yaw value <b>in rotations</b> from [-0.5 inclusive..0.5 exclusive).
     * @return yaw value in rotations
     */
    public synchronized double getYaw() {
        final var cycz = 1.0 - 2.0 * (y * y + z * z);
        final var cysz = 2.0 * (w * z + x * y);
        final var cy_sq = cycz * cycz + cysz * cysz;
        if (cy_sq > 1e-20) {
            return Math.atan2(cysz, cycz) / TAU;
        }
        return Math.atan2(2.0 * w * z, w * w - z * z) / TAU;
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
        w = ((short)((data[0] & 0xFF) | ((data[1] & 0xFF) << 8))) / 32767.0;
        x = ((short)((data[2] & 0xFF) | ((data[3] & 0xFF) << 8))) / 32767.0;
        y = ((short)((data[4] & 0xFF) | ((data[5] & 0xFF) << 8))) / 32767.0;
        z = ((short)((data[6] & 0xFF) | ((data[7] & 0xFF) << 8))) / 32767.0;

        // internally (re)normalize the vector.
        double norm = Math.sqrt(w*w + x*x + y*y + z*z);
        if (norm == 0.0) {
            w = 1;
            x = y = z = 0;
        } else {
            w /= norm;
            x /= norm;
            y /= norm;
            z /= norm;
        }

        this.dataValid = true;
        this.cache = null;
        update(timestamp);
    }
}
