// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include <algorithm>

#include "redux/sensors/canandcolor/CanandcolorSettings.h"

namespace redux::sensors::canandcolor {
  using namespace details;

const std::vector<uint8_t>& CanandcolorSettings::SettingAddresses() const {
  return setting::VDEP_SETTINGS;
}

CanandcolorSettings& CanandcolorSettings::SetStatusFramePeriod(units::second_t period) {
  int ms = static_cast<int>(period.value() * 1000);
  ms = std::clamp(ms, 1, 16383);
  values[setting::kStatusFramePeriod] = ms;
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetProximityFramePeriod(units::second_t period) {
  int ms = static_cast<int>(period.value() * 1000);
  ms = std::clamp(ms, 0, 65535);
  values[setting::kDistanceFramePeriod] = ms;
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetColorFramePeriod(units::second_t period) {
  int ms = static_cast<int>(period.value() * 1000);
  ms = std::clamp(ms, 0, 65535);
  values[setting::kColorFramePeriod] = ms;
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetDigoutFramePeriod(units::second_t period) {
  int ms = static_cast<int>(period.value() * 1000);
  ms = std::clamp(ms, 0, 65535);
  values[setting::kDigoutFramePeriod] = ms;
  return *this;
}

std::optional<units::second_t> CanandcolorSettings::GetStatusFramePeriod() {
  auto it = values.find(setting::kStatusFramePeriod);
  if (it != values.end()) {
    return units::second_t(it->second / 1000.0);
  }
  return std::nullopt;
}

std::optional<units::second_t> CanandcolorSettings::GetProximityFramePeriod() {
  auto it = values.find(setting::kDistanceFramePeriod);
  if (it != values.end()) {
    return units::second_t(it->second / 1000.0);
  }
  return std::nullopt;
}

std::optional<units::second_t> CanandcolorSettings::GetColorFramePeriod() {
  auto it = values.find(setting::kColorFramePeriod);
  if (it != values.end()) {
    return units::second_t(it->second / 1000.0);
  }
  return std::nullopt;
}

std::optional<units::second_t> CanandcolorSettings::GetDigoutFramePeriod() {
  auto it = values.find(setting::kDigoutFramePeriod);
  if (it != values.end()) {
    return units::second_t(it->second / 1000.0);
  }
  return std::nullopt;
}

CanandcolorSettings& CanandcolorSettings::SetLampLEDBrightness(double brightness) {
  brightness = std::clamp(brightness, 0.0, 1.0);
  values[setting::kLampBrightness] = static_cast<uint64_t>(brightness * 36000);
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetColorIntegrationPeriod(ColorPeriod period) {
  values[setting::kColorIntegrationPeriod] = static_cast<uint64_t>(period);
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetProximityIntegrationPeriod(ProximityPeriod period) {
  values[setting::kDistanceIntegrationPeriod] = static_cast<uint64_t>(period);
  return *this;
}

std::optional<double> CanandcolorSettings::GetLampLEDBrightness() {
  auto it = values.find(setting::kLampBrightness);
  if (it != values.end()) {
    return it->second / 36000.0;
  }
  return std::nullopt;
}

std::optional<ColorPeriod> CanandcolorSettings::GetColorIntegrationPeriod() {
  auto it = values.find(setting::kColorIntegrationPeriod);
  if (it != values.end()) {
    return static_cast<ColorPeriod>(it->second);
  }
  return std::nullopt;
}

std::optional<ProximityPeriod> CanandcolorSettings::GetProximityIntegrationPeriod() {
  auto it = values.find(setting::kDistanceIntegrationPeriod);
  if (it != values.end()) {
    return static_cast<ProximityPeriod>(it->second);
  }
  return std::nullopt;
}

CanandcolorSettings& CanandcolorSettings::SetAlignProximityFramesToIntegrationPeriod(bool align) {
  values[setting::kDistanceExtraFrameMode] = align ? 1 : 0;
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetAlignColorFramesToIntegrationPeriod(bool align) {
  values[setting::kColorExtraFrameMode] = align ? 1 : 0;
  return *this;
}

std::optional<bool> CanandcolorSettings::GetAlignProximityFramesToIntegrationPeriod() {
  auto it = values.find(setting::kDistanceExtraFrameMode);
  if (it != values.end()) {
    return it->second != 0;
  }
  return std::nullopt;
}

std::optional<bool> CanandcolorSettings::GetAlignColorFramesToIntegrationPeriod() {
  auto it = values.find(setting::kColorExtraFrameMode);
  if (it != values.end()) {
    return it->second != 0;
  }
  return std::nullopt;
}

CanandcolorSettings& CanandcolorSettings::SetDigoutPinConfig(uint8_t digout, const DigoutPinConfig& config) {
  uint8_t addr = (digout == 0) ?
      setting::kDigout1OutputConfig :
      setting::kDigout2OutputConfig;
  values[addr] = config.ToOutputSettingData();
  return *this;
}

CanandcolorSettings& CanandcolorSettings::SetDigoutFrameTrigger(uint8_t digout, DigoutFrameTrigger trigger) {
  uint8_t addr = (digout == 0) ?
      setting::kDigout1MessageOnChange :
      setting::kDigout2MessageOnChange;
  values[addr] = static_cast<uint64_t>(trigger);
  return *this;
}

std::optional<uint64_t> CanandcolorSettings::GetDigoutPinConfig(uint8_t digout) {
  uint8_t addr = (digout == 0) ?
      setting::kDigout1OutputConfig :
      setting::kDigout2OutputConfig;
  auto it = values.find(addr);
  if (it != values.end()) {
    return it->second;
  }
  return std::nullopt;
}

std::optional<DigoutFrameTrigger> CanandcolorSettings::GetDigoutFrameTrigger(uint8_t digout) {
  uint8_t addr = (digout == 0) ?
      setting::kDigout1MessageOnChange :
      setting::kDigout2MessageOnChange;
  auto it = values.find(addr);
  if (it != values.end()) {
    return static_cast<DigoutFrameTrigger>(it->second);
  }
  return std::nullopt;
}

}  // namespace redux::sensors::canandcolor
