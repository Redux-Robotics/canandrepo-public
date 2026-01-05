// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>
#include "redux/sensors/canandgyro/CanandgyroDetails.h"

namespace redux::sensors::canandgyro {

/**
 * A class to hold device faults for the Canandmag, as returned by Canandmag::GetActiveFaults and Canandmag::GetStickyFaults
 */
class CanandgyroFaults {
    public:
    /**
     * Constructs from a fault field.
     * @param field the fault bitfield
     * @param valid whether data is valid or not
    */
    constexpr CanandgyroFaults(uint8_t field, bool valid) :
        powerCycle(field & details::types::Faults::kPowerCycle),
        canIdConflict(field & details::types::Faults::kCanIdConflict),
        canGeneralError(field & details::types::Faults::kCanGeneralError),
        outOfTemperatureRange(field & details::types::Faults::kOutOfTemperatureRange),
        hardwareFault(field & details::types::Faults::kHardwareFault),
        calibrating(field & details::types::Faults::kCalibrating),
        angularVelocitySaturation(field & details::types::Faults::kAngularVelocitySaturation),
        accelerationSaturation(field & details::types::Faults::kAccelerationSaturation),
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
     * Returns the CAN general error flag, which will raise if the device has encountered a bus fault.
     * This typically indicates a physical wiring issue on the robot, such as loose connections or 
     * an intermittently shorting CAN bus
     */
    bool canGeneralError;

    /**
     * The temperature range flag, which will raise if the device is not between 0-95 degrees Celsius.
     * This may be of concern if the encoder is near very active motors.
     */
    bool outOfTemperatureRange;

    /**
     * The hardware fault flag, which will raise if a hardware issue is detected.
     * Generally will raise if the device's controller cannot read the physical sensor itself.
     */
    bool hardwareFault;

    /**
     * Returns the calibrating flag, which will raise if the device is currently calibrating.
     */
    bool calibrating;

    /**
     * The angular velocity saturation flag, which triggers on saturation of angular velocity.
     */
    bool angularVelocitySaturation;

    /**
     * The acceleration saturation flag, which triggers on saturation of acceleration.
     */
    bool accelerationSaturation;

    /**
     * Flag if any faults data has been received at all from the encoder. This will be false until the first status frame arrives
     * after either the start of robot code or after ClearStickyFaults is called.
     */
    bool faultsValid;
};
}