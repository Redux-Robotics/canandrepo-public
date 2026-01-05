// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import com.reduxrobotics.sensors.canandcolor.wpistruct.CanandcolorFaultsStruct;

import edu.wpi.first.util.struct.StructSerializable;

/**
 * A class to hold device faults for the {@link Canandcolor}, as returned by 
 * {@link Canandcolor#getStickyFaults()} and {@link Canandcolor#getActiveFaults()}.
 */
public class CanandcolorFaults implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final CanandcolorFaultsStruct struct = new CanandcolorFaultsStruct();
    private int faultField = 0;
    private boolean faultsValid = false;

    /**
     * Constructor for the {@link CanandcolorFaults} object.
     * @param faultField The byte from which to extract faults.
     * @param valid whether the data bitfield is valid (and not just uninited data)
     */ 
    public CanandcolorFaults(int faultField, boolean valid) {
        this.faultsValid = valid;
        if (this.faultsValid) {
            this.faultField = faultField;
        }
    }

    /**
     * Constructor, assuming valid data.
     * @param faultField The byte from which to extract faults.
     */
    public CanandcolorFaults(int faultField) {
        this(faultField, true);
    }

    /**
     * Returns the faults bitfield as an integer.
     * <p>
     * Which bits corresponds to which fault is documented in <a href="https://docs.reduxrobotics.com/canspec/Canandcolor#faults">the online message spec.</a>
     * </p>
     * @return faults bitfield
     */
    public int faultBitField() {
        return this.faultField;
    }

    /**
     * Returns the power cycle fault flag, which is set to true when the device first boots.
     * 
     * Clearing sticky faults and then checking this flag can be used to determine if the device
     * rebooted.
     * @return fault state
     */
    public boolean powerCycle() {
        return (faultField & 0b1) != 0;
    }

    /**
     * Returns the CAN ID conflict flag, which is set to true if there is a CAN id conflict.
     * In practice, you should physically inspect the device to ensure it's not flashing blue.
     * @return fault state
     */
    public boolean canIDConflict() {
        return (faultField & 0b10) != 0;
    }

    /**
     * Returns the CAN general error flag, which will raise if the device cannot RX packets 
     * reliably.
     * 
     * This is usually due to wiring issues, such as a shorted CAN bus.
     * @return fault state
     */
    public boolean canGeneralError() {
        return (faultField & 0b100) != 0;
    }

    /**
     * Returns the temperature range flag, which will raise if the device is not in its 
     * rated temperature range.
     * 
     * This may be of concern if the device is near very active motors.
     * @return fault state
     */
    public boolean outOfTemperatureRange() {
        return (faultField & 0b1000) != 0;
    }

    /**
     * Returns the proximity sensor hardware fault flag, which will raise if the proximity 
     * sensor is unreadable.
     * 
     * @return fault state
     */
    public boolean hardwareFaultProximity() {
        return (faultField & 0b10000) != 0;
    }

    /**
     * Returns the color sensor hardware fault flag, which will raise if the color sensor is 
     * unreadable.
     * 
     * @return fault state
     */
    public boolean hardwareFaultColor() {
        return (faultField & 0b100000) != 0;
    }

    /**
     * Returns the I²C bus recovery fault flag, which will raise if the color sensor is resetting
     * its internal I²C bus.
     * 
     * <p>
     * This fault is of no concern as long as it is not continuously active for long.
     * If the device is stuck in bus recovery, that may indicate a hardware defect.
     * </p>
     * 
     * @return fault state
     */
    public boolean i2cBusRecovery() {
        return (faultField & 0b1000000) != 0;
    }

    
    /**
     * Flag if any faults data has been received at all from the device.
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