// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include <string.h>
#include "redux/canand/CanandDevice.h"
#include "redux/canand/CanandUtils.h"
#include "ReduxCore.h"
#include <chrono>
#include "frc/Timer.h"
#include "frc/Errors.h"
#include <fmt/format.h>


namespace redux::canand {


void CanandDevice::CheckReceivedFirmwareVersion() {
    std::lock_guard<std::mutex> guard(settingRecvLock);
    if (!receivedFirmwareVersion.has_value()) {
        // yell that the device may not be on bus
        FRC_ReportError(frc::err::Error, 
        "{} did not respond to a firmware version check"
        "-- is the device powered and connected to the robot?", 
        GetDeviceName());
        return;
    }

    CanandFirmwareVersion version = *receivedFirmwareVersion;
    CanandFirmwareVersion minVersion = GetMinimumFirmwareVersion();
    if (version.ToSettingData() < minVersion.ToSettingData()) {
        FRC_ReportError(frc::err::Error, 
        "{} is running too old firmware (v{}.{}.{}) < minimum v{}.{}.{})"
        "-- please update the device at to avoid unforeseen errors!",
        GetDeviceName(),
        version.year, version.minor, version.patch,
        minVersion.year, minVersion.minor, minVersion.patch);
    }
}



bool CanandDevice::IsConnected(units::second_t timeout) {
    if (!lastMessageTs.has_value()) { return false; }
    return (frc::Timer::GetFPGATimestamp() - lastMessageTs.value_or(0_ms)) <= timeout;
}

void CanandDevice::PreHandleMessage(CanandMessage& msg) {
    lastMessageTs = msg.GetTimestamp();
    if (msg.GetApiIndex() == details::Message::kReportSetting && msg.GetLength() >= 7) {
        uint8_t* data = msg.GetData();
        if (data[0] == details::Setting::kFirmwareVersion) {
            std::lock_guard<std::mutex> settingRecvGuard(settingRecvLock);
            receivedFirmwareVersion = CanandFirmwareVersion{*(uint16_t*) (data + 3), data[2], data[1]};
        }
    }
}

std::string CanandDevice::GetDeviceName() {
    return fmt::format("{}[device_id={}]", GetDeviceClassName(), GetAddress().GetDeviceId());
}

}