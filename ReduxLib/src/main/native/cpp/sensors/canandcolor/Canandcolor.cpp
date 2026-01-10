// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include <algorithm>
#include <cstring>

#include "redux/sensors/canandcolor/Canandcolor.h"
#include "redux/canand/CanandEventLoop.h"
#include "hal/FRCUsageReporting.h"

namespace redux::sensors::canandcolor {
  using namespace details;

Canandcolor::Canandcolor(int canID, std::string bus) :
    proximity_{0.0, 0_s},
    color_{ColorData{0, 0, 0}, 0_s},
    digout_{DigoutSlotState{}, 0_s},
    status_{CanandcolorStatus{0, 0, false, 30_degC}, 0_s},
    stg_{*this},
    addr_{redux::canand::MessageBus::ByBusString(bus), 6, static_cast<uint8_t>(canID & 0x3f)},
    dataRecvOnce_{false},
    lastMessageTime_{0_s} {

  canand::AddCANListener(this);
  HAL_Report(HALUsageReporting::kResourceType_Redux_future2, canID + 1);
}

Canandcolor::~Canandcolor() {
  canand::RemoveCANListener(this);
}

double Canandcolor::GetProximity() {
  return proximity_.GetValue();
}

double Canandcolor::GetRed() {
  return color_.GetValue().red;
}

double Canandcolor::GetGreen() {
  return color_.GetValue().green;
}

double Canandcolor::GetBlue() {
  return color_.GetValue().blue;
}

double Canandcolor::GetHSVHue() {
  return color_.GetValue().GetHSVHue();
}

double Canandcolor::GetHSVSaturation() {
  return color_.GetValue().GetHSVSaturation();
}

double Canandcolor::GetHSVValue() {
  return color_.GetValue().GetHSVValue();
}

ColorData Canandcolor::GetColor() {
  return color_.GetValue();
}

DigoutSlotState Canandcolor::GetDigoutState() {
  return digout_.GetValue();
}

CanandcolorFaults Canandcolor::GetStickyFaults() {
  return status_.GetValue().stickyFaults;
}

CanandcolorFaults Canandcolor::GetActiveFaults() {
  return status_.GetValue().activeFaults;
}

units::celsius_t Canandcolor::GetTemperature() {
  return status_.GetValue().temperature;
}

CanandcolorStatus Canandcolor::GetStatus() {
  return status_.GetValue();
}

void Canandcolor::ClearStickyFaults() {
  uint8_t data[] = {0};
  SendCANMessage(msg::kClearStickyFaults, data, 0);
}

void Canandcolor::ClearStickyDigoutFlags() {
  uint8_t data[] = {0};
  SendCANMessage(msg::kClearStickyDigout, data, 0);
}

void Canandcolor::SetPartyMode(uint8_t level) {
  if (level > 10) level = 10;
  uint8_t data[] = {level};
  SendCANMessage(msg::kPartyMode, data, 1);
}

CanandcolorSettings Canandcolor::GetSettings(units::second_t timeout, units::second_t missingTimeout, uint32_t attempts) {
  return stg_.GetSettings(timeout, missingTimeout, attempts);
}

void Canandcolor::StartFetchSettings() {
  stg_.StartFetchSettings();
}

CanandcolorSettings Canandcolor::GetSettingsAsync() {
  return stg_.GetKnownSettings();
}

CanandcolorSettings Canandcolor::SetSettings(CanandcolorSettings& settings, units::second_t timeout, uint32_t attempts) {
  return stg_.SetSettings(settings, timeout, attempts);
}

CanandcolorSettings Canandcolor::ResetFactoryDefaults(units::second_t timeout) {
  return stg_.SendReceiveSettingCommand(details::types::SettingCommand::kResetFactoryDefault, timeout, true);
}

void Canandcolor::SetLampLEDBrightness(double brightness) {
  brightness = std::clamp(brightness, 0.0, 1.0);
  stg_.SetSettingById(setting::kLampBrightness, static_cast<uint64_t>(brightness * 36000), 0);
}

void Canandcolor::HandleMessage(redux::canand::CanandMessage& msg) {
  uint64_t dataLong = 0;
  memcpy(&dataLong, msg.GetData(), msg.GetLength());
  dataRecvOnce_ = true;
  units::second_t ts = msg.GetTimestamp();

  switch(msg.GetApiIndex()) {
    case msg::kDistanceOutput: {
      if (msg.GetLength() != 2) break;
      auto proxPacket = msg::DistanceOutput::decode(dataLong);
      proximity_.Update(proxPacket.distance / 65535.0, ts);
      break;
    }

    case msg::kColorOutput: {
      if (msg.GetLength() != 8) break;
      color_.Update(ColorData::FromColorMessage(msg::ColorOutput::decode(dataLong)), ts);
      break;
    }

    case msg::kDigitalOutput: {
      if (msg.GetLength() != 5) break;
      digout_.Update(DigoutSlotState::FromMsg(details::msg::DigitalOutput::decode(dataLong)), ts);
      break;
    }

    case msg::kStatus: {

      if (msg.GetLength() != 8) break;
      auto statusPacket = msg::Status::decode(dataLong);
      status_.Update(CanandcolorStatus{
          statusPacket.faults,
          statusPacket.sticky_faults,
          true,
          units::celsius_t{static_cast<double>(statusPacket.temperature) / 256.0}
      }, ts);
      break;
    }

    case msg::kReportSetting: {
      stg_.HandleSetting(msg);
      break;
    }

    default:
      break;
  }
}

redux::canand::CanandAddress& Canandcolor::GetAddress() {
  return addr_;
}

std::string Canandcolor::GetDeviceClassName() {
  return "Canandcolor";
}

redux::canand::CanandFirmwareVersion Canandcolor::GetMinimumFirmwareVersion() {
  return {2024, 0, 0};
}

}  // namespace redux::sensors::canandcolor
