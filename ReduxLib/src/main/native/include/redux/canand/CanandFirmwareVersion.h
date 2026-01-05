// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>

namespace redux::canand {

/**
 * Represents a firmware version associated with a Redux product.
 * 
 * @param year the year associated with the firmware version. 
 *             Within a year/season, the message API is expected to remain the same until the postseason.
 * @param minor the minor number associated with the firmware version
 * @param patch the patch number associated with the firmware version
 */
struct CanandFirmwareVersion {
    public:
    /**
     * Constructor.
     * 
     * @param year the year associated with the firmware version. 
     *             Within a year/season, the message API is expected to remain the same until the postseason.
     * @param minor the minor number associated with the firmware version
     * @param patch the patch number associated with the firmware version
     */
    constexpr CanandFirmwareVersion(uint16_t year, uint8_t minor, uint8_t patch): year{year}, minor{minor}, patch{patch} {};
    /** Firmware year. */
    uint16_t year;
    /** Firmware minor version. */
    uint8_t minor;
    /** Firmware patch version. */
    uint8_t patch;

    /**
     * Serializes the firmware version record into a wire-formattable Long.
     * 
     * @return long representing the setting data.
     */
    constexpr uint64_t ToSettingData() {
        return (year << 16) | (minor << 8) | (patch);
    }

    /**
     * Returns a new CanandFirmwareVersion generated from setting data.
     * 
     * @param value setting data, as a 48-bit long
     * @return CanandFirmwareVersion data
     */
    static constexpr CanandFirmwareVersion FromSettingData(uint64_t value) {
        return CanandFirmwareVersion{(uint16_t) (value >> 16), (uint8_t) ((value >> 8) & 0xff), (uint8_t) ((value & 0xff))};
    }
    
};

}