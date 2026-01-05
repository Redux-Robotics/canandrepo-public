// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>
#include <chrono>
#include <units/time.h>
#include <bit>

/**
 * Series of utility functions for CAN messaging and bit manipulation.
 * 
 * For more information, see https://docs.wpilib.org/en/stable/docs/software/can-devices/can-addressing.html
 */
namespace redux::canand::utils {
    // This is the Redux CAN id.
    static constexpr uint8_t REDUX_CAN_ID = 14;

    /**
     * Extracts 5-bit device type code from a full message id
     * @param fullId the full 29-bit message id
     * @return the device type code
     */
    constexpr uint8_t getDeviceType(uint32_t fullId) {
    return (fullId >> 24) & 0x1f;
    }

    /**
     * Extracts 2-bit product id/API class from a full message id.
     * Instead of doing a 6bit/4bit split for api class/api index, we use 2bit/8bit.
     * 
     * @param fullId the full 29-bit message id
     * @return the product id code
     */
    constexpr uint8_t getApiPage(uint32_t fullId) {
        return (fullId >> 14) & 0x3;
    }

    /**
     * Extracts the 8-bit API index from a full message id.
     * Instead of doing a 6bit/4bit split for api class/api index, we use 2bit/8bit.
     * 
     * @param fullId the full 29-bit message id
     * @return the product id code
     */
    constexpr uint8_t getApiIndex(uint32_t fullId) {
        return (fullId >> 6) & 0xff;
    }

    /**
     * Extracts 6-bit device id from a full message id
     * This is the "CAN id" that end users will see and care about.
     * 
     * @param fullId the full 29-bit message id
     * @return the device CAN id
     */
    constexpr uint8_t getDeviceId(uint32_t fullId) {
        return fullId & 0x3f;
    }

    /**
     * Checks if a full CAN id will match against device type, product id, and device id
     * We use this to determine if a message is intended for a specific device.
     * 
     * @param idToCompare full 29-bit id
     * @param deviceType device id code
     * @param devId device id
     * @return whether the parameters matches the message id
     */
    constexpr bool idMatches(uint32_t idToCompare, uint8_t deviceType, uint8_t devId) {
        return (idToCompare & 0x1f00003fu) == (((uint32_t) deviceType << 24u) | devId);
    }

    /**
     * Construct a CAN message id to send to a Redux device.
     * 
     * @param deviceType the device id code
     * @param devId CAN device id
     * @param msgId API message id
     * @return a 29-bit full CAN message id
     */
    constexpr uint32_t constructMessageId(uint8_t deviceType, uint16_t devId, uint8_t msgId) {
        return (deviceType << 24) | (REDUX_CAN_ID << 16) | (msgId << 6) | (devId);
    }

    /**
     * Converts seconds from the units library to seconds in std::chrono::duration.
     * 
     * Useful for condition variables.
     * 
     * @param seconds in units::second_t terms
     * @return seconds in std::chrono::duration terms
     */
    constexpr std::chrono::duration<double, std::ratio<1LL, 1LL>> toChronoSeconds(units::second_t seconds) {
        return seconds.to<double>() * std::chrono::seconds(1);
    }

    /**
     * Converts a byte buffer to a little endian buffer.
     * 
     * @param dst destination ptr
     * @param src source ptr
     * @param len the length of the data in bytes. If less than 8, the unused bit-space will be zeros.
     */
    inline void memcpyLE(void* dst, void* src, size_t len) {

        uint8_t* dst8 = reinterpret_cast<uint8_t*>(dst);
        uint8_t* src8 = reinterpret_cast<uint8_t*>(src);
        // we are NOT supporting big-endian. if the MRC is big-endian i will start questioning everything
        if constexpr (std::endian::native == std::endian::little) {
#ifdef __linux__
            memcpy(dst, src, len);
#else
            for (size_t i = 0; i < len; i++) {
                dst8[i] = src8[i];
            }

#endif
        } else {
            for (size_t i = 0; i < len; i++) {
                dst8[len - i - 1] = src8[i];
            }
        }
    }

