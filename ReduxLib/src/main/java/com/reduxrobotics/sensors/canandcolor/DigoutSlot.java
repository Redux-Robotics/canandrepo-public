// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import edu.wpi.first.math.MathUtil;

/**
 * 
 * Represents a digital output slot -- programmable digital logic updated at 1000 Hz to evaluate sensor conditions fast.
 * 
 * <h2>Introduction</h2>
 * <p>
 * The basic concept of digital output slots are that they let the Canandcolor detect and analyze things independent of and faster
 * than the typical core robot control loop, as well as give signals to other control electronics (e.g. motor limit switch inputs).
 * </p>
 * <p>
 * For example, a game element moving through a robot's indexer really fast may need a fast reaction from control electronics
 * in order for an internal mechanical handoff to work. Digout slots can be used to detect when an element passes by and have
 * the Canandcolor immidiately output a digital signal and/or a CAN message indicating that it has seen an element.
 * 
 * <p>
 * Each "slot" evaluates individual conditions about the current sensor readings e.g. comparing red and blue channels or
 * or comparing the proximity sensor against a known constant proximity.
 * </p>
 * 
 * <h2>Simple configuration</h2>
 * For simple configuration, see {@link DigoutChannel#configureSlots} and {@link RGBDigoutConfig} and {@link HSVDigoutConfig}.
 * 
 * <h2>Advanced quickstart</h2>
 * <p>
 * If simple configuration doesn't cut it, {@link DigoutChain} can be used to configure digouts via the builder, 
 * </p>
 * For example:
 * <pre>
 * // Instantiate object
 * Canandcolor color = new Canandcolor(0); 
 * 
 * // Setup digout "slots"; in this case we check if something is close and if the red channel is bigger than the blue one.
 * // Up to 16 of these condition "slots" can be used.
 * color.digout1().configureSlotsAdvanced(DigoutChannel.Index.kDigout1, DigoutChain.start()
 *   .greaterThan(DataSource.kRed, DataSource.kBlue) // slot 0; is red &gt; blue?
 *   .and() // How the slots are chained together. This can be and, or, or xor. Slots are evaluated from first to last and are "folded" over.
 *   .lessThan(DataSource.kProximity, 0.1) // slot 1; is something close?
 *   .and()
 *   .trueFor(0.005) // slot 2; the thing has been close for at least 5 milliseconds
 *   .finish()
 * );
 * 
 * // Configure the physical digital outputs as well.
 * // Digout slots are always evaluated regardless of output configuration
 * color.setSettings(
 *     new CanandcolorSettings()
 *          // configure DIG1 to output high on true, false otherwise
 *          .setDigoutPinConfig(DigoutChannel.Index.kDigout1, new DigoutPinConfig.kDigoutLogicActiveHigh)
 *          // send additional digout CAN messages when the digout state changes.
 *          // This ensures when you read digout state, you're always getting up-to-date information.
 *          .setDigoutFrameTrigger(DigoutChannel.Index.kDigout1, DigoutFrameTrigger.kRisingAndFalling)
 * );
 * color.clearStickyDigoutFlags();
 * 
 * // ...in your periodic loop handler...
 * if (color.digout1().getValue()) {
 *    // the digout state is currently true (proximity is less than 0.1 and red > blue for at least 5 ms)
 * }
 * 
 * if (color.digout1().getStickyValue()) {
 *    // The digout state has been set to true at some point since the last iteration when the sticky flags were cleared.
 *    // This can help catch transient conditions that may not stay true for longer than your robot update loop period.
 * }
 * 
 * // Reset the digout flags to detect changes on the next round.
 * color.clearStickyDigoutFlags();
 * // ... end periodic loop ...
 * 
 * </pre>
 * 
 * <p>While the boolean outputs can be configured to be outputted on the physical the digital output pins, 
 * the boolean state of each slot (and the overall channel outputs) is also always available over CAN.
 * This {@link DigoutSlotState} can be retreived using {@link Canandcolor#getDigoutState}, and further behavior can be configured 
 * using {@link CanandcolorSettings} or {@link DigoutChannel} to allow for interrupt-driven operation over CAN.
 * </p>
 * 
 * 
 * <h2>The more technical guts</h2>
 * <h3>Digout layout</h3>
 * <p>
 * Individual digout slots (numbered 0 to 15 inclusive) are composed of the following components:
 * </p>
 * <ol>
 * <li>A boolean enabled field; if disabled, the slot always evaluates to true</li>
 * <li>How the slot "chains" with the {@link NextSlotAction next slot}</li>
 * <li>A boolean indicating whether the slot should invert its currently evaluated condition</li>
 * <li> {@link DigoutOperation The comparison operation} to execute</li>
 * <li>A "left hand side" or LHS {@link DataSource data source}</li>
 * <li>A "right hand side" or RHS {@link DataSource data source} that can be scaled or added to before comparing with the LHS value.</li>
 * <li>An 8-bit unsigned "scaling immidiate" used to scale the RHS data source</li>
 * <li>A 21-bit signed "additive immidiate" added to the scaled RHS data source</li>
 * </ol>
 * <p>
 * Digout slots are stored as settings in the Canandcolor's firmware. Each slot has an addressible setting index.
 * Both {@link DigoutChannel.Index#kDigout1} and {@link DigoutChannel.Index#kDigout1} have their own independent set of digout slots.
 * </p>
 * 
 * <p>
 * The integer immidiates can be computed from scaled [0..1] magnitudes (as they are used in this vendor library) via
 * the {@link #computeAdditiveImmidiate(double)}, {@link #computeMultiplicativeImmidiate(double)}, and {@link #computeTimingImmidiate(double)}
 * methods. 
 * </p>
 * 
 * <h3>Digout evaluation</h3>
 * Under the hood, the true or false value of each slot is computed by taking
 * 
 * <pre class="not-code">
 * lhsDataSource [operation] (scalingImmidiate + 1) / 256 * rhsDataSource + additiveImmidiate
 * </pre>
 * 
 * for operations comparing equality, less than (or equal to) and greater than (or equal to).
 * 
 * If the RHS could go negative, it saturates at 0.
 * 
 * <p>For operations that concern the period of time the previous slot or chain has been true for, the threshold value 
 * in milliseconds is computed by:</p>
 * 
 * <pre class="not-code">
 * milliseconds = (scalingImmidiate + 1) / 256 * rhsDataSource + additiveImmidiate
 * </pre>
 * 
 * and the rhsDataSource should be set to {@link DataSource#kZero}.
 * 
 * <p>If the invert flag is set, then the above conditions have their output run through a NOT operation.</p>
 * <h3>Digout chains </h3>
 * <p>
 * Digout slots can be grouped into "chains" by specifying their {@link NextSlotAction next slot action} field.
 * Each chain is comprised of adjacent slots that are joined together with {@link NextSlotAction#kAndWithNextSlot},
 * {@link NextSlotAction#kOrWithNextSlot}, or {@link NextSlotAction#kXorWithNextSlot} until a slot specifies
 * {@link NextSlotAction#kTerminateChain}. 
 * <b>Each chain must be true for the overall digital output value to be true,</b> and this behavior can be used to gate
 * entire expressions around time requirements (e.g. for debouncing reasons)
 * </p>
 * 
 * <p>
 * For example, to evaluate the condition
 * </p>
 * <pre class="not-code">
 * red &gt; 0.2 for 0.01 sec AND ((blue &gt; 0.2 OR green &gt; 0.2) for 0.1 sec) AND (proximity &lt; 0.1 for 0.03 sec)
 * </pre>
 * 
 * One might configure:
 * <pre>
 * // Instantiate object
 * Canandcolor color = new Canandcolor(0); 
 * color.digout1().configureSlotsAdvanced(
 *   DigoutChain.start()
 *     .greaterThan(DataSource.kRed, 0.2) // slot 0; red &gt; 0.2, kAndWithNextSlot
 *     .and()
 *     .trueFor(0.01) // slot 1; prevTrueFor(0.01 seconds), kTerminateChain
 *     .finish(),
 *   DigoutChain.start()
 *     .greaterThan(DataSource.kBlue, 0.2) // slot 2; blue &gt; 0.2, kOrWithNextSlot
 *     .or()
 *     .greaterThan(DataSource.kGreen, 0.2) // slot 3; green &gt; 0.2, kTerminateChain
 *     .finish(),
 *   DigoutChain.start()
 *     .prevChainTrueFor(0.1) // slot 4; prevTrueFor(0.1 seconds), kTerminateChain
 *     .finish(),
 *   DigoutChain.start()
 *     .lessThan(DataSource.kProximity, 0.1) // slot 5; proximity &lt; 0.1, kAndWithNextSlot
 *     .and()
 *     .trueFor(0.03) // slot 5; prevTrueFor(0.03 seconds), kTerminateChain
 *     .finish()
 * );
 * </pre>
 * 
 * <p>
 * Each chain represents a self-contained unit that gets ANDed into the greater evaluated expression.
 * All chains thus must be true for the overall channel output to be true.
 * </p>
 * 
 */
