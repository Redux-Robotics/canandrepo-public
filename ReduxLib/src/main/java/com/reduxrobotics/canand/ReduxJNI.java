// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.concurrent.atomic.AtomicBoolean;

import edu.wpi.first.util.RuntimeLoader;


/**
 * Java side of the Redux device driver JNI wrapper.
 * 
 * It is generally not necessary to directly interact with this class.
 */
public class ReduxJNI {
  private ReduxJNI() {}
  /** Supported driver year */
  public static final int DRIVER_YEAR = 2026;

  /** Supported driver major version */
  public static final int DRIVER_MAJOR_VERSION = 1;

  /** Supported driver minor version */
  public static final int DRIVER_MINOR_VERSION = 1;

  static boolean libraryLoaded = false;

  static class WrongDriverVersionException extends RuntimeException {
    public WrongDriverVersionException(String msg) { super(msg); }
  }

  /**
   * Returned on failing JNI calls.
   */
  public static class ReduxJNIException extends RuntimeException {
    /**
     * ctor
     * @param msg message
     */
    public ReduxJNIException(String msg) { super(msg); }
  }

  /**
   * Internal use class.
   */
  public static class Helper {
    private Helper() {}
    private static AtomicBoolean extractOnStaticLoad = new AtomicBoolean(true);

    /**
     * Internal use method
     * @return should extract on static load
     */
    public static boolean getExtractOnStaticLoad() {
      return extractOnStaticLoad.get();
    }

    /**
     * Internal use method
     * @param load should load
     */
    public static void setExtractOnStaticLoad(boolean load) {
      extractOnStaticLoad.set(load);
    }
  }

  static {
    if (Helper.getExtractOnStaticLoad()) {
      try {
        RuntimeLoader.loadLibrary("reduxfifo");
      } catch (IOException ex) {
        ex.printStackTrace();
        System.exit(1);
      }
      libraryLoaded = true;
    }
  }

  /**
   * Force load the library.
   * @throws java.io.IOException thrown if the native library cannot be found
   */
  public static synchronized void forceLoad() throws IOException {
    if (libraryLoaded) {
      return;
    }
    RuntimeLoader.loadLibrary("reduxfifo");
    libraryLoaded = true;
  }

  private static boolean initialized = false;

  /**
   * Starts the Redux CANlink server -- not usually needed to be called manually.
   * @return 0 on success, nonzero otherwise
   */
  public static int init() {
    if(!initialized) {
      int ver = getDriverVersion();
      int yearVer = ((ver >> 16) & 0xffff);
      int majorVer = ((ver >> 8) & 0xff);
      int minorVer = (ver & 0xff);

      if ( yearVer != DRIVER_YEAR ||
           majorVer != DRIVER_MAJOR_VERSION ||
           minorVer != DRIVER_MINOR_VERSION) {
        throw new WrongDriverVersionException(String.format("ReduxCore version v%d.%d.%d does not match vendordep version v%d.%d.%d", 
            yearVer, majorVer, minorVer,
            DRIVER_YEAR, DRIVER_MAJOR_VERSION, DRIVER_MINOR_VERSION
        ));
      }

      initialize();
      initServer();
      Runtime.getRuntime().addShutdownHook(new Thread(() -> {
        stopServer();
      }));
      initialized = true;
    }
    return 0;
  }

  /**
   * Sends a CAN message.
   * (At the moment, this more or less calls HAL_CAN_SendMessage under the hood, and does not support sending to non-Rio buses.)
   * @param bus the bus to send on
   * @param messageID 29-bit full CAN message id
   * @param data payload of up to 8 bytes.
   * @return success
   */
  public static boolean sendCANMessage(MessageBus bus, int messageID, byte[] data) {
    if (data.length > 8)
      return false;
    
    if (TransmitDeferrer.isActive()) {
      TransmitDeferrer.queueCANMessage(bus, messageID, data);
      return true;
    }
    return enqueueCANMessage(bus.getDescriptor(), messageID, data) >= 0;
  }

