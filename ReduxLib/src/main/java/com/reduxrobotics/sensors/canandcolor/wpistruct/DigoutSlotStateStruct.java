// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandcolor.CanandcolorDetails;
import com.reduxrobotics.sensors.canandcolor.DigoutChannel;
import com.reduxrobotics.sensors.canandcolor.DigoutSlotState;

import edu.wpi.first.util.struct.Struct;

/** WPILib struct implementation for {@link DigoutSlotState} */
public class DigoutSlotStateStruct implements Struct<DigoutSlotState> {

    @Override
    public Class<DigoutSlotState> getTypeClass() {
        return DigoutSlotState.class;
    }

    @Override
    public String getTypeName() {
        return "DigoutSlotState";
    }

    @Override
    public int getSize() {
        return 8;
    }

    @Override
    public String getSchema() {
        return (
            "bool digout1_state;" +
            "bool digout2_state;" +
            "bool digout1_sticky;" +
            "bool digout2_sticky;" +
            "uint16 digout1_cond;" +
            "uint16 digout2_cond;"
        );
    }

    @Override
    public DigoutSlotState unpack(ByteBuffer bb) {
        return new DigoutSlotState(CanandcolorDetails.Msg.constructDigitalOutput(
            bb.get() != 0,
            bb.get() != 0,
            bb.get() != 0,
            bb.get() != 0,
            (int) bb.getShort() & 0xffff,
            (int) bb.getShort() & 0xffff
        ));
    }

    @Override
    public void pack(ByteBuffer bb, DigoutSlotState value) {
        bb.put((byte) (value.getDigoutChannelValue(DigoutChannel.Index.kDigout1) ? 1 : 0));
        bb.put((byte) (value.getDigoutChannelValue(DigoutChannel.Index.kDigout2) ? 1 : 0));
        bb.put((byte) (value.getStickyDigoutValue(DigoutChannel.Index.kDigout1) ? 1 : 0));
        bb.put((byte) (value.getStickyDigoutValue(DigoutChannel.Index.kDigout2) ? 1 : 0));
        bb.putShort((short) value.getDigoutSlotBitfield(DigoutChannel.Index.kDigout1));
        bb.putShort((short) value.getDigoutSlotBitfield(DigoutChannel.Index.kDigout2));
    }
    
}
