// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>
#include <algorithm>
#include <cmath>
#include "DigoutOperation.h"
#include "DataSource.h"
#include "NextSlotAction.h"

namespace redux::sensors::canandcolor {

/**
 * One slot in a digital output logic chain.
 *
 * Slots are serialized to/from raw setting data via ToSettingData()/FromSettingData().
 */
class DigoutSlot {
 public:
  /**
   * Constructs a DigoutSlot.
   * @param enabled slot enable
   * @param nextSlotAction next slot action
   * @param invertValue invert comparison result of slot
   * @param opcode opcode
   * @param additiveImmediate add value
   * @param scalingImmediate scaling value
   * @param lhsDataSource left-hand comparison datasource
   * @param rhsDataSource right-hand comparison datasource
   */
  constexpr DigoutSlot(
        const bool enabled = false,
        const NextSlotAction nextSlotAction = NextSlotAction::kTerminateChain,
        const bool invertValue = false,
        const DigoutOperation opcode = DigoutOperation::kEquals,
        const int32_t additiveImmediate = 0,
        const uint8_t scalingImmediate = 0,
        const DataSource lhsDataSource = DataSource::kZero,
        const DataSource rhsDataSource = DataSource::kZero
    ) :
      enabled{enabled}, 
      nextSlotAction{nextSlotAction},
      invertValue{invertValue},
      opcode{opcode},
      additiveImmediate{additiveImmediate},
      scalingImmediate{scalingImmediate},
      lhsDataSource{lhsDataSource},
      rhsDataSource{rhsDataSource} {};

  /** Whether this slot is enabled. */
  bool enabled{false};
  /** How to combine this slot with the next slot. */
  NextSlotAction nextSlotAction{NextSlotAction::kTerminateChain};
  /** Whether to invert the computed slot boolean. */
  bool invertValue{false};
  /** Operation to perform for this slot. */
  DigoutOperation opcode{DigoutOperation::kEquals};
  /** Additive immediate value used by certain operations. */
  int32_t additiveImmediate{0};
  /** Scaling immediate value used by certain operations. */
  uint8_t scalingImmediate{0};
  /** Left-hand-side data source. */
  DataSource lhsDataSource{DataSource::kZero};
  /** Right-hand-side data source. */
  DataSource rhsDataSource{DataSource::kZero};

  /**
   * Converts a normalized floating-point value to the slot additive immediate encoding.
   * @param value value clamped to [-1..1]
   * @return encoded immediate
   */
  static int32_t ComputeAdditiveImmediate(double value);

  /**
   * Converts a normalized floating-point scaling factor to the slot multiplicative immediate encoding.
   * @param value scaling factor clamped to [0..1]
   * @return encoded scaling immediate
   */
  static uint8_t ComputeMultiplicativeImmediate(double value);

  /**
   * Converts a millisecond duration to the slot timing immediate encoding.
   * @param value duration in milliseconds
   * @return encoded timing immediate
   */
  static int32_t ComputeTimingImmediate(double value);

  /**
   * Serializes this slot into raw setting data expected by the device.
   * @return raw setting data
   */
  uint64_t ToSettingData() const;

  /**
   * Deserializes a slot from raw setting data.
   * @param data raw setting data
   * @return DigoutSlot decoded
   */
  static DigoutSlot FromSettingData(uint64_t data);

  /**
   * Returns a disabled slot.
   * @return disabled slot
   */
  static DigoutSlot Disabled();
};

}  // namespace redux::sensors::canandcolor
