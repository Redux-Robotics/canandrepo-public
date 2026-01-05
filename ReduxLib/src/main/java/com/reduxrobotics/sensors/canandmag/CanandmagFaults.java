// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag;

import com.reduxrobotics.sensors.canandmag.wpistruct.CanandmagFaultsStruct;

import edu.wpi.first.util.struct.StructSerializable;

/**
 * A class to hold device faults for the {@link Canandmag}. 
 * Returned by {@link Canandmag#getStickyFaults} and {@link Canandmag#getActiveFaults}.
 */
public class CanandmagFaults implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final CanandmagFaultsStruct struct = new CanandmagFaultsStruct();
    private int faultField = 0;
    private boolean faultsValid = false;

    /**
     * Constructor for the CanandmagFaults object.
     * @param faultField The byte from which to extract faults.
     * @param valid Whether or not the data is valid (and not from some initialization value)
     */
    public CanandmagFaults(int faultField, boolean valid) {
        this.faultsValid = valid;
        if (this.faultsValid) {
            this.faultField = faultField;
        }
    }

    /**
     * Constructor for the CanandmagFaults object.
     * @param faultField The byte from which to extract faults.
     */
    public CanandmagFaults(int faultField) {
        this(faultField, true);
    }

    /**
     * Returns the faults bitfield as an integer.
     * <p>
     * Which bits corresponds to which fault is documented in <a href="https://docs.reduxrobotics.com/canspec/Canandmag#faults">the online message spec.</a>
     * </p>
     * @return faults bitfield
     */
    public int faultBitField() {
        return this.faultField;
    }

    /**
     * Returns the power cycle fault flag, which is set to true when the encoder first boots.
     * Clearing sticky faults and then checking this flag can be used to determine if the 
     * encoder rebooted.
     * @return fault state
     */
    public boolean powerCycle() {
        return (faultField & 0b1) != 0;
    }

    /**
     * Returns the CAN ID conflict flag, which is set to true if there is a CAN id conflict.
     * In practice, you should physically inspect the encoder to ensure it's not flashing blue.
     * @return fault state
     */
    public boolean canIDConflict() {
        return (faultField & 0b10) != 0;
    }

    /**
     * Returns the CAN general error flag, which will raise if the encoder cannot RX packets 
     * reliably.
     * This is usually due to wiring issues, such as a shorted CAN bus.
     * @return fault state
     */
    public boolean canGeneralError() {
        return (faultField & 0b100) != 0;
    }

    /**
     * Returns the temperature range flag, which will raise if the encoder is not between 0-70 
     * degrees Celsius.
     * This may be of concern if the encoder is near very active motors.
     * @return fault state
     */
    public boolean outOfTemperatureRange() {
        return (faultField & 0b1000) != 0;
    }

    /**
     * Returns the hardware fault flag, which will raise if a hardware issue is detected.
     * Generally will raise if the device's controller cannot read the physical sensor itself.
     * @return fault state
     */
    public boolean hardwareFault() {
        return (faultField & 0b10000) != 0;
    }

    /**
     * Returns the magnet out of range flag, which will raise if the measured shaft's magnet is 
     * not detected.
     * This will match the encoder's LED shining red in normal operation.
     * @return fault state
     */
    public boolean magnetOutOfRange() {
        return (faultField & 0b100000) != 0;
    }

    /**
     * Returns the undervolt flag, which will raise if the encoder is experiencing brownout 
     * conditions.
     * @return fault state
     */
    public boolean underVolt() {
        return (faultField & 0b1000000) != 0;
    }

    
    /**
     * Flag if any faults data has been received at all from the device.
     * 
     * <p>
     * This will be faults until the first status frame arrives either
     * after the start of robot code or after sticky faults have been cleared.
     * </p>
     * @return if fault data has actually been received (and thus if the other fields are valid)
     */
    public boolean faultsValid() {
        return faultsValid;
    }

}