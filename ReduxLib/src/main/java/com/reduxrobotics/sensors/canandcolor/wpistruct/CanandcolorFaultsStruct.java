// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandcolor.CanandcolorFaults;

import edu.wpi.first.util.struct.Struct;

/** WPILib struct implementation for {@link CanandcolorFaults} */
public class CanandcolorFaultsStruct implements Struct<CanandcolorFaults> {

    @Override
    public Class<CanandcolorFaults> getTypeClass() {
        return CanandcolorFaults.class;
    }

    @Override
    public String getTypeName() {
        return "CanandcolorFaults";
    }

    @Override
    public int getSize() {
        return 1;
    }

    @Override
    public String getSchema() {
        return (
            "bool power_cycle:1;" +
            "bool can_id_conflict:1;" +
            "bool can_general_error:1;" +
            "bool out_of_temperature_range:1;" +
            "bool hardware_fault_proximity:1;" +
            "bool hardware_fault_color:1;" +
            "bool reserved_0:1;" +
            "bool reserved_1:1;"
        );
    }

    @Override
    public CanandcolorFaults unpack(ByteBuffer bb) {
        int faultField = (int) bb.get() & 0xff;
        return new CanandcolorFaults(faultField);
    }

    @Override
    public void pack(ByteBuffer bb, CanandcolorFaults value) {
        bb.put((byte) value.faultBitField());
    }
    
}
