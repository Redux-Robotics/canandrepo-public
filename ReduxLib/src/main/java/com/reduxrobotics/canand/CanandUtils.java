// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import java.util.BitSet;

import edu.wpi.first.hal.HALUtil;

/**
 * Series of utility functions for CAN messaging and bit manipulation.
 * 
 * For more information, see https://docs.wpilib.org/en/stable/docs/software/can-devices/can-addressing.html
 */
public class CanandUtils {
    private CanandUtils() {}
    // this is the Redux CAN id.
    private final static int REDUX_CAN_ID = 14;

    /**
     * Extracts 5-bit device type code from a full message id
     * @param fullId the full 29-bit message id
     * @return the device type code
     */
    public static int getDeviceType(int fullId) {
        return (fullId >> 24) & 0x1f;
    }

    /**
     * Extracts 2-bit product id/API class from a full message id
     * Instead of doing a 6bit/4bit split for api class/api index, we use an even 5 bit split.
     * 
     * @param fullId the full 29-bit message id
     * @return the product id code
     */
    public static int getApiPage(int fullId) {
        return (fullId >> 14) & 0x3;
    }

    /**
     * Extracts the 8-bit API index from a full message id.
     * Instead of doing a 6bit/4bit split for api class/api index, we use an even 5 bit split.
     * 
     * @param fullId the full 29-bit message id
     * @return the product id code
     */
    public static int getApiIndex(int fullId) {
        return (fullId >> 6) & 0xff;
    }

    /**
     * Extracts 6-bit device id from a full message id
     * This is the "CAN id" that end users will see and care about.
     * 
     * @param fullId the full 29-bit message id
     * @return the device CAN id
     */
    public static int getDeviceId(int fullId) {
        return fullId & 0x3f;
    }

    /**
     * Checks if a full CAN id will match against device type and device id
     * We use this to determine if a message is intended for a specific device.
     * 
     * @param idToCompare full 29-bit id
     * @param deviceType device id code
     * @param devId device id
     * @return whether the parameters matches the message id
     */
    public static boolean idMatches(int idToCompare, int deviceType, int devId) {
        return (idToCompare & 0x1f00003f) == ((deviceType << 24) | devId);
    }

    /**
     * Construct a CAN message id to send to a Redux device.
     * 
     * @param deviceType the device id code
     * @param devId CAN device id
     * @param pageId API page id
     * @param msgId API message id
     * @return a 29-bit full CAN message id
     */
    public static int constructMessageId(int deviceType, int devId, int pageId, int msgId) {
        return (deviceType << 24) | (REDUX_CAN_ID << 16) | (pageId << 14) | (msgId << 6) | (devId);
    }


     /**
      * Convert bytes to long without allocating new bytebuffers.
      *
      * @param data byte array
      * @return long made of the first 8 bytes
      */
    public static long bytesToLong(byte[] data) {
       long buf = 0;
       for (int i = 0; i < Math.min(data.length, 8); i++) {
            // if you do not mask like this negative bytes will bleed over.
            // i love not having unsigned types!
           buf |= ((long) data[i] & 0xff) << (i << 3);
       }

       return buf;
    }


    /**
     * Sign extends a {@link BitSet} to 64 bits by copying the bit from index {@code s-1} to 
     * {@code [s..64)}.
     * 
     * @param v the BitSet to sign extend.
     * @param s the index of the first bit to apply sign extension to. Bit {@code s-1} will be 
     *     sampled to determine the sign value.
     * @return v, the input BitSet
     */
    public static BitSet signExtend(BitSet v, int s) {
        v.set(s, 64, v.get(s-1));
        return v;
    }

    /**
     * Shorthand for {@code BitSet.toLongArray()[0]}, except zero length bitsets return 0.
     * 
     * @param v the input {@link BitSet}
     * @return v interpreted as a long
     */
    public static long bsToLong(BitSet v) {
        long[] a = v.toLongArray();
        return (a.length > 0) ? a[0] : 0;
    }

