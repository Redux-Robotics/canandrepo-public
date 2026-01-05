// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandgyro.h"
#include <stdexcept>

namespace redux::sensors::canandgyro {
    using namespace details;

    // canandcolorsettings impl
    const std::vector<uint8_t>& CanandgyroSettings::SettingAddresses() const {
        return setting::VDEP_SETTINGS; 
    }

    void CanandgyroSettings::SetYawFramePeriod(units::second_t period) {
        if (period < 0_ms || period > 65535_ms) { throw std::out_of_range("period must be between 0_s and 65.535_s");}
        values[setting::kYawFramePeriod] = period.convert<units::milliseconds>().to<uint16_t>();
    }

    void CanandgyroSettings::SetAngularPositionFramePeriod(units::second_t period) {
        if (period < 0_ms || period > 65535_ms) { throw std::out_of_range("period must be between 0_s and 65.535_s");}
        values[setting::kAngularPositionFramePeriod]  = period.convert<units::milliseconds>().to<uint16_t>();
    }

    void CanandgyroSettings::SetAngularVelocityFramePeriod(units::second_t period) {
        if (period < 0_ms || period > 65535_ms) { throw std::out_of_range("period must be between 0_s and 65.535_s");}
        values[setting::kAngularVelocityFramePeriod]  = period.convert<units::milliseconds>().to<uint16_t>();
    }

    void CanandgyroSettings::SetAccelerationFramePeriod(units::second_t period) {
        if (period < 0_ms || period > 65535_ms) { throw std::out_of_range("period must be between 0_s and 65.535_s");}
        values[setting::kAccelerationFramePeriod]  = period.convert<units::milliseconds>().to<uint16_t>();
    }

    void CanandgyroSettings::SetStatusFramePeriod(units::second_t period) {
        if (period < 1_ms || period > 16383_ms) { throw std::out_of_range("period must be between 0.001_s and 16.383_s");}
        values[setting::kStatusFramePeriod] = period.convert<units::milliseconds>().to<uint16_t>();
    }

    std::optional<units::second_t> CanandgyroSettings::GetYawFramePeriod() {
        if (!values.contains(setting::kYawFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[setting::kYawFramePeriod]};
    }

    std::optional<units::second_t> CanandgyroSettings::GetAngularPositionFramePeriod() {
        if (!values.contains(setting::kAngularPositionFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[setting::kAngularPositionFramePeriod]};
    }

    std::optional<units::second_t> CanandgyroSettings::GetAngularVelocityFramePeriod() {
        if (!values.contains(setting::kAngularVelocityFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[setting::kAngularVelocityFramePeriod]};
    }

    std::optional<units::second_t> CanandgyroSettings::GetAccelerationFramePeriod() {
        if (!values.contains(setting::kAccelerationFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[setting::kAccelerationFramePeriod]};
    }

    std::optional<units::second_t> CanandgyroSettings::GetStatusFramePeriod() {
        if (!values.contains(setting::kStatusFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[setting::kStatusFramePeriod]};
    }

}