    /**
     * Extracts an unsigned integer up to 8 bits wide.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 8 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr uint8_t extractU8(uint64_t data, uint8_t width, uint8_t offset) {
        uint64_t mask = ((1ull) << width) - 1;
        return static_cast<uint8_t>((data >> offset) & mask);
    }

    /**
     * Extracts an unsigned integer up to 16 bits wide.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 16 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr uint16_t extractU16(uint64_t data, uint8_t width, uint8_t offset) {
        uint64_t mask = ((1ull) << width) - 1;
        return static_cast<uint16_t>((data >> offset) & mask);
    }

    /**
     * Extracts an unsigned integer up to 32 bits wide.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 32 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr uint32_t extractU32(uint64_t data, uint8_t width, uint8_t offset) {
        uint64_t mask = ((1ull) << width) - 1;
        return static_cast<uint32_t>((data >> offset) & mask);
    }

    /**
     * Extracts an unsigned integer up to 64 bits wide.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 64 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr uint64_t extractU64(uint64_t data, uint8_t width, uint8_t offset) {
        uint64_t mask = ((1ull) << width) - 1;
        return static_cast<uint64_t>((data >> offset) & mask);
    }


    /**
     * Extracts a signed integer up to 8 bits wide, performing a sign extension if necessary.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 8 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr int8_t extractI8(uint64_t data, uint8_t width, uint8_t offset) {
        const size_t BIT_WIDTH = (sizeof(int8_t) << 3);
        int8_t result = static_cast<int8_t>(extractU8(data, width, offset));
        if (width >= BIT_WIDTH) { return result; }
        uint8_t shift = (BIT_WIDTH - width);
        return (result << shift) >> shift;
    }

    /**
     * Extracts a signed integer up to 16 bits wide, performing a sign extension if necessary.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 16 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr int16_t extractI16(uint64_t data, uint8_t width, uint8_t offset) {
        const size_t BIT_WIDTH = (sizeof(int16_t) << 3);
        int16_t result = static_cast<int16_t>(extractU16(data, width, offset));
        if (width >= BIT_WIDTH) { return result; }
        uint8_t shift = (BIT_WIDTH - width);
        return (result << shift) >> shift;
    }

    /**
     * Extracts a signed integer up to 32 bits wide, performing a sign extension if necessary.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 32 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr int32_t extractI32(uint64_t data, uint8_t width, uint8_t offset) {
        const size_t BIT_WIDTH = (sizeof(int32_t) << 3);
        int32_t result = static_cast<int32_t>(extractU32(data, width, offset));
        if (width >= BIT_WIDTH) { return result; }
        uint8_t shift = (BIT_WIDTH - width);
        return (result << shift) >> shift;
    }

    /**
     * Extracts a signed integer up to 64 bits wide, performing a sign extension if necessary.
     * @param data bitfield to extract from
     * @param width width of integer in bits. Values larger than 64 are undefined behavior.
     * @param offset bit offset of the integer to extract
     * @return extracted integer
     */
    constexpr int64_t extractI64(uint64_t data, uint8_t width, uint8_t offset) {
        const size_t BIT_WIDTH = (sizeof(int64_t) << 3);
        int64_t result = static_cast<int64_t>(extractU64(data, width, offset));
        if (width >= BIT_WIDTH) { return result; }
        uint8_t shift = (BIT_WIDTH - width);
        return (result << shift) >> shift;
    }

    /**
     * Extracts a 24-bit float.
     * 
     * <p>24-bit floats have 1 sign bit, 8 exponent bits, and 15 mantissa bits. </p>
     * 
     * @param data bitfield to extract from
     * @param offset bit offset of the float to extract
     * @return extracted float
     */
    constexpr float extractF24(uint64_t data, uint8_t offset) {
        return std::bit_cast<float>((static_cast<uint32_t>(data >> offset) & 0xffffff) << 8);
    }


