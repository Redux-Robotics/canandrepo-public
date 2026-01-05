// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Digout slot builder class.
 * 
 * <p>To use this class, see {@link DigoutSlot} and {@link DigoutChannel#configureSlotsAdvanced}</p>
 */
class DigoutSlotBuilder {
    DigoutSlot slot;
    DigoutChain chain;

    DigoutSlotBuilder(DigoutChain chain) {
        this.slot = new DigoutSlot(
            false, 
            NextSlotAction.kTerminateChain, false, 
            DigoutOperation.kEquals, 0, 0, 
            DataSource.kZero,
            DataSource.kZero
        );
        this.chain = chain;

    }

    /**
     * Digout chain builder class.
     * <p>
     * This class is returned as the intermediary between two slots to determine 
     * if and how they should join as a chain.
     * </p>
     */
    public static class DigoutChainBuilder {
        private DigoutSlot slot;
        private DigoutChain chain;

        DigoutChainBuilder(DigoutChain chain, DigoutSlot slot) {
            this.slot = slot;
            this.chain = chain;
        }

        /**
         * If called, sets that the digout slot should invert its truth value.
         * @return the current builder.
         */
        public DigoutChainBuilder invert() {
            slot.invertValue = true;
            return this;
        }

        /**
         * If called, combines the previous digout slot with next slot's value with an AND condition.
         * @return the next slot's builder.
         */
        public DigoutSlotBuilder and() {
            slot.enabled = true;
            slot.nextSlotAction = NextSlotAction.kAndWithNextSlot;
            return chain.increment();
        }

        /**
         * If called, combines the previous digout slot with next slot's value with a OR condition.
         * @return the next slot's builder.
         */
        public DigoutSlotBuilder or() {
            slot.enabled = true;
            slot.nextSlotAction = NextSlotAction.kOrWithNextSlot;
            return chain.increment();
        }

        /**
         * If called, combines the previous digout slot with next slot's value with a XOR condition.
         * @return the next slot's builder.
         */
        public DigoutSlotBuilder xor() {
            slot.enabled = true;
            slot.nextSlotAction = NextSlotAction.kXorWithNextSlot;
            return chain.increment();
        }

        /**
         * If called, combines the previous digout slot with the next slot's value with the specified condition.
         * @param action next slot action
         * @return the next slot's builder.
         */
        public DigoutSlotBuilder join(NextSlotAction action) {
            if (action == NextSlotAction.kTerminateChain) {
                throw new IllegalArgumentException("kTerminateChain is not valid here, use .finish() instead");
            }
            slot.enabled = true;
            slot.nextSlotAction = action;
            return chain.increment();
        }

        /**
         * If called, finishes the digout chain.
         * @return finished digout chain
         */
        public DigoutChain finish() {
            slot.enabled = true;
            slot.nextSlotAction = NextSlotAction.kTerminateChain;
            return chain.finish();
        }
    }

    private void setupFixed(DigoutOperation op, DataSource source, double value) {
        slot.lhsDataSource = source;
        slot.additiveImmidiate = DigoutSlot.computeAdditiveImmidiate(value);
        slot.opcode = op;
    }

    private void setupDirect(DigoutOperation op, DataSource source, DataSource compareTo) {
        slot.lhsDataSource = source;
        slot.scalingImmidiate = 255;
        slot.rhsDataSource = compareTo;
        slot.opcode = op;
    }

    private void setupAffine(DigoutOperation op, DataSource source, double scale, double add, DataSource compareTo) {
        slot.lhsDataSource = source;
        slot.scalingImmidiate = DigoutSlot.computeMultiplicativeImmidiate(scale);
        slot.additiveImmidiate = DigoutSlot.computeAdditiveImmidiate(add);
        slot.rhsDataSource = compareTo;
        slot.opcode = op;
    }

    /**
     * Checks if the data source is equal to a fixed value in the range [0..1] inclusive.
     * @param source The data source to compare
     * @param value A fixed value in the range [0..1] inclusive to compare to
     * @return current chain builder
     */
    public DigoutChainBuilder equals(DataSource source, double value) {
        setupFixed(DigoutOperation.kEquals, source, value);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is greater than a fixed value in the range [0..1] inclusive.
     * @param source The data source to compare
     * @param value A fixed value in the range [0..1] inclusive to compare to
     * @return current chain builder
     */
    public DigoutChainBuilder greaterThan(DataSource source, double value) {
        setupFixed(DigoutOperation.kGreaterThan, source, value);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is greater than or equal to a fixed value in the range [0..1] inclusive.
     * @param source The data source to compare
     * @param value A fixed value in the range [0..1] inclusive to compare to
     * @return current chain builder
     */
    public DigoutChainBuilder greaterThanEqualTo(DataSource source, double value) {
        setupFixed(DigoutOperation.kGreaterThanOrEquals, source, value);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is less than a fixed value in the range [0..1] inclusive.
     * @param source The data source to compare
     * @param value A fixed value in the range [0..1] inclusive to compare to
     * @return current chain builder
     */
    public DigoutChainBuilder lessThan(DataSource source, double value) {
        setupFixed(DigoutOperation.kLessThan, source, value);
        return new DigoutChainBuilder(this.chain, this.slot);
    }


    /**
     * Checks if the data source is less than or equal to a fixed value in the range [0..1] inclusive.
     * @param source The data source to compare
     * @param value A fixed value in the range [0..1] inclusive to compare to
     * @return current chain builder
     */
    public DigoutChainBuilder lessThanEqualTo(DataSource source, double value) {
        setupFixed(DigoutOperation.kLessThanOrEquals, source, value);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is equal to another data source
     * @param source The data source to compare
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder equals(DataSource source, DataSource compareTo) {
        setupDirect(DigoutOperation.kEquals, source, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is greater than another data source
     * @param source The data source to compare
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder greaterThan(DataSource source, DataSource compareTo) {
        setupDirect(DigoutOperation.kGreaterThan, source, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is greater than or equal to another data source
     * @param source The data source to compare
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder greaterThanEqualTo(DataSource source, DataSource compareTo) {
        setupDirect(DigoutOperation.kGreaterThanOrEquals, source, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is less than another data source
     * @param source The data source to compare
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder lessThan(DataSource source, DataSource compareTo) {
        setupDirect(DigoutOperation.kLessThan, source, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source is less than or equal to another data source
     * @param source The data source to compare
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder lessThanEqualTo(DataSource source, DataSource compareTo) {
        setupDirect(DigoutOperation.kLessThanOrEquals, source, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }
    
    /**
     * Checks if the data source, with a scaling and offset transform, is equal to another data source.
     * 
     * Internally, this method multiplies the data source by the scale value, then adds the offset value to it.
     * 
     * @param source The data source to compare
     * @param scale The amount to scale by in the range (0 exclusive..1 inclusive]
     * @param offset The amount to offset the data source by in the range [-1..1] inclusive
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder affineEquals(DataSource source, double scale, double offset, DataSource compareTo) {
        setupAffine(DigoutOperation.kEquals, source, scale, offset, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source, with a scaling and offset transform, is greater than another data source.
     * 
     * Internally, this method multiplies the data source by the scale value, then adds the offset value to it.
     * 
     * @param source The data source to compare
     * @param scale The amount to scale by, in [0 inclusive..1 exclusive)
     * @param offset The amount to offset the data source by, in [-1,1]
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder affineGreaterThan(DataSource source, double scale, double offset, DataSource compareTo) {
        setupAffine(DigoutOperation.kGreaterThan, source, scale, offset, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source, with a scaling and offset transform, is greater than or equal to another data source.
     * 
     * Internally, this method multiplies the data source by the scale value, then adds the offset value to it.
     * 
     * @param source The data source to compare
     * @param scale The amount to scale by, in [0 inclusive..1 exclusive)
     * @param offset The amount to offset the data source by, in [-1,1]
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder affineGreaterThanEqualTo(DataSource source, double scale, double offset, DataSource compareTo) {
        setupAffine(DigoutOperation.kGreaterThanOrEquals, source, scale, offset, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source, with a scaling and offset transform, is less than another data source.
     * 
     * Internally, this method multiplies the data source by the scale value, then adds the offset value to it.
     * 
     * @param source The data source to compare
     * @param scale The amount to scale by, in [0 inclusive..1 exclusive)
     * @param offset The amount to offset the data source by, in [-1,1]
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder affineLessThan(DataSource source, double scale, double offset, DataSource compareTo) {
        setupAffine(DigoutOperation.kLessThan, source, scale, offset, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the data source, with a scaling and offset transform, is less than or equal to another data source.
     * 
     * Internally, this method multiplies the data source by the scale value, then adds the offset value to it.
     * 
     * @param source The data source to compare
     * @param scale The amount to scale by, in [0 inclusive..1 exclusive)
     * @param offset The amount to offset the data source by, in [-1,1]
     * @param compareTo The data source to compare the source to
     * @return current chain builder
     */
    public DigoutChainBuilder affineLessThanEqualTo(DataSource source, double scale, double offset, DataSource compareTo) {
        setupAffine(DigoutOperation.kLessThanOrEquals, source, scale, offset, compareTo);
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the previous slot has been true for a given number of seconds
     * 
     * @param timeSeconds The amount of time the previous slot needs to be true for in order for this to be true in [0 inclusive..1 exclusive)
     * @return current chain builder
     */
    public DigoutChainBuilder trueFor(double timeSeconds) {
        slot.lhsDataSource = DataSource.kZero;
        slot.rhsDataSource = DataSource.kZero;
        slot.additiveImmidiate = DigoutSlot.computeTimingImmidiate(timeSeconds);
        slot.opcode = DigoutOperation.kPrevSlotTrue;
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * Checks if the previous chain (if any) has been true for a given number of seconds
     * 
     * @param timeSeconds The amount of time the previous chain needs to be true for in order for this to be true in [0 inclusive..1 exclusive)
     * @return current chain builder
     */
    public DigoutChainBuilder prevChainTrueFor(double timeSeconds) {
        slot.lhsDataSource = DataSource.kZero;
        slot.rhsDataSource = DataSource.kZero;
        slot.additiveImmidiate = DigoutSlot.computeTimingImmidiate(timeSeconds);
        slot.opcode = DigoutOperation.kPrevSlotChainTrue;
        return new DigoutChainBuilder(this.chain, this.slot);
    }

    /**
     * <b>Advanced</b> Directly accept a {@link DigoutSlot} object.
     * Note that the chain/invert setting of the passed-in slot will be overwritten by the chain builder.
     * 
     * @param slot the DigoutSlot to write.
     * @return current chain builder
     */
    public DigoutChainBuilder directSlot(DigoutSlot slot) {
        this.slot = new DigoutSlot(slot);
        return new DigoutChainBuilder(this.chain, this.slot);
    }
}


