// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>

namespace redux::sensors::canandmag {

/**
 * A class to hold device faults for the Canandmag, as returned by Canandmag::GetActiveFaults and Canandmag::GetStickyFaults
 * This class is immutable once constructed.
 */
class CanandmagFaults {
    public:
    /**
     * Constructs from a fault field.
     * @param field the fault bitfield
     * @param valid whether data is valid or not
    */
    constexpr CanandmagFaults(uint8_t field, bool valid) :
        powerCycle(field & 0b1),
        canIdConflict(field & 0b10),
        canGeneralError(field & 0b100),
        outOfTemperatureRange(field & 0b1000),
        hardwareFault(field & 0b10000),
        magnetOutOfRange(field & 0b100000),
        underVolt(field & 0b1000000),
        faultsValid(valid) {};

    /**
     * The power cycle fault flag, which is set to true when the encoder first boots.
     * Clearing sticky faults and then checking this flag can be used to determine if the encoder rebooted.
     */
    bool powerCycle;

    /**
     * The CAN ID conflict flag, which is set to true if there is a CAN id conflict.
     * In practice, you should physically inspect the encoder to ensure it's not flashing blue.
     */
    bool canIdConflict;

    /**
     * The CAN general error flag, which will raise if the encoder cannot RX packets reliably.
     * This is usually due to wiring issues, such as a shorted CAN bus
     */
    bool canGeneralError;

    /**
     * The temperature range flag, which will raise if the encoder is not between 0-70 degrees Celsius.
     * This may be of concern if the encoder is near very active motors.
     */
    bool outOfTemperatureRange;

    /**
     * The hardware fault flag, which will raise if a hardware issue is detected.
     * Generally will raise if the device's controller cannot read the physical sensor itself.
     */
    bool hardwareFault;

    /**
     * The magnet out of range flag, which will raise if the measured shaft's magnet is not detected.
     * This will match the encoder's LED shining red in normal operation.
     */
    bool magnetOutOfRange;

    /**
     * The undervolt flag, which will raise if the encoder is experiencing brownout conditions.
     */
    bool underVolt;

    /**
     * Flag if any faults data has been received at all from the encoder. This will be false until the first status frame arrives
     * after either the start of robot code or after ClearStickyFaults is called.
     */
    bool faultsValid;
};
}