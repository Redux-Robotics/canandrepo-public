// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandgyro.h"
#include "redux/sensors/canandgyro/CanandgyroStruct.h"
#include "wpi/struct/Struct.h"

namespace wpi {

    ::redux::sensors::canandgyro::CanandgyroFaults Struct<::redux::sensors::canandgyro::CanandgyroFaults>::Unpack(std::span<const uint8_t> data) {
        auto b = wpi::UnpackStruct<uint8_t, 0>(data);
        return redux::sensors::canandgyro::CanandgyroFaults(b, true);
    }

    void Struct<redux::sensors::canandgyro::CanandgyroFaults>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::CanandgyroFaults& value) {
        data[0] = (
            (static_cast<uint8_t>(value.powerCycle)) |
            (static_cast<uint8_t>(value.canIdConflict) << 1) |
            (static_cast<uint8_t>(value.canGeneralError) << 2) |
            (static_cast<uint8_t>(value.outOfTemperatureRange) << 3) |
            (static_cast<uint8_t>(value.hardwareFault) << 4) |
            (static_cast<uint8_t>(value.calibrating) << 5) |
            (static_cast<uint8_t>(value.angularVelocitySaturation) << 6) |
            (static_cast<uint8_t>(value.accelerationSaturation) << 7)
        );
    }

    ::redux::sensors::canandgyro::CanandgyroStatus Struct<::redux::sensors::canandgyro::CanandgyroStatus>::Unpack(std::span<const uint8_t> data) {
        return redux::sensors::canandgyro::CanandgyroStatus(
            wpi::UnpackStruct<uint8_t, 0>(data),    
            wpi::UnpackStruct<uint8_t, 1>(data),    
            true,
            units::celsius_t{wpi::UnpackStruct<double, 2>(data)}
        );
    }

    void Struct<redux::sensors::canandgyro::CanandgyroStatus>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::CanandgyroStatus& value) {
        wpi::PackStruct<0>(data, value.activeFaults);
        wpi::PackStruct<1>(data, value.stickyFaults);
        wpi::PackStruct<2>(data, value.temperature.value());
    }

    ::redux::sensors::canandgyro::AngularVelocity Struct<::redux::sensors::canandgyro::AngularVelocity>::Unpack(std::span<const uint8_t> data) {
        return redux::sensors::canandgyro::AngularVelocity(
            units::turns_per_second_t{ wpi::UnpackStruct<double, 0>(data) },    
            units::turns_per_second_t{ wpi::UnpackStruct<double, 8>(data) },    
            units::turns_per_second_t{ wpi::UnpackStruct<double, 16>(data) } 
        );
    }

    void Struct<redux::sensors::canandgyro::AngularVelocity>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::AngularVelocity& value) {
        wpi::PackStruct<0>(data, value.Roll().value());
        wpi::PackStruct<8>(data, value.Pitch().value());
        wpi::PackStruct<16>(data, value.Yaw().value());
    }

    ::redux::sensors::canandgyro::Acceleration Struct<::redux::sensors::canandgyro::Acceleration>::Unpack(std::span<const uint8_t> data) {
        return redux::sensors::canandgyro::Acceleration(
            units::standard_gravity_t{ wpi::UnpackStruct<double, 0>(data) },    
            units::standard_gravity_t{ wpi::UnpackStruct<double, 8>(data) },    
            units::standard_gravity_t{ wpi::UnpackStruct<double, 16>(data) } 
        );
    }

    void Struct<redux::sensors::canandgyro::Acceleration>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::Acceleration& value) {
        wpi::PackStruct<0>(data, value.X().value());
        wpi::PackStruct<8>(data, value.Y().value());
        wpi::PackStruct<16>(data, value.Z().value());
    }

}