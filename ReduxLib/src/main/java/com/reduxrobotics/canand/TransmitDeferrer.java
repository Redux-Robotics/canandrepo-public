// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.List;

/**
 * Class that allows for batching messages to be transmitted.
 * 
 * Batching allows for multiple messages to go over the JNI barrier in a single call.
 * 
 * This does not work with calls that wait on a response!
 */
public class TransmitDeferrer {
    /** ctor */
    public TransmitDeferrer() {}
    // ScopedValue would probably be more correct here but ThreadLocal is the next best thing
    private static final ThreadLocal<TransmitDeferrer> instance = new ThreadLocal<>() {
        @Override
        protected TransmitDeferrer initialValue() {
            return new TransmitDeferrer();
        }
    };

    // Whether or not the batcher is active on the current thread
    private boolean active = false;

    private List<CanandMessage> messagesToSend = new ArrayList<>();
    private int messagesToSendCount = 0;
    private ByteBuffer buf = ReduxJNI.allocateMessageBuffer(8);

    /**
     * Returns whether or not the current thread is collecting all message transmits into the
     * local MessageBatcher.
     * 
     * @return true if active, false if not
     */
    public static boolean isActive() {
        return instance.get().active;
    }

    /**
     * Queues a CAN message to be sent in the batch.
     * 
     * Does nothing if the active flag is not set.
     * 
     * @param bus the bus to send to
     * @param messageID the message id
     * @param data the data to transmit
     */
    public static void queueCANMessage(MessageBus bus, int messageID, byte[] data) {
        CanandMessage msg = getNextMessage();
        if (msg == null) return;
        msg.updateFromData(bus, messageID, data);
    }

    /**
     * Queues a CAN message to be sent in the batch.
     * 
     * Does nothing if the active flag is not set.
     * 
     * @param bus the bus to send to
     * @param messageID the message id
     * @param data the data to transmit
     * @param length the length of data to send.
     */
    public static void queueCANMessage(MessageBus bus, int messageID, long data, int length) {
        CanandMessage msg = getNextMessage();
        if (msg == null) return;
        msg.updateFromLong(bus, messageID, data, length);
    }

    private static CanandMessage getNextMessage() {
        TransmitDeferrer inst = instance.get();
        if (!inst.active) return null;
        CanandMessage msg;
        if (inst.messagesToSendCount == inst.messagesToSend.size()) {
            msg = new CanandMessage();
            inst.messagesToSend.add(msg);
        } else {
            msg = inst.messagesToSend.get(inst.messagesToSendCount);
        }
        inst.messagesToSendCount++;
        return msg;
    }



    /**
     * Clears the internal message buffer.
     * 
     * The internal List holding the buffer of messasges to send is not shrinked automatically to
     * avoid garbage collection as presumably {@link #deferTransmit(Runnable)} is used in a loop,
     * with a similar number of CAN message sends each time. 
     * 
     * <p>This method frees that memory, in the case that this somehow becomes an issue. If called
     * inside the Runnable, no messages will be sent when it returns.
     */
    public static void clearMessageBuffer() {
        TransmitDeferrer inst = instance.get();
        inst.messagesToSend.clear();
        inst.messagesToSendCount = 0;
        inst.buf = ReduxJNI.allocateMessageBuffer(8);
    }

    /**
     * Batches Redux device messages writes in the passed Runnable into a single JNI-passing call.
     * 
     * <b>Settings sets must have timeouts set to 0 to function correctly!</b> This is because since
     * message sending is deferred, no setting confirmation or fetch will return.
     * 
     * @param func Function to batch calls for
     * @return non-negative on success, negative for error
     */
    public static int deferTransmit(Runnable func) {
        // Set the active state to true
        TransmitDeferrer inst = instance.get();
        inst.active = true;

        // Run the lambda
        func.run();
        inst.active = false;

        // If the current byte buf is too small, allocate a new one.
        // Hopefully it's big enough.

        if (ReduxJNI.messageCountBufferCanHold(inst.buf) < inst.messagesToSendCount) {
            inst.buf = ReduxJNI.allocateMessageBuffer(inst.messagesToSendCount);
        }
        
        // Clear and order the buffer.
        inst.buf.clear();
        inst.buf.order(ByteOrder.LITTLE_ENDIAN);

        // Write the buffer
        CanandMessage msg;
        for (int i = 0; i < inst.messagesToSendCount; i++) {
            msg = inst.messagesToSend.get(i);
            msg.writeToByteBuf(inst.buf);
        }

        // Finally transmit the data
        int status = ReduxJNI.batchEnqueueCANMessages(inst.buf, inst.messagesToSendCount);
        inst.messagesToSendCount = 0;
        return status;
    }
}