    /**
     * Extracts a 32-bit single-precision float.
     * 
     * @param data bitfield to extract from
     * @param offset bit offset of the float to extract
     * @return extracted float
     */
    constexpr float extractF32(uint64_t data, uint8_t offset) {
        return std::bit_cast<float>(static_cast<uint32_t>(data >> offset));
    }

    /**
     * Extracts a 64-bit double-precision float.
     * 
     * @param data bitfield to extract from
     * @return extracted float
     */
    constexpr double extractF64(uint64_t data) {
        return std::bit_cast<double>(data);
    }

    /**
     * Extracts a boolean from a bitfield.
     * @param data bitfield to extract from
     * @param offset the offset of the boolean bit in the bitfield
     * @return extracted boolean value.
     */
    constexpr bool extractBool(uint64_t data, uint8_t offset) { 
        return ((data >> offset) & 1) != 0;
    }

    /**
     * Packs an unsigned integer of variable length into a bitfield.
     * @param data the unsigned integer to pack
     * @param width the width of the unsigned integer
     * @param offset the 0-indexed offset of the integer in the bitfield
     * @return packed bitfield that can be ORed with other fields to construct a full bitfield
     */
    constexpr uint64_t packUInt(uint64_t data, uint8_t width, uint8_t offset) {
        uint64_t mask = ((1ull) << width) - 1;
        return (data & mask) << offset;
    }

    /**
     * Packs a signed integer of variable length into a bitfield as twos complement.
     * @param data the signed integer to pack
     * @param width the width of the signed integer
     * @param offset the 0-indexed offset of the integer in the bitfield
     * @return packed bitfield that can be ORed with other fields to construct a full bitfield
     */
    constexpr uint64_t packInt(int64_t data, uint8_t width, uint8_t offset) {
        uint64_t mask = ((1ull) << width) - 1;
        return (static_cast<uint64_t>(data) & mask) << offset;
    }

    /**
     * Packs a float into a 24-bit field. 
     * This is accomplished by ignoring the least significant 8 bits of the mantissa.
     * @param data the float to pack 
     * @param offset the 0-indexed offset of the float in the bitfield
     * @return packed bitfield that can be ORed with other fields to construct a full bitfield
     */
    constexpr uint64_t packF24(float data, uint8_t offset) {
        return static_cast<uint64_t>(std::bit_cast<uint32_t>(data) >> 8) << offset;
    }

    /**
     * Packs a float into a 32-bit field. 
     * @param data the float to pack 
     * @param offset the 0-indexed offset of the float in the bitfield
     * @return packed bitfield that can be ORed with other fields to construct a full bitfield
     */
    constexpr uint64_t packF32(float data, uint8_t offset) {
        return static_cast<uint64_t>(std::bit_cast<uint32_t>(data)) << offset;
    }

    /**
     * Packs a double into a 64-bit field. 
     * @param data the double to pack 
     * @return packed bitfield that can be ORed with other fields to construct a full bitfield
     */
    constexpr uint64_t packF64(double data) {
        return std::bit_cast<uint64_t>(data);
    }

    /**
     * Packs a boolean as a single bit into a 64-bit field.
     * @param data the boolean value
     * @param offset the 0-indexed offset of the bit in the bitfield
     * @return packed bitfield that can be ORed with other fields to construct a full bitfield
     */
    constexpr uint64_t packBool(bool data, uint8_t offset) {
        return static_cast<uint64_t>(data) << offset;
    }

    /**
     * Converts an enum to an underlying type
     * @param e value to convert
     * @return underlying type
     */
    template <typename E>
    constexpr typename std::underlying_type<E>::type to_underlying(E e) noexcept {
        return static_cast<typename std::underlying_type<E>::type>(e);
    }

}