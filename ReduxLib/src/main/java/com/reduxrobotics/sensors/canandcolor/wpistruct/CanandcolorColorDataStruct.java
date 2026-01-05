// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor.wpistruct;

import java.nio.ByteBuffer;

import com.reduxrobotics.sensors.canandcolor.ColorData;

import edu.wpi.first.util.struct.Struct;

/** WPILib struct implementation for {@link ColorData} */
public class CanandcolorColorDataStruct implements Struct<ColorData> {

    @Override
    public Class<ColorData> getTypeClass() {
        return ColorData.class;
    }

    @Override
    public String getTypeName() {
        return "CanandcolorColorData";
    }

    @Override
    public int getSize() {
        return 8 * 3;
    }

    @Override
    public String getSchema() {
        return "double red;double green;double blue;";
    }

    @Override
    public ColorData unpack(ByteBuffer bb) {
        return new ColorData(bb.getDouble(), bb.getDouble(), bb.getDouble());
    }

    @Override
    public void pack(ByteBuffer bb, ColorData value) {
        bb.putDouble(value.red());
        bb.putDouble(value.green());
        bb.putDouble(value.blue());
    }
    
}
