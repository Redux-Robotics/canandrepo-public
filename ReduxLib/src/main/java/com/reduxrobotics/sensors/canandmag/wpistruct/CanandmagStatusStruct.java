// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandmag.CanandmagFaults;
import com.reduxrobotics.sensors.canandmag.CanandmagStatus;

import edu.wpi.first.util.struct.Struct;

/**
 * WPILib struct implementation for {@link CanandmagStatus}
 */
public class CanandmagStatusStruct implements Struct<CanandmagStatus> {

    @Override
    public Class<CanandmagStatus> getTypeClass() {
        return CanandmagStatus.class;
    }

    @Override
    public String getTypeName() {
        return "CanandmagStatus";
    }

    @Override
    public int getSize() {
        return 2 + kSizeDouble;
    }

    @Override
    public String getSchema() {
        return (
            "CanandmagFaults active_faults;" +
            "CanandmagFaults sticky_faults;" +
            "double temperature;"
        );
    }

    @Override
    public Struct<?>[] getNested() {
        return new Struct<?>[] {CanandmagFaults.struct};
    }

    @Override
    public CanandmagStatus unpack(ByteBuffer bb) {
        int faults = ((int) bb.getShort()) & 0xffff;
        double temperature = bb.getDouble();
        var activeFaults = new CanandmagFaults(faults & 0xff, true);
        var stickyFaults = new CanandmagFaults(faults >> 8, true);

        return new CanandmagStatus(
            activeFaults, 
            stickyFaults,
            temperature,
            !activeFaults.magnetOutOfRange()
        );
    }

    @Override
    public void pack(ByteBuffer bb, CanandmagStatus value) {
        int faults = (
            value.activeFaults().faultBitField() |
            (value.stickyFaults().faultBitField() << 8)
        );
        bb.putShort((short) faults);
        bb.putDouble(value.temperature());
    }
}
