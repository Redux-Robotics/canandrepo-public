// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import edu.wpi.first.wpilibj.DriverStation;

/**
 * Base class for Redux CAN devices. 
 * The ReduxLib vendordep does all CAN message parsing in the Java/C++ classes themselves, rather
 * than the driver.
 * 
 * CanandDevice provides general facilities for sending and receiving CAN messages (abstracted from 
 * the actual underlying buses) as well as helper functions and constants common for all Redux CAN 
 * products.
 * 
 * Classes implementing CanandDevice need to do the following:
 * <ul>
 * <li> 
 * Implement {@link getAddress}, usually by instantiating a {@link CanandAddress} in the constructor
 * and returning it
 * </li>
 * <li> Implement {@link handleMessage} which will be called asynchronously whenever new CAN 
 * messages matching the object's {@link CanandAddress} get received by the robot 
 * </li>
 * <li> 
 * Run super() in the constructor so {@link handleMessage} actually gets called at runtime 
 * </li>
 * </ul>
 */
 public abstract class CanandDevice implements AutoCloseable {

    // These are common CAN API message ids shared across all CanandDevices
    private CanandFirmwareVersion receivedFirmwareVersion = null;

    /** 
     * The last received message timestamp. 
     * This is timed with respect to the FPGA timer, and is updated before {@link #handleMessage}
     *  gets called in {@link #preHandleMessage}.
     */
    protected double lastMessageTs = Double.NEGATIVE_INFINITY;

    /**
     * Default constructor that just adds the device to the incoming CAN message listener. 
     */
    public CanandDevice() {
        CanandEventLoop.getInstance().addListener(this);
    }

    /**
     * A callback called when a Redux CAN message is received and should be parsed.
     * Subclasses of {@link CanandDevice} should override this to update their internal state 
     * accordingly.
     * 
     * <p>
     * handleMessage will be called on all Redux CAN packets received by the vendordep that match 
     * the {@link CanandAddress} returned by {@link CanandDevice#getAddress()}.
     * </p>
     * 
     * @param msg a {@link CanandMessage} representing the received message.
     */
    public abstract void handleMessage(CanandMessage msg);

    /**
     * Returns the {@link CanandAddress} representing the combination of CAN bus and 
     * CAN device ID that this CanandDevice refers to.
     * 
     * <p>
     * Implementing device subclasses should likely construct a new {@link CanandAddress}
     * in their constructor and return it here.
     * </p>
     * @return the {@link CanandAddress} for the device.
     */
    public abstract CanandAddress getAddress();

    /**
     * Called before {@link #handleMessage} gets called to run some common logic. 
     * (namely, handling setting receives and last message times)
     * 
     * This function can be overridden to change or disable its logic.
     * 
     * @param msg a {@link CanandMessage} representing the received message.
     */
    public void preHandleMessage(CanandMessage msg) {
        lastMessageTs = msg.getTimestamp();
        // Handle the setting receive.
        byte[] data = msg.getData();
        if (msg.getApiIndex() == CanandDeviceDetails.Msg.kReportSetting
                && msg.getLength() >= 7 && data[0] == CanandDeviceDetails.Stg.kFirmwareVersion) {
            synchronized (this) {
                receivedFirmwareVersion = new CanandFirmwareVersion(CanandUtils.extractLong(data, 8, 56, false));
            }
        }
    }

    /**
     * Checks whether or not the device has sent a message within the last timeout seconds.
     * @param timeout window to check for message updates in seconds
     * @return true if there has been a message within the last timeout seconds, false if not
     */
    public boolean isConnected(double timeout) {
        return (CanandUtils.getFPGATimestamp() - lastMessageTs) <= timeout;
    }

    /**
     * Checks whether or not the device has sent a message within the last 2000 milliseconds.
     * @return true if there has been a message within the last 2000 milliseconds, false if not
     */
    public boolean isConnected() {
        return isConnected(2);
    }

    /**
     * Checks the received firmware version.
     * 
     * <p>
     * If no firmware version has been received, complain to the driver station about potentially 
     * missing devices from the bus.
     * </p>
     * <p>
     * If the reported firmware version is too old, also complain to the driver station.
     * </p>
     * 
     */
    protected synchronized void checkReceivedFirmwareVersion() {
        CanandFirmwareVersion minFirmwareVersion = getMinimumFirmwareVersion();
        if (minFirmwareVersion == null) return;
        if (receivedFirmwareVersion == null) {
            // yell that the device may not be on bus
            DriverStation.reportError(
                String.format("%s did not respond to a firmware version check" + 
                " -- is the device powered and connected to the robot?", 
                toString()), false);
            return;
        }
        String hostname = System.getenv("HOSTNAME");
        if (hostname == null) {
            hostname = "roborio-XXXX-frc";
        }

        if (receivedFirmwareVersion.compareTo(getMinimumFirmwareVersion()) < 0) {
            DriverStation.reportError(
                String.format("%s is running too old firmware (%s < minimum %s)" + 
                " -- please update the device at http://%s.local:7244/ to avoid unforeseen errors!",
                toString(), 
                receivedFirmwareVersion.toString(), 
                getMinimumFirmwareVersion().toString(),
                hostname
            ), false);
        }
    }

    /**
     * Returns the minimum firmware version this vendordep requires for this device.
     * 
     * User-implmenting classes can return null to disable firmware checks.
     * @return minimum firmware version
     */
    public CanandFirmwareVersion getMinimumFirmwareVersion() {
        return new CanandFirmwareVersion(0);
    }

    /**
     * Returns the firmware version received by the device.
     * 
     * May return null if a firmware version has not been received from the device yet.
     * @return the received firmware version or null
     */
    public synchronized CanandFirmwareVersion getFirmwareVersion() {
        return receivedFirmwareVersion;
    }

    /**
     * Directly sends a CAN message to the device.
     * 
     * <p>
     * Implementing device classes should call this function to send messages to the device, 
     * or use {@code getAddress().sendCANMessage(int, byte[])}, which this aliases.
     * </p>
     * 
     * @param apiIndex the individual API index to value to send
     * @param data 0-8 byte bytes payload
     */
    public void sendCANMessage(int apiIndex, byte[] data) {
        getAddress().sendCANMessage(apiIndex, data);
    }

    /**
     * Directly sends a CAN message to the device.
     * 
     * <p>
     * Implementing device classes should call this function to send messages to the device, 
     * or use {@code getAddress().sendCANMessage(int, long, int)}, which this aliases.
     * </p>
     * 
     * @param apiIndex the individual API index to value to send
     * @param data 0-8 byte bytes payload (little endian long)
     * @param length length of data [0..8] inclusive
     */
    public void sendCANMessage(int apiIndex, long data, int length) {
        getAddress().sendCANMessage(apiIndex, data, length);
    }

    @Override
    public String toString() {
        return String.format("%s[device_id=%d]", 
            this.getClass().getSimpleName(),
            this.getAddress().getDeviceId()
        );
    }

    @Override
    public void close() {
        CanandEventLoop.getInstance().removeListener(this);
    }
}