class DigoutSlot {

    /** Whether the digout slot is enabled */
    public boolean enabled;
    /** Whether the digout slot should interact with the next numbered slot */
    public NextSlotAction nextSlotAction;
    /** Whether the digout slot should invert its value */
    public boolean invertValue;
    /** The operation to perform */
    public DigoutOperation opcode;
    /** The 21-bit additive immidiate */
    public int additiveImmidiate;
    /** The 8-bit scaling immidiate */
    public int scalingImmidiate;
    /** The LHS data source */
    public DataSource lhsDataSource;
    /** The RHS data source */
    public DataSource rhsDataSource;


    /**
     * Constructor.
     * 
     * <p>Truthiness is evaluated as:</p>
     * 
     * <pre class="not-code">
     * DataSource[lhsDataSource] [operation] (scalingImmidiate + 1) / 256 * DataSource[rhsDataSource] + ((additiveImmidiate &lt;&lt; 11) &gt;&gt; 11)
     * </pre>
     * 
     * For more information, see the class-level doc comment.
     * 
     * @param enabled whether the digout is enabled
     * @param nextSlotAction if and how the digout slot should chain with the next indexed slot
     * @param invertValue whether to invert the value of the operation
     * @param opcode the operation to perform
     * @param additiveImmidiate A 2's complement signed immidiate that is added to the (scaled) RHS data source; lower 21 bits are used
     * @param scalingImmidiate An unsigned immidiate that is used to scale the RHS data source; lower 8 bits are used
     * @param lhsDataSource The LHS data source
     * @param rhsDataSource The RHS data source
     */
    public DigoutSlot(
        boolean enabled, 
        NextSlotAction nextSlotAction, 
        boolean invertValue,
        DigoutOperation opcode,
        int additiveImmidiate,
        int scalingImmidiate,
        DataSource lhsDataSource,
        DataSource rhsDataSource
    ) {
        this.enabled = enabled;
        this.nextSlotAction = nextSlotAction;
        this.invertValue = invertValue;
        this.opcode = opcode;
        this.additiveImmidiate = additiveImmidiate;
        this.scalingImmidiate = scalingImmidiate;
        this.lhsDataSource = lhsDataSource;
        this.rhsDataSource = rhsDataSource;
    }

