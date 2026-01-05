// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro;

import com.reduxrobotics.sensors.canandgyro.wpistruct.CanandgyroFaultsStruct;

import edu.wpi.first.util.struct.StructSerializable;

/**
 * A class to hold device faults for the {@link Canandgyro}, as returned by 
 * {@link Canandgyro#getStickyFaults} and {@link Canandgyro#getActiveFaults}.
 */
public class CanandgyroFaults implements StructSerializable {
    /** This field is necessary for WPILib structs to work around Java type erasure. */
    public static final CanandgyroFaultsStruct struct = new CanandgyroFaultsStruct();
    private int faultField = 0;
    private boolean faultsValid = false;

    /**
     * Constructor for the {@link CanandgyroFaults} object.
     * @param faultField The byte from which to extract faults.
     * @param valid whether the data bitfield is valid (and not just uninited data)
     */ 
    public CanandgyroFaults(int faultField, boolean valid) {
        this.faultsValid = valid;
        if (this.faultsValid) {
            this.faultField = faultField;
        }
    }

    /**
     * Returns the faults bitfield as an integer.
     * <p>
     * Which bits corresponds to which fault is documented in <a href="https://docs.reduxrobotics.com/canspec/Canandgyro#faults">the online message spec.</a>
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
        return CanandgyroDetails.Bitsets.extractFaults_PowerCycle(faultField);
    }

    /**
     * Returns the CAN ID conflict flag, which is set to true if there is a CAN id conflict.
     * In practice, you should physically inspect the device to ensure it's not flashing blue.
     * @return fault state
     */
    public boolean canIDConflict() {
        return CanandgyroDetails.Bitsets.extractFaults_CanIdConflict(faultField);
    }

    /**
     * Returns the CAN general error flag, which will raise if the device has encountered a bus fault.
     * This typically indicates a physical wiring issue on the robot, such as loose connections or 
     * an intermittently shorting CAN bus
     * @return fault state
     */
    public boolean canGeneralError() {
        return CanandgyroDetails.Bitsets.extractFaults_CanGeneralError(faultField);
    }

    /**
     * Returns the out of temperature spec flag, which will raise if the device is above 95 
     * degrees Celsius.
     * 
     * This may be of concern if the device is near very active motors.
     * @return fault state
     */
    public boolean outOfTemperatureRange() {
        return CanandgyroDetails.Bitsets.extractFaults_OutOfTemperatureRange(faultField);
    }

    /**
     * Returns the hardware fault flag, which will raise if a hardware issue is detected.
     * Generally will raise if the device's controller cannot read the physical sensor itself.
     * 
     * @return fault state
     */
    public boolean hardwareFault() {
        return CanandgyroDetails.Bitsets.extractFaults_HardwareFault(faultField);
    }

    /**
     * Returns the calibrating flag, which will raise if the device is currently calibrating.
     * 
     * @return fault state
     */
    public boolean calibrating() {
        return CanandgyroDetails.Bitsets.extractFaults_Calibrating(faultField);
    }

    /**
     * Returns the angular velocity saturation flag which will raise if angular velocity 
     * has been sensed to saturate (potentially degrading accuracy.)
     * 
     * @return fault state
     */
    public boolean angularVelocitySaturation() {
        return CanandgyroDetails.Bitsets.extractFaults_AngularVelocitySaturation(faultField);
    }

    /**
     * Returns the acceleration flag which will raise if acceleration 
     * has been sensed to saturate (potentially degrading accuracy.)
     * 
     * @return fault state
     */
    public boolean accelerationSaturation() {
        return CanandgyroDetails.Bitsets.extractFaults_AccelerationSaturation(faultField);
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