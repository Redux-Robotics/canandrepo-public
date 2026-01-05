// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandgyro.CanandgyroFaults;

import edu.wpi.first.util.struct.Struct;

/** WPILib struct implementation for {@link CanandgyroFaults} */
public class CanandgyroFaultsStruct implements Struct<CanandgyroFaults> {

    @Override
    public Class<CanandgyroFaults> getTypeClass() {
        return CanandgyroFaults.class;
    }

    @Override
    public String getTypeName() {
        return "CanandgyroFaults";
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
            "bool calibrating:1;" +
            "bool angular_velocity_saturation:1;" +
            "bool acceleration_saturation:1;"
        );
    }

    @Override
    public CanandgyroFaults unpack(ByteBuffer bb) {
        int faultField = (int) bb.get() & 0xff;
        return new CanandgyroFaults(faultField, true);
    }

    @Override
    public void pack(ByteBuffer bb, CanandgyroFaults value) {
        bb.put((byte) value.faultBitField());
    }

}
