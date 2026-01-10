// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>
#include "CanandcolorDetails.h"

namespace redux::sensors::canandcolor {

/**
 * Decoded digital output state frame.
 *
 * Includes current output state, sticky flags, and per-slot condition bitfields.
 */
struct DigoutSlotState {
  /** Current digout1 output state. */
  bool digout1State;
  /** Current digout2 output state. */
  bool digout2State;
  /** Sticky flag for digout1 state changes. */
  bool digout1Sticky;
  /** Sticky flag for digout2 state changes. */
  bool digout2Sticky;
  /** Per-slot condition bitfield for digout1 logic chain. */
  uint16_t digout1Cond;
  /** Per-slot condition bitfield for digout2 logic chain. */
  uint16_t digout2Cond;

  /**
   * Gets the boolean state of an individual slot condition for digout1.
   * @param slotIndex slot index [0..15]
   * @return true if the slot condition is true
   */
  bool GetDigout1SlotState(int slotIndex) const {
    return (digout1Cond >> slotIndex) & 1;
  }

  /**
   * Gets the boolean state of an individual slot condition for digout2.
   * @param slotIndex slot index [0..15]
   * @return true if the slot condition is true
   */
  bool GetDigout2SlotState(int slotIndex) const {
    return (digout2Cond >> slotIndex) & 1;
  }

  /**
   * Decodes a DigoutSlotState from a packed digital output message payload.
   * @param msg message data (see protocol details)
   * @return decoded DigoutSlotState
   */
  static constexpr DigoutSlotState FromMsg(details::msg::DigitalOutput msg) {
    return DigoutSlotState {
      .digout1State = msg.digout1_state,
      .digout2State = msg.digout2_state,
      .digout1Sticky = msg.digout1_sticky,
      .digout2Sticky = msg.digout2_sticky,
      .digout1Cond = msg.digout1_cond,
      .digout2Cond = msg.digout2_cond
    };
  }
};

}  // namespace redux::sensors::canandcolor
