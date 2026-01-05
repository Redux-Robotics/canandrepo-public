// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>
#include <vector>

/**
 * Namespace for Canandmag-specific constants and details not generally needed by end users
 */
namespace redux::sensors::canandmag::details {
// constants_compiler: begin vendordep_Canandmag_cpp

/** Canandmag-specific CAN message IDs*/
class Message {
public:
enum : uint8_t {
    /** Message id for Position frame */
    kPositionOutput     = 0x1F,
    /** Message id for Velocity frame */
    kVelocityOutput     = 0x1E,
    /** Message id for Raw position frame */
    kRawPositionOutput  = 0x1D,

    // common to all devices

    /** Message id for setting control command */
    kSettingCommand     = 0x2,
    /** Message id for update setting on device */
    kSetSetting         = 0x3,
    /** Message id for setting value report from device */
    kReportSetting      = 0x4,
    /** Message id for clear device sticky faults */
    kClearStickyFaults  = 0x5,
    /** Message id for status frames */
    kStatus             = 0x6,
    /** Message id for party mode */
    kPartyMode          = 0x7,
};};

/** Setting IDs valid for Canandmag */
class Setting {
public:
enum : uint8_t {
    /** Setting msg id for Encoder zero offset */
    kZeroOffset               = 0xFF,
    /** Setting msg id for Velocity window width (value*250us) */
    kVelocityWindow           = 0xFE,
    /** Setting msg id for Position frame period (ms) */
    kPositionFramePeriod      = 0xFD,
    /** Setting msg id for Velocity frame period (ms) */
    kVelocityFramePeriod      = 0xFC,
    /** Setting msg id for Raw position frame period (ms) */
    kRawPositionFramePeriod   = 0xFB,
    /** Setting msg id for Invert direction (use cw instead of ccw) */
    kInvertDirection          = 0xFA,
    /** Setting msg id for Relative position value */
    kRelativePosition         = 0xF9,
    /** Setting msg id for Disable the zero button */
    kDisableZeroButton        = 0xF8,
    /** Setting msg id for status frame period (ms) */
    kStatusFramePeriod        = 0x4
};};

/** Canandmag-specific setting command IDs*/
class SettingCommand {
public:
enum : uint8_t {
      /** Setting command id for Fetch all settings from device */
    kFetchSettings        = 0x0,
    /** Setting command id for Reset everything to factory default */
    kResetFactoryDefault = 0x1,
    /** setting command for Fetch individual setting */
    kFetchSettingValue = 0x2,
    /** Setting command id for Factory defaults, but keep the encoder zero offset */
    kResetFactoryDefaultKeepZero = 0xFF,
}; };

//constants_compiler: end

/** std::vector of relevant settings IDS for the vendordep*/
const std::vector<uint8_t> VDEP_SETTINGS = { 
  Setting::kStatusFramePeriod,
  Setting::kZeroOffset,
  Setting::kVelocityWindow,
  Setting::kPositionFramePeriod,
  Setting::kVelocityFramePeriod,
  Setting::kInvertDirection,
  Setting::kDisableZeroButton
};
}