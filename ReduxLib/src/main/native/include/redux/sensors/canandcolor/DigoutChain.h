// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstddef>
#include <array>
#include "DigoutSlot.h"

namespace redux::sensors::canandcolor {

/**
 * Fixed-size container for a digital output logic chain.
 *
 * Chains are limited to 16 slots.
 */
class DigoutChain {
 public:

  /**
   * Creates a new empty chain.
   */
  constexpr DigoutChain();

  /**
   * Adds a slot to the chain (up to 16).
   * @param slot slot to add
   * @return reference to this chain
   */
  DigoutChain& Add(const DigoutSlot& slot);

  /**
   * Gets a slot by index.
   * @param index slot index
   * @return the slot at index, or DigoutSlot::Disabled() if out of range
   */
  DigoutSlot GetSlot(size_t index);

  /**
   * Gets the number of slots currently stored in the chain.
   * @return chain length
   */
  size_t GetLength();

 private:
  std::array<DigoutSlot, 16> slots{ DigoutSlot{} };
  size_t length = 0;
};

}  // namespace redux::sensors::canandcolor
