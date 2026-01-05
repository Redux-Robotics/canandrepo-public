// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandmag.h"
#include <stdexcept>

namespace redux::sensors::canandmag {
    // canandmagsettings impl
    const std::vector<uint8_t>& CanandmagSettings::SettingAddresses() const {
        return details::VDEP_SETTINGS; 
    }

    void CanandmagSettings::SetVelocityFilterWidth(units::millisecond_t widthMs) {
        if (widthMs < 0.25_ms || widthMs > 63.75_ms) { throw std::out_of_range("velocity widthMs must be between 0.25_ms and 63.75_ms");}
        uint8_t width = (uint8_t) (widthMs.to<double>() * 4);
        values[details::Setting::kVelocityWindow] = width;
    }

    void CanandmagSettings::SetPositionFramePeriod(units::second_t period) {
        if (period < 0_ms || period > 65535_ms) { throw std::out_of_range("period must be between 0_s and 65.535_s");}
        values[details::Setting::kPositionFramePeriod] = period.convert<units::milliseconds>().to<uint16_t>();
    }

    void CanandmagSettings::SetVelocityFramePeriod(units::second_t period) {
        if (period < 0_ms || period > 65535_ms) { throw std::out_of_range("period must be between 0_s and 65.535_s");}
        values[details::Setting::kVelocityFramePeriod] = period.convert<units::milliseconds>().to<uint16_t>();
    }


    void CanandmagSettings::SetStatusFramePeriod(units::second_t period) {
        if (period < 1_ms || period > 16383_ms) { throw std::out_of_range("period must be between 0.001_s and 16.383_s");}
        values[details::Setting::kStatusFramePeriod]  = period.convert<units::milliseconds>().to<uint16_t>();
    }

    void CanandmagSettings::SetInvertDirection(bool invert) {
        values[details::Setting::kInvertDirection] = invert;
    }

    void CanandmagSettings::SetDisableZeroButton(bool disable) {
        values[details::Setting::kDisableZeroButton] = disable;
    }

    void CanandmagSettings::SetZeroOffset(units::turn_t offset) {
        if (offset < 0_deg || offset >= 1_tr) { throw std::out_of_range("offset must be between 0 rotations inclusive and 1 rotations exclusive"); }
        uint16_t newPos = ((uint16_t) (offset.to<double>() * Canandmag::kCountsPerRotation));
        values[details::Setting::kZeroOffset] = newPos;
    }

    std::optional<units::millisecond_t> CanandmagSettings::GetVelocityFilterWidth() {
        if (!values.contains(details::Setting::kVelocityWindow)) return std::nullopt;
        return std::optional<units::millisecond_t>{(values[details::Setting::kVelocityWindow] & 0xff) / 4.0f};
    }

    std::optional<units::second_t> CanandmagSettings::GetPositionFramePeriod() {
        if (!values.contains(details::Setting::kPositionFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[details::Setting::kPositionFramePeriod]};
    }

    std::optional<units::second_t> CanandmagSettings::GetVelocityFramePeriod() {
        if (!values.contains(details::Setting::kVelocityFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[details::Setting::kVelocityFramePeriod]};
    }

    std::optional<units::second_t> CanandmagSettings::GetStatusFramePeriod() {
        if (!values.contains(details::Setting::kStatusFramePeriod)) return std::nullopt;
        return std::optional<units::millisecond_t>{values[details::Setting::kStatusFramePeriod]};
    }

    std::optional<bool> CanandmagSettings::GetInvertDirection() {
        if (!values.contains(details::Setting::kInvertDirection)) return std::nullopt;
        return std::optional<bool>{values[details::Setting::kInvertDirection] != 0};
    }

    std::optional<bool> CanandmagSettings::GetDisableZeroButton() {
        if (!values.contains(details::Setting::kDisableZeroButton)) return std::nullopt;
        return std::optional<bool>{values[details::Setting::kDisableZeroButton] != 0};
    }

    std::optional<units::turn_t> CanandmagSettings::GetZeroOffset() {
        if (!values.contains(details::Setting::kZeroOffset)) return std::nullopt;
        return units::turn_t{values[details::Setting::kZeroOffset] / Canandmag::kCountsPerRotation};
    }
}