// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandgyro.CanandgyroFaults;
import com.reduxrobotics.sensors.canandgyro.CanandgyroStatus;

import edu.wpi.first.util.struct.Struct;

/**
 * WPILib struct implementation for {@link CanandgyroStatus}
 */
public class CanandgyroStatusStruct implements Struct<CanandgyroStatus> {

    @Override
    public Class<CanandgyroStatus> getTypeClass() {
        return CanandgyroStatus.class;
    }

    @Override
    public String getTypeName() {
        return "CanandgyroStatus";
    }

    @Override
    public int getSize() {
        return 2 + kSizeDouble;
    }

    @Override
    public String getSchema() {
        return (
            "CanandgyroFaults active_faults;" +
            "CanandgyroFaults sticky_faults;" +
            "double temperature;"
        );
    }

    @Override
    public Struct<?>[] getNested() {
        return new Struct<?>[] {CanandgyroFaults.struct};
    }

    @Override
    public CanandgyroStatus unpack(ByteBuffer bb) {
        int faults = ((int) bb.getShort()) & 0xffff;
        double temperature = bb.getDouble();

        return new CanandgyroStatus(
            true, 
            new CanandgyroFaults(faults & 0xff, true),
            new CanandgyroFaults(faults >> 8, true), 
            temperature
        );
    }

    @Override
    public void pack(ByteBuffer bb, CanandgyroStatus value) {
        int faults = (
            value.activeFaults().faultBitField() |
            (value.stickyFaults().faultBitField() << 8)
        );
        bb.putShort((short) faults);
        bb.putDouble(value.temperature());
    }
}
