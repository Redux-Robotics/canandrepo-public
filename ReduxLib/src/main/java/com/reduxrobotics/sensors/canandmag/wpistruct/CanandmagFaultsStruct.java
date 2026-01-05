// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandmag.CanandmagFaults;

import edu.wpi.first.util.struct.Struct;

/** WPILib struct implementation for {@link CanandmagFaults} */
public class CanandmagFaultsStruct implements Struct<CanandmagFaults> {

    @Override
    public Class<CanandmagFaults> getTypeClass() {
        return CanandmagFaults.class;
    }

    @Override
    public String getTypeName() {
        return "CanandmagFaults";
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
            "bool hardware_fault:1;" +
            "bool magnet_out_of_range:1;" +
            "bool under_volt:1;" +
            "bool reserved:1;"
        );
    }

    @Override
    public CanandmagFaults unpack(ByteBuffer bb) {
        int faultField = (int) bb.get() & 0xff;
        return new CanandmagFaults(faultField, true);
    }

    @Override
    public void pack(ByteBuffer bb, CanandmagFaults value) {
        bb.put((byte) value.faultBitField());
    }

}
