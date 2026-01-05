// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Builder class for digout slot chains.
 * <p>
 * <b>Most users will probably want to look at {@link RGBDigoutConfig} or {@link HSVDigoutConfig} instead.</b>
 * </p>
 * 
 * <p>These get passed to {@link DigoutChannel#configureSlotsAdvanced(DigoutChain...)} to configure digout slots.</p>
 * <p>For more information on advanced digout slot configuration, see {@link DigoutSlot}.</p>
 */
class DigoutChain {
    DigoutSlotBuilder[] slots = new DigoutSlotBuilder[16];
    int i = 0;

    DigoutChain() {
        for (int j = 0; j < slots.length; j++) {
            slots[j] = new DigoutSlotBuilder(this);
        }
    }

    /**
     * Starts a digout slot chain.
     * <p> This is the method you want to use to configure digouts with.</p>
     * @return a builder object for the first slot in the chain.
     * @see DigoutSlot
     */
    public static DigoutSlotBuilder start() {
        DigoutChain inst = new DigoutChain();
        for (int j = 0; j < inst.slots.length; j++) {
            inst.slots[j] = new DigoutSlotBuilder(inst);
        }
        inst.i = 0;
        return inst.slots[0];
    }

    DigoutSlotBuilder increment() {
        i += 1;
        return slots[i];
    }

    DigoutChain finish() {
        if (i >= slots.length) {
            throw new IllegalArgumentException("digout chains must be 16 slots or less");
        }
        i += 1;
        return this;
    }

    /**
     * Gets the length of the digout chain.
     * @return digout chain length.
     */
    public int length() {
        return i;
    }

    /**
     * Fetches the {@link DigoutSlot} configuration at the current index of the chain.
     * @param index index to fetch from
     * @return digout slot or exception if out of bounds.
     */
    public DigoutSlot getSlot(int index) {
        if (index > length()) {
            throw new IllegalArgumentException(
                String.format("slot index %d out of bounds for chain of length %d", index, length())
            );
        }
        return slots[index].slot;
    }

}
