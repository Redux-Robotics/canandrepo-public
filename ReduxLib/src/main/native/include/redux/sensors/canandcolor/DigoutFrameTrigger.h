// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once

namespace redux::sensors::canandcolor {

/**
 * Digital output frame trigger configuration.
 *
 * When enabled, the device sends a digital output frame when the corresponding output changes state.
 */
enum class DigoutFrameTrigger {
  kDisabled = 0x0,
  kRisingEdgeOnly = 0x1,
  kFallingEdgeOnly = 0x2,
  kRisingAndFalling = 0x3,
};

}  // namespace redux::sensors::canandcolor
