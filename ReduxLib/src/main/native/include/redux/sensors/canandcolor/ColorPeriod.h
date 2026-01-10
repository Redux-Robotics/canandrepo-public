// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once

namespace redux::sensors::canandcolor {

/**
 * Color integration period selection.
 *
 * Values correspond to firmware-defined integration periods used by the color sensor.
 */
enum class ColorPeriod {
  k1ms = 0x0,
  k2ms = 0x1,
  k4ms = 0x2,
  k8ms = 0x3,
  k16ms = 0x4,
  k32ms = 0x5,
  k64ms = 0x6,
  k128ms = 0x7,
  k256ms = 0x8,
  k512ms = 0x9,
  k1024ms = 0xa,
};

}  // namespace redux::sensors::canandcolor
