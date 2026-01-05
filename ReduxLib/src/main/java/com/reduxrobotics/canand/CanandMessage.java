// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;
import java.nio.ByteBuffer;


/**
 * 
 * Class that represents a CAN message received from the Redux {@link CanandEventLoop}
 * 
 * This class is generally (re)initialized by {@link updateFromByteBuf} with a {@link ByteBuffer} 
 * from the JNI via {@link CanandEventLoop}. From there, it is then passed into 
 * {@link CanandDevice#handleMessage} through the event loop.
 * 
 * <p>
 * To avoid garbage collection/heap allocation, the CanandMessage object passed into handleMessage()
 * gets reused and has its contents overwritten upon receipt of a new CAN message.
 * To hold onto the contained values for longer, use the copy constructor to create a new object.
 * </p>
 * 
 * <p>
 * Of particular note are {@link #getData} to read the packet's data and {@link #getApiIndex} to see
 * what type of packet it is.
 * </p>
 * 
 * 
 */
public class CanandMessage {
    private int id;
    private int length;
    private MessageBus bus;
    private double timestamp;
    private byte[] data;

    /**
     * Construct a new {@link CanandMessage} from nothing (to be filled in later.)
     */
    public CanandMessage() {
        id = 0;
        length = 0;
        timestamp = 0;
        data = new byte[64];
        bus = null;
    }

    /**
     * Copy constructor
     * @param other other object to copy values from
     */
    public CanandMessage(CanandMessage other) {
        this.id = other.id;
        this.timestamp = other.timestamp;
        this.data = other.data.clone();
        this.bus = other.bus;
        this.length = other.length;
    }

    /**
     * Updates a {@link CanandMessage} from a {@link ByteBuffer} returned from the JNI interface.
     * The byte buffer must be little endian.
     * 
     * Used by the event loop to overwrite data with a new packet.
     * @param bb {@link ByteBuffer} from Redux JNI
     */
    CanandMessage updateFromByteBuf(ByteBuffer bb) {
        id = bb.getInt(); // message_id: u32
        bus = MessageBus.byDescriptor(((int) bb.getShort()) & 0xffff); // bus_id: u16
        bb.get(); // pad byte
        length = Math.min(bb.get() & 0xff, 64); // data_size: u8
        timestamp = bb.getLong() / 1000000.0; // timestamp_us: u64 

        bb.get(data, 0, data.length); // data: [u8; 64]

        return this;
    }

    void updateFromData(MessageBus bus, int messageID, byte[] messageData) {
        this.bus = bus;
        this.id = messageID;
        this.length = data.length;
        System.arraycopy(messageData, 0, this.data, 0, this.length);
    }

    void updateFromLong(MessageBus bus, int messageID, long messageData, int length) {
        this.bus = bus;
        this.id = messageID;
        this.length = length;
        for (int i = 0 ; i < length; i++) {
            this.data[i] = (byte) (messageData & 0xff);
            messageData >>= 8;
        }
    }


    void writeToByteBuf(ByteBuffer bb) {
        bb.putInt(id);               // message_id: u32,
        bb.putShort((short) bus.getDescriptor()); // bus_id: u16,
        bb.put((byte) 0);            // pad: u8,
        bb.put((byte) length);       // data_size: u8,
        bb.putLong(0);               // timestamp_us: u64,
        bb.put(data);                // data: [u8; 64],
    }

    /**
     * Gets the full 29-bit CAN message id.
     * 
     * A summary of how the CAN message id works is described in {@link CanandAddress}.
     * @return The full 29-bit message id.
     */
    public int getId() {
        return id;
    }

    /**
     * Gets the 5-bit CAN API index. 
     * 
     * This is the value that generally describes what type of CAN message was sent.
     * @return the CAN API index.
     */
    public int getApiIndex() {
        return CanandUtils.getApiIndex(id);
    }

    /**
     * Gets the 6-bit CAN Device id.
     * 
     * This is the user-adjustible "CAN Id" of the associated CAN device in question.
     * @return the device id.
     */
    public int getDeviceId() {
        return CanandUtils.getDeviceId(id);
    }

    /**
     * Gets the 5-bit Product ID / API class.
     * 
     * Product ID/ device type combinations will be unique to a Redux product.
     * @return the product id.
     */
    public int getApiPage() {
        return CanandUtils.getApiPage(id);
    }

    /**
     * Gets the 5-bit device type code
     * 
     * Product ID/ device type combinations will be unique to a Redux product.
     * @return the device type code.
     */
    public int getDeviceType() {
        return CanandUtils.getDeviceType(id);
    }

    /**
     * Gets the CAN message timestamp, in seconds.
     * The time base is relative to the FPGA timestamp.
     * @return timestamp in seconds.
     */
    public double getTimestamp() {
        return timestamp;
    }

    /**
     * Gets the CAN message payload (up to 8 bytes).
     * 
     * The length of the array does not correspond to the actual length of valid data. 
     * To fetch the associated data length code, use {@link #getLength()}
     * @return array of bytes that is 8 bytes long.
     */
    public byte[] getData() {
        return data;
    }

    /**
     * Gets the CAN message payload as a long.
     * The length of the array does not correspond to the actual length of valid data. 
     * This will also not work for FD packets >8 bytes long.
     * To fetch the associated data length code, use {@link #getLength()}
     * @return long
     */
    public long getDataAsLong() {
        return CanandUtils.bytesToLong(data);
    }

    /**
     * Gets the length of the CAN message's data in bytes.
     * @return length (1-8)
     */
    public int getLength() {
        return length;
    }

    /**
     * Gets an object representing the CAN bus that received the message
     * @return bus object
     */
    public MessageBus getBus() {
        return bus;
    }
}
