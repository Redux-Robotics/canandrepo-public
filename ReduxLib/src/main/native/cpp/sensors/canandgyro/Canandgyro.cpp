// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/Canandgyro.h"
#include "redux/canand/CanandUtils.h"
#include "redux/frames/Frame.h"
#include "frc/Timer.h"
#include "hal/FRCUsageReporting.h"
#include <cmath>
#include <string>

#if !defined(M_PI)
// thanks msvc i love you so much
#define M_PI 3.14159265358979323846 /* pi */
#endif

int16_t quat2U16(double v) {
    int ret = (int) (v * 32767);
    if (ret > 32767) ret = 32767;
    if (ret < -32767) ret = -32767;
    return ret;
}

namespace redux::sensors::canandgyro {
    using namespace details;
    Canandgyro::Canandgyro(int canID, std::string bus) : stg(*this), addr(redux::canand::MessageBus::ByBusString(bus), 4, (uint8_t) (canID & 0x3f)) {
        redux::canand::AddCANListener(this);
        HAL_Report(HALUsageReporting::kResourceType_Redux_future3, canID + 1);
    }

    void Canandgyro::StartCalibration() {
        uint8_t data[8] = { 0 };
        SendCANMessage(msg::kCalibrate, data, 8);
        calibrating.Update(true, frc::Timer::GetFPGATimestamp());
    }

    bool Canandgyro::WaitForCalibrationToFinish(units::second_t timeout) {
        if (timeout <= 0_ms) { return !calibrating.GetValue(); }
        auto data = frames::WaitForFrames(timeout, this->calibrating);
        if (!data) { return false; }
        redux::frames::FrameData<bool> result;
        std::tie(result) = *data;
        return result.GetValue();
    }

    bool Canandgyro::SetPose(frc::Quaternion newPose, units::second_t timeout, uint32_t attempts) {
        newPose = newPose.Normalize();
        uint8_t idxToSet = (newPose.W() >= 0) ? setting::kSetPosePositiveW : setting::kSetPoseNegativeW;

        bool success = false;
        for (uint32_t i = 0; i < attempts && !success; i++) {
            success = stg.ConfirmSetSetting(idxToSet, types::QuatXyz{
                .x = quat2U16(newPose.X()),
                .y = quat2U16(newPose.Y()),
                .z = quat2U16(newPose.Z()),
            }.encode(), timeout, 0).IsValid();
        }
        return success;
    }

    bool Canandgyro::SetYaw(units::turn_t yaw, units::second_t timeout, uint32_t attempts) {

        // wraparounds counts how many times we've rotated past the plus/minus 180 degree point.
        // so to convert whole rotations into this format, we need to add/subtract 0.5 rotations so 
        // that fractional portions roll over properly at the boundary.
        double dyaw = yaw.to<double>();
        double offset = std::copysign(0.5, dyaw);
        dyaw += offset;
        int32_t wraparound = (int32_t) dyaw;
        dyaw = dyaw - (double) wraparound - offset;

        bool success = false;
        for (uint32_t i = 0; i < attempts && !success; i++) {
            success = stg.ConfirmSetSetting(
                setting::kSetYaw,
                setting::constructSetYaw(details::types::Yaw {
                    .yaw = (float) (dyaw * (M_PI * 2)),
                    .wraparound = (int16_t) wraparound,
                }),
                timeout,
                0
            ).IsValid();
        }
        return success;
    }

    void Canandgyro::ClearStickyFaults() {
        uint8_t dummy = 0;
        SendCANMessage(msg::kClearStickyFaults, &dummy, 1);
    }

    void Canandgyro::SetPartyMode(uint8_t level) {
        level = (level != 0);
        SendCANMessage(msg::kPartyMode, &level, 1);
    }

    void Canandgyro::HandleMessage(redux::canand::CanandMessage& msg) {
        uint64_t dataLong = 0;
        memcpy(&dataLong, msg.GetData(), msg.GetLength()); // buffer is guarenteed to be 8 bytes
        lastMessageTime = frc::Timer::GetFPGATimestamp();
        uint32_t dataLength = msg.GetLength();
        //uint8_t* data = msg.GetData();
        units::second_t ts = msg.GetTimestamp();

        switch(msg.GetApiIndex()) {
            case msg::kYawOutput: {
                if (dataLength != msg::YawOutput::DLC_MAX) break;
                auto yawPacket = msg::YawOutput::decode(dataLong).yaw;
                auto singleYawValue = units::radian_t{yawPacket.yaw};
                auto yawValue = singleYawValue + units::turn_t{static_cast<double>(yawPacket.wraparound)};
                this->multiYaw.Update(yawValue, ts);
                this->singleYaw.Update(singleYawValue, ts);
                break;
            }
            case msg::kAngularPositionOutput: {
                if (dataLength != msg::AngularPositionOutput::DLC_MAX) break;
                auto localQuat = msg::AngularPositionOutput::decode(dataLong);
                quat.Update(frc::Quaternion {
                    localQuat.w / 32767.0,
                    localQuat.x / 32767.0,
                    localQuat.y / 32767.0,
                    localQuat.z / 32767.0,
                }.Normalize(), ts);
                break;
            }
            case msg::kAngularVelocityOutput: {
                if (dataLength != msg::AngularVelocityOutput::DLC_MAX) break;
                auto localVel = msg::AngularVelocityOutput::decode(dataLong);

                vel.Update(AngularVelocity {
                    units::degrees_per_second_t{localVel.roll * 2000.0 / 32767.0},
                    units::degrees_per_second_t{localVel.pitch * 2000.0 / 32767.0},
                    units::degrees_per_second_t{localVel.yaw * 2000.0 / 32767.0},
                }, ts);

                break;
            }
            case msg::kAccelerationOutput: {
                if (dataLength != msg::AccelerationOutput::DLC_MAX) break;
                auto localAccel = msg::AccelerationOutput::decode(dataLong);
                accel.Update(Acceleration {
                    units::standard_gravity_t{localAccel.x * 16.0 / 32767.0},
                    units::standard_gravity_t{localAccel.y * 16.0 / 32767.0},
                    units::standard_gravity_t{localAccel.z * 16.0 / 32767.0},
                }, ts);
                break;
            }
            case msg::kCalibrationStatus: {
                calibrating.Update(false, ts);
                break;
            }

            case msg::kStatus: {
                if (dataLength != msg::Status::DLC_MAX) break;
                auto localStatus = msg::Status::decode(dataLong);

                status.Update(CanandgyroStatus{
                    localStatus.faults,
                    localStatus.sticky_faults,
                    true, 
                    units::celsius_t{static_cast<double>(localStatus.temperature) / 256.0 }
                }, ts);
                if (!status.GetValue().activeFaults.calibrating) {
                    calibrating.Update(false, ts);
                }
                break;
            }
            case msg::kReportSetting: {
                stg.HandleSetting(msg);
                break;
            }
            default:
            break;

        }

    }

    redux::canand::CanandAddress& Canandgyro::GetAddress() { return addr; }

}