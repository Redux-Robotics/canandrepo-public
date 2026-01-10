// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>

namespace redux::sensors::canandcolor {

/**
 * How a digital output slot combines with the next slot in a chain.
 */
enum class NextSlotAction : uint8_t {
  kTerminateChain = 0,
  kOrWithNextSlot = 1,
  kXorWithNextSlot = 2,
  kAndWithNextSlot = 3,
};

}  // namespace redux::sensors::canandcolor
