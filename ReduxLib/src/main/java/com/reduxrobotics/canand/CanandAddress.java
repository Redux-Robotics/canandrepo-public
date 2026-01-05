// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

/**
 * Class representing the exact combination of CAN bus, product IDs, and device IDs that uniquely 
 * correspond to a Redux CAN device on a robot.
 * 
 * <p>
 * The full 29-bit CAN ID that actually gets put on the bus can be broken down into four components:
 * </p>
 * <ol>
 * <li> A 5 bit device type (devType) that is product specific (the Canandmag is 7 for 
 * "gear tooth sensor")</li>
 * <li> an 8-bit manufacturer code unique to each vendor (the Redux code is 14) </li>
 * <li> 10 bits of API identifier</li>
 * <li> a 6-bit device number (devId) that is user-configurable so one can have multiple of a device
 * on a bus (this is what the "CAN Id" in robot code and vendor tuners usually refer to)</li>
 * </ol>
 * 
 * <a href="https://docs.wpilib.org/en/stable/docs/software/can-devices/can-addressing.html"> The 
 * WPILib docs</a> elaborate on this in a bit more detail. 
 * 
 * Of note is that it breaks down the 10-bit API identifier into a 6-bit API class and 4-bit API 
 * index. Redux products, however, break it down into a 2 bit API page (page) and an 8 bit API index
 * (apiIndex), which actually gets used for a device's message API. 
 * 
 * <p>
 * The breakdown can be seen in the diagram provided below:
 * </p>
 * <pre class="not-code">
 * +-------------------+-------------------------------+-------+-------------------------------+-----------------------+
 * |    Device Type    | Manufacturer ID   (redux=0xE) | Page  |           API Index           |   Device ID (devID)   |
 * +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
 * | 28| 27| 26| 25| 24| 23| 22| 21| 20| 19| 18| 17| 16| 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
 * +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
 * </pre>
 * Documentation on what each apiIndex value does as well as DBCs for Redux devices is available at 
 * <a href="https://docs.reduxrobotics.com/">https://docs.reduxrobotics.com/</a>
 * 
 * <p>
 * In summary, to check if an incoming CAN message is from the device the CanandAddress represents, 
 * it needs to check:
 * </p>
 * <ul>
 * <li> the devType </li>
 * <li> the devId </li>
 * <li> and the CAN bus the message was sent from, as different devices on different buses can share
 * identical IDs</li>
 * </ul>
 * to ensure they all match, leaving the apiIndex to be parsed by device class code. 
 * (The manufacturer ID is filtered for in the native driver portion of ReduxLib.)
 * 
 * <p>
 * Subclasses of {@link CanandDevice} will use this class for CAN i/o to a device, as this class 
 * provides a method to send CAN packets to the device it describes, and {@link CanandEventLoop} 
 * uses it to determine which messages to give to {@link CanandDevice#handleMessage} by checking 
 * them against {@link msgMatches}, yielding an asynchronous method of receiving packets.
 * </p>
 * 
 * 
 * 
 * 
 */
public class CanandAddress {
    private MessageBus bus;
    private int devType;
    private int devId;
    private int baseMsgId;

    /**
     * Constructor with explicit CAN bus.
     * @param bus the bus the address is associated with
     * @param devType the device type
     * @param devId the device CAN id
     */
    public CanandAddress(MessageBus bus, int devType, int devId)  {
        if (devType < 0 || devType > 31) {
            throw new IllegalArgumentException("devType must be between 0 and 31");
        }
        if (devId < 0 || devId > 63) {
            throw new IllegalArgumentException("CAN Device ID must be between 0 and 63");
        }
        this.bus = bus;
        this.devType = devType;
        this.devId = devId;
        this.baseMsgId = CanandUtils.constructMessageId(devType, devId, 0, 0);
    }

    /**
     * Construct new address with a bus string.
     * 
     * @param busString bus string
     * @param devType device type
     * @param devId device id
     */
    public CanandAddress(String busString, int devType, int devId) {
        this.bus = MessageBus.byBusString(busString);
        this.devType = devType;
        this.devId = devId;
        this.baseMsgId = CanandUtils.constructMessageId(devType, devId, 0, 0);
    }

    /**
     * Constructor with implicit Rio CAN bus.
     * @param devType the device type
     * @param devId the device CAN id
     */
    public CanandAddress(int devType, int devId)  {
        this(MessageBus.getRioBus(), devType, devId);
    }

    /**
     * Checks if a CAN message matches against the device type, product id, and device can id
     * 
     * @param msg a CanandMessage matching
     * @return if there is a match
     */
    public boolean msgMatches(CanandMessage msg) {
        return CanandUtils.idMatches(msg.getId(), devType, devId) && bus.equals(msg.getBus());
    }

    /**
     * Sends a CAN message to the CanandAddress.
     * @param apiIndex the API index the message should have (between 0-255 inclusive)
     * @param data 0-8 bytes of payload -- length of array will correspond to length of packet
     * @return if the operation was successful
     */
    public boolean sendCANMessage(int apiIndex, byte[] data) {
        if (apiIndex < 0 || apiIndex > 0xff) return false;
        return ReduxJNI.sendCANMessage(bus, baseMsgId | (apiIndex << 6), data);
    }

    /**
     * Sends a CAN message to the CanandAddress.
     * @param apiIndex the API index the message should have (between 0-255 inclusive)
     * @param data 0-8 bytes of payload -- encoded as little endian long (lower bits == lower bytes)
     * @param length length of payload [0..8] inclusive
     * @return if the operation was successful
     */
    public boolean sendCANMessage(int apiIndex, long data, int length) {
        if (apiIndex < 0 || apiIndex > 0xff) return false;
        return ReduxJNI.sendCANMessage(bus, baseMsgId | (apiIndex << 6), data, length);
    }

    /**
     * Sends a repeating CAN message to the CanandAddress.
     * 
     * @param repeater The message repeater handle to use
     * @param apiIndex the API index of the message
     * @param data the data as a 64-bit long
     * @param length the length of the data
     * @param periodMs the period in milliseconds to repeat the message at
     * @param times the number of times to repeat the message before the repeater will sleep
     */
    public void sendRepeatingCANMessage(Repeater repeater, int apiIndex, long data, int length, int periodMs, int times) {
        if (apiIndex < 0 || apiIndex > 0xff) return;
        repeater.update(bus.getDescriptor(), baseMsgId | (apiIndex << 6), data, length, periodMs, times);
    }

    /**
     * Sends a repeating CAN message to the CanandAddress.
     * 
     * This will repeat the message at a 40 millisecond frequency 25 times (1 second).
     * 
     * @param repeater The message repeater handle to use
     * @param apiIndex the API index of the message
     * @param data the data as a 64-bit long
     * @param length the length of the data
     */
    public void sendRepeatingCANMessage(Repeater repeater, int apiIndex, long data, int length) {
        sendRepeatingCANMessage(repeater, apiIndex, data, length, 40, 25);
    }

    /**
     * Returns the 5-bit device type.
     * 
     * @return the device type
     */
    public int getDeviceType() {
        return devType;
    }

    /**
     * Returns the user-settable device ID.
     * 
     * @return the device ID
     */
    public int getDeviceId() {
        return devId;
    }

    /**
     * Returns the bus associated with the address.
     * @return bus
     */
    public MessageBus getBus() {
        return bus;
    }
}