    /**
     * Converts a long to up to N bytes in a little-endian fashion.
     * @param value the value to convert
     * @param limit the limit of bytes to extract. Values greater than 8 are clamped to 8.
     * @return a byte array of the given length.
     */
    public static byte[] longToBytes(long value, int limit) {
        limit = Math.min(limit, 8);
        byte[] result = new byte[limit];
        for (int i = 0; i < limit; i++) {
            result[i] = (byte) ((value >> (i * 8)) & 0xff);
        }
        return result;
    }

    /**
     * Extracts a long from a {@link BitSet}, optionally sign-extending the output.
     * 
     * @param v input BitSet to extract from
     * @param start index of the first bit to extract
     * @param end index after the last bit to extract
     * @param signed whether the long is signed 
     * @return an extracted long
     */
    public static long extractLong(BitSet v, int start, int end, boolean signed) {
        BitSet s = v.get(start, end);
        if (signed) {
            signExtend(s, end - start);
        }

        return bsToLong(s);

    }

    /**
     * More scope-limited long-extracting helper that only works on up to 8-length byte arrays.
     * Use the BitSet version for values that are across 9 or more bytes.
     * 
     * @param v byte array that is 0-8 bytes long
     * @param start index of the first bit to extract
     * @param end index after the last bit to extract
     * @param signed whether to sign extend the result
     * @return an extracted long
     */
    public static long extractLong(byte[] v, int start, int end, boolean signed) {
        long res = bytesToLong(v);
        // remove lower bits
        res >>= start;
        int len = end - start;

        // remove upper bits
        long mask = (1L << (len)) - 1;
        res = res & mask;

        // perform sign extension
        if (signed && (res & (1L << (len - 1))) > 0) {
            res |= ~mask;
        }

        return res;
    }

    /**
     * Extracts a float from a byte array.
     * @param v byte array, length should be >= 4
     * @param bstart byte index to start reading from where bstart + 4 &lt; v.length
     * @return float value
     */
    public static float extractFloat(byte[] v, int bstart) {
        int buf = 0;
        for (int i = 0; i < 4; i++) {
            // you need the & 0xff because java always sign extends -_-
            buf |= (((int) v[i + bstart]) & 0xff) << (i << 3);
        }
        return Float.intBitsToFloat(buf);
    }

    /**
     * Writes a float to a byte array.
     * @param v the value to write
     * @param dest the byte array to write into
     * @param bstart index to start writing at
     */
    public static void writeFloatToBytes(float v, byte[] dest, int bstart) {
        int buf = Float.floatToIntBits(v);
        for (int i = 0; i < 4; i++) {
            dest[i + bstart] = (byte) (buf & 0xff);
            buf >>= 8;
        }
    }

    /**
     * Writes a float to a long.
     * @param v the value to write
     * @param bitstart bitindex to start writing at
     * @return shifted long
     */
    public static long floatToShiftedLong(float v, int bitstart) {
        return ((long) (Float.floatToIntBits(v)) & 0xffffffff) << bitstart;
    }

    /**
     * Converts a 32-bit float to a 24-bit representation with the least significant eight  bits of 
     * mantissa removed.
     * 
     * This properly conserves NaN/inf values as {@link Float#floatToIntBits} always coerces to the 
     * canonical represntations for NaNs.
     * 
     * @param v float value
     * @return 24-bit value as long
     */
    public static long floatToGuineaFloat24(float v) {
        return (long) ((Float.floatToIntBits(v) >> 8) & 0xffffff);
    }

    /**
     * Convert a 24-bit float in a long field to a 32-bit float.
     * @param v 24-bit float value
     * @return java float value
     */
    public static float extractFloatFromGuineaFloat24(long v) {
        return Float.intBitsToFloat((int) v << 8);
    }

    /**
     * Always returns the hardware FPGA timestamp.
     * 
     * <p> <i>Certain</i> popular libraries think it is really funny to replace 
     * {@link edu.wpi.first.wpilibj.Timer#getFPGATimestamp} with a version that only updates 
     * every 20 milliseconds.
     * 
     * <p> Unfortunately for us, this makes ReduxLib internals not function correctly, so this calls 
     * the JNI function directly to sidestep that shim.
     * 
     * @return the current time in seconds according to the FPGA
     */
    public static double getFPGATimestamp() {
        return HALUtil.getFPGATime() / 1000000.0;
    }
}