  /**
   * Sends a CAN message using a long as the data medium.
   * (At the moment, this more or less calls HAL_CAN_SendMessage under the hood, and does not support sending to non-Rio buses.)
   * @param bus the bus to send on
   * @param messageID 29-bit full CAN message id
   * @param data payload of up to 8 bytes, formatted as a little-endian long.
   * @param length data payload length.
   * @return success
   */
  public static boolean sendCANMessage(MessageBus bus, int messageID, long data, int length) {
    if (length > 8)
      return false;
    
    if (TransmitDeferrer.isActive()) {
      TransmitDeferrer.queueCANMessage(bus, messageID, data, length);
      return true;
    }
    return enqueueCANMessageAsLong(bus.getDescriptor(), messageID, data, length) >= 0;
  }


  static ByteBuffer allocateMessageBuffer(int cnt) {
    return allocateBuffer(cnt);
  }

  static int messageCountBufferCanHold(ByteBuffer buf) {
    if (cachedSizeOfReduxCoreCANMessage < 0) {
      cachedSizeOfReduxCoreCANMessage = sizeofReduxCoreCANMessage();
    }
    return buf.capacity() / cachedSizeOfReduxCoreCANMessage;
  }

  static native int openLog(String path, int busId);
  static native int closeLog(int busId);

  private static native int initialize();
  private static native int initServer();
  private static native int stopServer();

  /**
   * <b>Don't use this function directly -- use {@link CanandDevice} with {@link CanandEventLoop} instead!!!</b> Blocks until a new CAN message is returned. 
   * <p>
   * Receives a CAN message direct into a byte buffer.
   * All CAN messages returned will be those under the Redux vendor id.
   * Messages returned will not be repeated elsewhere -- do not directly use this function unless you have good
   * reasons for doing so!!!!
   * </p>
   * @param buf Buffer to fill in
   * @return success or failure
   */
  static native int waitForCANMessage(ByteBuffer buf);
  private static native int enqueueCANMessage(int busId, int messageID, byte[] data);
  private static native int enqueueCANMessageAsLong(int busId, int messageID, long data, int length);

  /**
   * Batch enqueue messages.
   * 
   * @param buf direct byte buffer.
   * @param messageCount number of messages to send out of the buffer.
   * @return number of messages sent if non-negative, error code if negative
   */
  static native int batchEnqueueCANMessages(ByteBuffer buf, int messageCount);

  /**
   * Batch read messages.
   * 
   * @param buf direct byte buffer.
   * @param messageCount number of messages to send out of the buffer.
   * @return number of messages sent if non-negative, error code if negative
   */
  static native int batchWaitForCANMessage(ByteBuffer buf, int messageCount);

  private static int cachedSizeOfReduxCoreCANMessage = -1;
  private static native int sizeofReduxCoreCANMessage();

  private static native int getDriverVersion();

  /**
   * Allocates a new {@link ByteBuffer} sized and aligned for a certain number of CAN messages.
   */
  private static native ByteBuffer allocateBuffer(int messageCount);

  /**
   * Deallocates a {@link ByteBuffer} allocated by {@link #allocateBuffer}.
   * <h1>THIS IS INCREDIBLY UNSAFE TO USE!</h1>
   * Calling this twice on the same {@link ByteBuffer} <b>will</b> cause a double-free!
   * 
   * @param buffer buffer to deallocate
   */
  static native void deallocateBuffer(ByteBuffer buffer);

  /**
   * Opens or gets a bus by string index.
   * Will throw an exception if the bus string is invalid.
   * 
   * @param busAddress bus address.
   * @return bus address.
   */
  static native int openBusByString(String busAddress);

  /**
   * Opens or gets a bus by id.
   * 
   * @param busId bus address.
   * @return bus address.
   */
  static native int openBusById(int busId);

  static native long newRepeater();

  static native void updateRepeater(
    long repeaterHandle,
    int busId,
    int messageID,
    long data,
    int length,
    int periodMs,
    int times
  );

  static native void deallocateRepeater(long repeaterHandle);
}
