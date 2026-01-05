// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandmag.h"
#include "redux/sensors/canandmag/CanandmagStruct.h"
#include "wpi/struct/Struct.h"

namespace wpi {

    ::redux::sensors::canandmag::CanandmagFaults Struct<::redux::sensors::canandmag::CanandmagFaults>::Unpack(std::span<const uint8_t> data) {
        auto b = wpi::UnpackStruct<uint8_t, 0>(data);
        return redux::sensors::canandmag::CanandmagFaults(b, true);
    }

    void Struct<redux::sensors::canandmag::CanandmagFaults>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandmag::CanandmagFaults& value) {
        data[0] = (
            (static_cast<uint8_t>(value.powerCycle)) |
            (static_cast<uint8_t>(value.canIdConflict) << 1) |
            (static_cast<uint8_t>(value.canGeneralError) << 2) |
            (static_cast<uint8_t>(value.outOfTemperatureRange) << 3) |
            (static_cast<uint8_t>(value.hardwareFault) << 4) |
            (static_cast<uint8_t>(value.magnetOutOfRange) << 5) |
            (static_cast<uint8_t>(value.underVolt) << 6)
        );
    }

    ::redux::sensors::canandmag::CanandmagStatus Struct<::redux::sensors::canandmag::CanandmagStatus>::Unpack(std::span<const uint8_t> data) {
        uint8_t active_faults = wpi::UnpackStruct<uint8_t, 0>(data);
        return redux::sensors::canandmag::CanandmagStatus(
            active_faults,
            wpi::UnpackStruct<uint8_t, 1>(data),    
            true,
            units::celsius_t{wpi::UnpackStruct<double, 2>(data)},
            (active_faults & (1 << 5)) == 0
        );
    }

    void Struct<redux::sensors::canandmag::CanandmagStatus>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandmag::CanandmagStatus& value) {
        wpi::PackStruct<0>(data, value.activeFaults);
        wpi::PackStruct<1>(data, value.stickyFaults);
        wpi::PackStruct<2>(data, value.temperature.value());
    }
}