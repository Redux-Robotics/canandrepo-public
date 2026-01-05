// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Low-level enum corresponding to digout slot opcodes.
 * <p>For more information see {@link DigoutSlot}</p>
 */
enum DigoutOperation {

    /** LHS equals RHS */
    kEquals(CanandcolorDetails.Enums.SlotOpcode.kEquals),
    /** LHS less than RHS */
    kLessThan(CanandcolorDetails.Enums.SlotOpcode.kLessThan),
    /** LHS greater than RHS */
    kGreaterThan(CanandcolorDetails.Enums.SlotOpcode.kGreaterThan),
    /** LHS less than or equals RHS */
    kLessThanOrEquals(CanandcolorDetails.Enums.SlotOpcode.kLessThanOrEquals),
    /** LHS greater than or equals RHS */
    kGreaterThanOrEquals(CanandcolorDetails.Enums.SlotOpcode.kGreaterThanOrEquals),
    /** previous slot true for RHS milliseconds */
    kPrevSlotTrue(CanandcolorDetails.Enums.SlotOpcode.kPrevSlotTrue),
    /** previous slot chain true for RHS milliseconds */
    kPrevSlotChainTrue(CanandcolorDetails.Enums.SlotOpcode.kPrevClauseTrue) ;
    private int index;
    DigoutOperation(int index) { this.index = index; } 

    /**
     * Gets the corresponding index for the value in question.
     * @return the index for the opcode (used in serialization)
     */
    public int getIndex() { return index; }

    /**
     * Returns a corresponding opcode from the given index.
     * @param idx the index to fetch.
     * @return a valid opcode. If invalid, returns equals immidiate.
     */
    public static DigoutOperation fromIndex(int idx) {
        return switch (idx) {
            case CanandcolorDetails.Enums.SlotOpcode.kEquals -> kEquals;
            case CanandcolorDetails.Enums.SlotOpcode.kLessThan -> kLessThan;
            case CanandcolorDetails.Enums.SlotOpcode.kGreaterThan -> kGreaterThan;
            case CanandcolorDetails.Enums.SlotOpcode.kLessThanOrEquals -> kLessThanOrEquals;
            case CanandcolorDetails.Enums.SlotOpcode.kGreaterThanOrEquals -> kGreaterThanOrEquals;
            case CanandcolorDetails.Enums.SlotOpcode.kPrevSlotTrue -> kPrevSlotTrue;
            case CanandcolorDetails.Enums.SlotOpcode.kPrevClauseTrue -> kPrevSlotChainTrue;
            default -> kEquals;
        };
    }

}