    /**
     * Clone constructor.
     * <p>For more information, see the class-level doc comment.</p>
     * @param other other slot.
     */
    public DigoutSlot(DigoutSlot other) {
        this.enabled = other.enabled;
        this.nextSlotAction = other.nextSlotAction;
        this.invertValue = other.invertValue;
        this.opcode = other.opcode;
        this.additiveImmidiate = other.additiveImmidiate;
        this.scalingImmidiate = other.scalingImmidiate;
        this.lhsDataSource = other.lhsDataSource;
        this.rhsDataSource = other.rhsDataSource;
    }
    
    /**
     * Returns a disabled digital output slot config.
     * Disabled slots always get evaluated to true.
     * To set all slots for a digout to be disabled, use {@link DigoutChannel#clearAllDigoutSlots()}
     * <p>
     * An instance of this object can be passed to {@link DigoutChannel#setRawDigoutSlot(int, DigoutSlot, double)} to apply it.
     * </p>
     * @return a disabled slot
     */
    public static DigoutSlot disabled() {
        return new DigoutSlot(
            false,
            NextSlotAction.kTerminateChain,
            false,
            DigoutOperation.kEquals,
            0,
            0,
            DataSource.kZero,
            DataSource.kZero
        );
    }

    /**
     * Deserializes a 48-bit setting value into a {@link DigoutSlot} object.
     * @param data setting data
     * @return {@link DigoutSlot} object
     */
    public static DigoutSlot fromSettingData(long data) {
        
        return new DigoutSlot(
            CanandcolorDetails.Stg.extractDigout1Config0_SlotEnabled(data),
            NextSlotAction.fromIndex(CanandcolorDetails.Stg.extractDigout1Config0_NextSlotAction(data)),
            CanandcolorDetails.Stg.extractDigout1Config0_InvertValue(data),
            DigoutOperation.fromIndex(CanandcolorDetails.Stg.extractDigout1Config0_Opcode(data)),
            CanandcolorDetails.Stg.extractDigout1Config0_ImmidiateAdditive(data),
            CanandcolorDetails.Stg.extractDigout1Config0_ImmidiateScaling(data),
            DataSource.fromIndex(CanandcolorDetails.Stg.extractDigout1Config0_DataSourceA(data)),
            DataSource.fromIndex(CanandcolorDetails.Stg.extractDigout1Config0_DataSourceB(data))
        );
    }

