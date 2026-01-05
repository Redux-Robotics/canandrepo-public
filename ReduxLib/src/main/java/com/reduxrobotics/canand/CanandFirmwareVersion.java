// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

/**
 * Represents a firmware version associated with a Redux product.
 * 
 * @param year the year associated with the firmware version. 
 *     Within a year/season, the message API is expected to remain the same until the postseason.
 * @param minor the minor number associated with the firmware version
 * @param patch the patch number associated with the firmware version
 */
public record CanandFirmwareVersion(int year, int minor, int patch) implements Comparable<CanandFirmwareVersion> {

    /**
     * Returns a new CanandFirmwareVersion generated from setting data.
     * 
     * @param data setting data, as a long
     */
    public CanandFirmwareVersion(long data) {
        this((int) ((data >> 16) & 0xffff), (int) ((data >> 8) & 0xff), (int) (data & 0xff));
    }

    /**
     * Serializes the firmware version record into a wire-formattable Long.
     * 
     * @return long representing the setting data.
     */
    public long toSettingData() {
        return (year << 16) | (minor << 8) | (patch);
    }

    @Override
    public int compareTo(CanandFirmwareVersion arg0) {
        CanandFirmwareVersion other = (CanandFirmwareVersion) arg0;
        return Long.compareUnsigned(this.toSettingData(), other.toSettingData());
    }

    @Override
    public String toString() {
        return String.format("v%d.%d.%d", year, minor, patch);
    }
}
