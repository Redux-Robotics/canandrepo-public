package com.reduxrobotics.canand;

/**
 * This class can be instantiated to transmit digital state over CAN.
 */
public class DigitalStateSender implements AutoCloseable {
    private final Repeater sender;
    private MessageBus bus; 
    private long bitField;
    private int periodMs;

    private static final int BASE_MESSAGE_ID = 0x010e0000 | (31 << 6);

    /**
     * Constructor
     * @param bus bus
     * @param periodMs period to send at
     */
    public DigitalStateSender(MessageBus bus, int periodMs) {
        this.sender = new Repeater();
        this.bitField = 0;
        this.bus = bus;
        this.periodMs = periodMs;
    }

    private void update() {
        this.sender.update(bus.getDescriptor(), BASE_MESSAGE_ID, bitField, 2, periodMs, Integer.MAX_VALUE);
    }

    /**
     * Set state at index
     * @param index index
     * @param value value
     */
    public void setState(int index, boolean value) {
        if (index > 15) {
            return;
        }

        if (value) {
            bitField |= (1 << index);
        } else {
            bitField &= ~(1 << index);
        }
        update();
    }

    @Override
    public void close() throws Exception {
        sender.disarm();
    }

}
