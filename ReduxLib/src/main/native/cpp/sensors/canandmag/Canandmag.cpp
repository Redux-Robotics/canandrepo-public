// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandmag.h"
#include "redux/canand/CanandEventLoop.h"
#include "redux/canand/CanandUtils.h"
#include "redux/canand/CanandDevice.h"
#include "redux/canand/MessageBus.h"
#include <wpi/system/Timer.hpp>
#include <wpi/hal/Errors.h>
#include <string.h>
#include <string>
#include <chrono>
#include <stdexcept>

namespace redux::sensors::canandmag {
    using namespace redux;
    Canandmag::Canandmag(int canID, std::string bus) : stg(*this), addr(redux::canand::MessageBus::ByBusString(bus), 7, (uint8_t (canID & 0x3f))) {
        canand::AddCANListener(this);
        canand::utils::reportUsage("Canandmag", bus, canID);
    }

    wpi::units::turn_t Canandmag::GetPosition() {
        return position.GetValue();
    }

    wpi::units::turn_t Canandmag::GetAbsPosition() {
        return absPosition.GetValue();
    }

    bool Canandmag::SetPosition(wpi::units::turn_t newPosition, wpi::units::second_t timeout) {
        if (newPosition < -131072_tr || newPosition >= 131072_tr) 
            throw std::out_of_range("new relative position is not in the range [-131072..131072) turns"); 
        int32_t newPos = (int32_t) (newPosition.to<double>() * kCountsPerRotation);
        return stg.ConfirmSetSetting(details::Setting::kRelativePosition, ((uint8_t*) &newPos), 4, timeout, 0).IsValid();
    }

    bool Canandmag::SetAbsPosition(wpi::units::turn_t newPosition, wpi::units::second_t timeout, bool ephemeral) {
        if (newPosition < 0_tr || newPosition >= 1_tr) 
            throw std::out_of_range("new relative position is not in the range [0.0..1.0) turns"); 
        
        if (!ephemeral && setAbsPositionWarning.feed()) {
            fmt::println(stderr,  
                ("Calling SetAbsPosition() at high frequency will quickly wear out the Canandmag's internal flash.\n"\
                "Consider either using SetPosition() instead or passing in ephemeral=true to not write to flash.")
            );
        }

        uint8_t flags = (ephemeral) ? redux::canand::SettingFlags::kEphemeral : 0;
        uint16_t newPos = ((uint16_t) (newPosition.to<double>() * kCountsPerRotation));
        uint8_t buf[3] = {static_cast<uint8_t>(newPos & 0xff), static_cast<uint8_t>((newPos >> 8) & 0x3f), 1};

        return stg.ConfirmSetSetting(details::Setting::kZeroOffset, buf, 3, timeout, flags).IsValid();
    }

    bool Canandmag::ZeroAll(wpi::units::second_t timeout) {
        return (SetPosition(0_tr, timeout) && SetAbsPosition(0_tr, timeout)) == 1;
    }

    wpi::units::turns_per_second_t Canandmag::GetVelocity() {
        return velocity.GetValue();
    }

    bool Canandmag::MagnetInRange() {
        return status.GetValue().magnetInRange;
    }

    CanandmagFaults Canandmag::GetStickyFaults() {
        return status.GetValue().stickyFaults;
    }

    CanandmagFaults Canandmag::GetActiveFaults() {
        return status.GetValue().activeFaults;
    }

    void Canandmag::ClearStickyFaults() {
        uint8_t data[] = {0};
        SendCANMessage(details::Message::kClearStickyFaults, data, sizeof(data));
        // reset status framedata such that faults are now invalid again
        status.Update(CanandmagStatus{0, 0, false, status.GetValue().temperature, status.GetValue().magnetInRange}, status.GetTimestamp());
    }

    wpi::units::celsius_t Canandmag::GetTemperature() {
        return status.GetValue().temperature;
    }

    void Canandmag::SetPartyMode(uint8_t level) {
        if (level > 10) 
            throw std::out_of_range("party level must be between 0 and 10 (inclusive)"); 
        uint8_t data[] = {level};
        SendCANMessage(details::Message::kPartyMode, data, sizeof(data));
    }

    void Canandmag::HandleMessage(redux::canand::CanandMessage& msg) {
        uint64_t dataLong = 0;
        memcpy(&dataLong, msg.GetData(), msg.GetLength()); // buffer is guarenteed to be 8 bytes
        dataRecvOnce = true;
        lastMessageTime = wpi::Timer::GetMonotonicTimestamp();
        uint32_t dataLength = msg.GetLength();
        uint8_t* data = msg.GetData();
        wpi::units::second_t ts = msg.GetTimestamp();
        
        switch(msg.GetApiIndex()) {
            case details::Message::kPositionOutput: {
                if (dataLength != 6) break;
                position.Update(wpi::units::turn_t{(*(int32_t*) data) / kCountsPerRotation}, ts);
                absPosition.Update(wpi::units::turn_t{((dataLong >> 34) & 0x3fff) / kCountsPerRotation}, ts);
                break;
            }
            case details::Message::kVelocityOutput: {
                if (dataLength != 3) break;
                int32_t tmp = (dataLong & 0x3fffff);
                velocity.Update(wpi::units::turns_per_second_t{ ((tmp << 10) >> 10) / kCountsPerRotationPerSecond}, ts);
                break;
            }
            case details::Message::kStatus: {
                if (dataLength != 8) break;
                status.Update(CanandmagStatus{data[0], data[1], true, wpi::units::celsius_t{static_cast<double>((int8_t) data[2])}, (data[0] & 0b100000) == 0}, ts);
                break;
            }
            case details::Message::kReportSetting: {
                stg.HandleSetting(msg);
                break;
            }
            default:
            break;

        }
    }

    redux::canand::CanandAddress& Canandmag::GetAddress() { return addr; }



}