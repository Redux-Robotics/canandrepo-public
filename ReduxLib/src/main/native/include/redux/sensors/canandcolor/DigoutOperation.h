// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>

namespace redux::sensors::canandcolor {

/**
 * Digital output slot operations.
 *
 * These are used when building digital output logic chains.
 */
enum class DigoutOperation : uint8_t {
  kEquals = 0x00,
  kLessThan = 0x01,
  kGreaterThan = 0x02,
  kLessThanOrEquals = 0x03,
  kGreaterThanOrEquals = 0x04,
  kPrevSlotTrue = 0x20,
  kPrevClauseTrue = 0x21,
};

}  // namespace redux::sensors::canandcolor
