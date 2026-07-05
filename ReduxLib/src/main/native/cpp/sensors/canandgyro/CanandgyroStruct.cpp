// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandgyro.h"
#include "redux/sensors/canandgyro/CanandgyroStruct.h"
#include <wpi/util/struct/Struct.hpp>

namespace wpi::util {

    ::redux::sensors::canandgyro::CanandgyroFaults Struct<::redux::sensors::canandgyro::CanandgyroFaults>::Unpack(std::span<const uint8_t> data) {
        auto b = wpi::util::UnpackStruct<uint8_t, 0>(data);
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
            wpi::util::UnpackStruct<uint8_t, 0>(data),    
            wpi::util::UnpackStruct<uint8_t, 1>(data),    
            true,
            wpi::units::celsius_t{wpi::util::UnpackStruct<double, 2>(data)}
        );
    }

    void Struct<redux::sensors::canandgyro::CanandgyroStatus>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::CanandgyroStatus& value) {
        wpi::util::PackStruct<0>(data, value.activeFaults);
        wpi::util::PackStruct<1>(data, value.stickyFaults);
        wpi::util::PackStruct<2>(data, value.temperature.value());
    }

    ::redux::sensors::canandgyro::AngularVelocity Struct<::redux::sensors::canandgyro::AngularVelocity>::Unpack(std::span<const uint8_t> data) {
        return redux::sensors::canandgyro::AngularVelocity(
            wpi::units::turns_per_second_t{ wpi::util::UnpackStruct<double, 0>(data) },    
            wpi::units::turns_per_second_t{ wpi::util::UnpackStruct<double, 8>(data) },    
            wpi::units::turns_per_second_t{ wpi::util::UnpackStruct<double, 16>(data) } 
        );
    }

    void Struct<redux::sensors::canandgyro::AngularVelocity>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::AngularVelocity& value) {
        wpi::util::PackStruct<0>(data, value.Roll().value());
        wpi::util::PackStruct<8>(data, value.Pitch().value());
        wpi::util::PackStruct<16>(data, value.Yaw().value());
    }

    ::redux::sensors::canandgyro::Acceleration Struct<::redux::sensors::canandgyro::Acceleration>::Unpack(std::span<const uint8_t> data) {
        return redux::sensors::canandgyro::Acceleration(
            wpi::units::standard_gravity_t{ wpi::util::UnpackStruct<double, 0>(data) },    
            wpi::units::standard_gravity_t{ wpi::util::UnpackStruct<double, 8>(data) },    
            wpi::units::standard_gravity_t{ wpi::util::UnpackStruct<double, 16>(data) } 
        );
    }

    void Struct<redux::sensors::canandgyro::Acceleration>::Pack(
        std::span<uint8_t> data, const redux::sensors::canandgyro::Acceleration& value) {
        wpi::util::PackStruct<0>(data, value.X().value());
        wpi::util::PackStruct<8>(data, value.Y().value());
        wpi::util::PackStruct<16>(data, value.Z().value());
    }

}