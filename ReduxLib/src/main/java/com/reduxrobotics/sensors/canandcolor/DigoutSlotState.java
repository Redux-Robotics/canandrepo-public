// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import com.reduxrobotics.sensors.canandcolor.wpistruct.DigoutSlotStateStruct;

import edu.wpi.first.util.struct.StructSerializable;

/**
 * Class representing the state of the Canandcolor digout channels and slots.
 * <p>
 * This includes the overall value of both {@link DigoutChannel channels} and their 
 * sticky values, as well as the states of all 32 individual digout logic slots 
 * (mostly available for debugging purposes).
 * </p>
 * <p>
 * See {@link DigoutChannel} on how to configure digout logic.
 * </p>
 * 
 */
public class DigoutSlotState implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final DigoutSlotStateStruct struct = new DigoutSlotStateStruct();
    private long field;
    /**
     * Constructor -- used by the {@link Canandcolor} class to populate this object.
     * @param field Digital output bitset from the CAN message
     */
    public DigoutSlotState(long field) {
        this.field = field;
    }

    /**
     * Instatiate with blank (all zeros) digout state.
     */
    public DigoutSlotState() {
        this.field = 0;
    }

    /**
     * Gets the output value of either digital output channel, which corresponds to what the Canandcolor is outputting on those pins (assuming digout slot mode rather than duty cycle)
     * Note that this ignores normally open/normally connected, so even if the board output is high electrically, the digital output state may be false.
     * @param digout The digital output whose value is to be returned
     * @return the boolean state of that digital output
     */
    public boolean getDigoutChannelValue(DigoutChannel.Index digout) {
        if (digout == DigoutChannel.Index.kDigout1) {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout1State(field);
        } else {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout2State(field);
        }
    }

    /**
     * Gets the sticky flag of either digital output channel, which corresponds to whether the Canandcolor has flagged the digout for ever being true.
     * <p>Note that this ignores normally open/normally connected, so even if the board output is high electrically, the digital output state may be false.</p>
     * <p>These sticky flags can be cleared with {@link Canandcolor#clearStickyDigoutFlags} </p>
     * @param digout The digital output whose value is to be returned
     * @return the boolean state of that digital output
     */
    public boolean getStickyDigoutValue(DigoutChannel.Index digout) {
        if (digout == DigoutChannel.Index.kDigout1) {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout1Sticky(field);
        } else {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout2Sticky(field);
        }
    }


    /**
     * Gets the boolean value of a specific digout slot contributing to a digital output.
     * 
     * @param digout the digital output associated with the slot
     * @param slot the slot index
     * @return Whether or not that specific slot is returning true or not.
     */
    public boolean getDigoutSlotValue(DigoutChannel.Index digout, int slot) {
        if (slot < 0 || slot > 15) throw new IllegalArgumentException("Condition slot index must be between 0-15 inclusive!");
        // this is the byte of the digout field
        int subfield = ((digout == DigoutChannel.Index.kDigout1) 
            ? CanandcolorDetails.Msg.extractDigitalOutput_Digout1Cond(field) 
            : CanandcolorDetails.Msg.extractDigitalOutput_Digout2Cond(field)
        );
        return (subfield & (1 << slot)) > 0;
    }

    /**
     * Gets a little endian 16-bit bitfield of individual digout slot values.
     * @param digout the digital output associated with the slots
     * @return slot bitfield
     */
    public int getDigoutSlotBitfield(DigoutChannel.Index digout) {
        if (digout == DigoutChannel.Index.kDigout1) {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout1Cond(field);
        } else {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout2Cond(field);
        }
    }

}