    /**
     * Compute internal 21-bit additive immidiate.
     * @param value value from [-1 inclusive..1 exclusive)
     * @return 21-bit value. 
     */
    public static int computeAdditiveImmidiate(double value) {
        return (int) (MathUtil.clamp(value, -1.0, 1.0) * ((1 << 20)-1));
    }

    /**
     * Compute internal 8-bit multiplicative immidiate
     * @param value value form [0 exclusive...1 inclusive]
     * @return 8-bit scaling immidiate
     */
    public static int computeMultiplicativeImmidiate(double value) {
        return MathUtil.clamp((int) (value * 256) , 1, 256) - 1;
    }

    /**
     * Computes a new timing immidiate (for prevSlot/ChainTrue)
     * @param value time to convert from seconds to immidiate format
     * @return 21-bit timing immidiate 
     */
    public static int computeTimingImmidiate(double value) {
        return MathUtil.clamp((int) (value * 1000), 0, (1<<20)-1);
    }

    /**
     * Serializes a digout slot to setting data
     * @return 48-bit setting data
     */
    public long toSettingData() {
        return CanandcolorDetails.Stg.constructDigout1Config0(
            enabled,
            nextSlotAction.getIndex(),
            invertValue,
            opcode.getIndex(),
            additiveImmidiate,
            scalingImmidiate,
            lhsDataSource.getIndex(),
            rhsDataSource.getIndex()
        );
    }

    @Override
    public String toString() {
        return String.format(
            "DigoutSlot(\n" + 
            "  enabled=%b,\n" +
            "  nextSlotAction=%s,\n" + 
            "  invert=%b,\n" +
            "  opcode=%s,\n" +
            "  additive=%d,\n" +
            "  scaling=%d,\n" +
            "  lhs=%s,\n" +
            "  rhs=%s,\n)", 
            enabled, 
            nextSlotAction.toString(),
            invertValue,
            opcode.toString(),
            additiveImmidiate,
            scalingImmidiate,
            lhsDataSource.toString(),
            rhsDataSource.toString()
        );

    }
}
