// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandcolor.CanandcolorFaults;
import com.reduxrobotics.sensors.canandcolor.CanandcolorStatus;

import edu.wpi.first.util.struct.Struct;

/** WPILib struct implementation for {@link CanandcolorStatus} */
public class CanandcolorStatusStruct implements Struct<CanandcolorStatus> {

    @Override
    public Class<CanandcolorStatus> getTypeClass() {
        return CanandcolorStatus.class;
    }

    @Override
    public String getTypeName() {
        return "CanandcolorStatus";
    }

    @Override
    public int getSize() {
        return 2 + kSizeDouble;
    }

    @Override
    public String getSchema() {
        return (
            "CanandcolorFaults active_faults;" +
            "CanandcolorFaults sticky_faults;" +
            "double temperature;"
        );
    }

    @Override
    public Struct<?>[] getNested() {
        return new Struct<?>[] {CanandcolorFaults.struct};
    }

    @Override
    public CanandcolorStatus unpack(ByteBuffer bb) {
        int faults = ((int) bb.getShort()) & 0xffff;
        double temperature = bb.getDouble();

        return new CanandcolorStatus(
            new CanandcolorFaults(faults & 0xff, true),
            new CanandcolorFaults(faults >> 8, true), 
            temperature
        );
    }

    @Override
    public void pack(ByteBuffer bb, CanandcolorStatus value) {
        int faults = (
            value.activeFaults().faultBitField() |
            (value.stickyFaults().faultBitField() << 8)
        );
        bb.putShort((short) faults);
        bb.putDouble(value.temperature());
    }
    
}
