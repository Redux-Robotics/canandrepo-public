// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once

namespace redux::sensors::canandcolor {

/**
 * Proximity integration period selection.
 *
 * Values correspond to firmware-defined integration periods used by the proximity sensor.
 */
enum class ProximityPeriod {
  k3125us = 0x0,
  k6250us = 0x1,
  k12500us = 0x2,
  k25ms = 0x3,
  k50ms = 0x4,
  k100ms = 0x5,
  k200ms = 0x6,
  k400ms = 0x7,
  k800ms = 0x8,
};

}  // namespace redux::sensors::canandcolor
