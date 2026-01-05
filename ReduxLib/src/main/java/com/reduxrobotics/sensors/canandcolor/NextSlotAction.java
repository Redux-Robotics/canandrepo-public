// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Low-level enum specifying how a digout slot should interact with the following slot.
 * 
 * Digout slots may either join with the next index numbered slot (e.g. slot 0 to slot 1) with a boolean operation to connect them in a chain,
 * or terminate the current chain.
 * All chains of digout slots must evaluate to true for the digital output to be true.
 * @see DigoutSlot
 */
enum NextSlotAction {
    /** 
     * Do not join this slot with the next slot -- rather, terminate the clause here. 
     * If no previous slots are joining to this slot, then we have a single-slot "singleton" clause.
     */
    kTerminateChain(CanandcolorDetails.Enums.NextSlotAction.kTerminateChain),
    /** Logical OR this slot's value with the next slot's value  */
    kOrWithNextSlot(CanandcolorDetails.Enums.NextSlotAction.kOrWithNextSlot), 
    /** Logical XOR this slot's value with the next slot's value  */
    kXorWithNextSlot(CanandcolorDetails.Enums.NextSlotAction.kXorWithNextSlot),
    /** Logical AND this slot's value with the next slot's value  */
    kAndWithNextSlot(CanandcolorDetails.Enums.NextSlotAction.kAndWithNextSlot);

    private int index;
    NextSlotAction(int index) { this.index = index; }

    /**
     * Fetches the index associated with the enum.
     * @return index value
     */
    public int getIndex() { return index; }
    private static final NextSlotAction map[] = {kTerminateChain, kOrWithNextSlot, kXorWithNextSlot, kAndWithNextSlot};

    /**
     * Fetches the enum associated with the index value.
     * @param index int containing index value; only lower 2 bits are used
     * @return corresponding enum
     */
    public static NextSlotAction fromIndex(int index) {
        return map[index & 0b11];
    }
}