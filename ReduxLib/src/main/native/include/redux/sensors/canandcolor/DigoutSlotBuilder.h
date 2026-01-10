// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>
#include "DigoutSlot.h"
#include "units/time.h"

namespace redux::sensors::canandcolor {

/**
 * Helper for constructing DigoutSlot instances.
 *
 * Typical usage:
 * ```cpp
 * using namespace redux::sensors::canandcolor;
 * DigoutSlot slot = DigoutSlotBuilder()
 *   .GreaterThan(DataSource::kProximity, DataSource::kZero)
 *   .Add(-0.5)
 *   .BuildTerminateChain();
 * ```
 */
class DigoutSlotBuilder {
 public:
  DigoutSlotBuilder();

  /**
   * Builds a slot with `lhs == rhs`.
   * @param lhs left hand data source
   * @param rhs right hand data source
   * @return builder handle
   */
  DigoutSlotBuilder& Equals(DataSource lhs, DataSource rhs);
  /** 
   * Builds a slot with `lhs < rhs`. 
   * @param lhs left hand data source
   * @param rhs right hand data source
   * @return builder handle
   */
  DigoutSlotBuilder& LessThan(DataSource lhs, DataSource rhs);
  /** 
   * Builds a slot with `lhs > rhs`. 
   * @param lhs left hand data source
   * @param rhs right hand data source
   * @return builder handle
   */
  DigoutSlotBuilder& GreaterThan(DataSource lhs, DataSource rhs);
  /**
   * Builds a slot with `lhs <= rhs`. 
   * @param lhs left hand data source
   * @param rhs right hand data source
   * @return builder handle
   */
  DigoutSlotBuilder& LessThanOrEquals(DataSource lhs, DataSource rhs);
  /**
   * Builds a slot with `lhs >= rhs`. 
   * @param lhs left hand data source
   * @param rhs right hand data source
   * @return builder handle
   */
  DigoutSlotBuilder& GreaterThanOrEquals(DataSource lhs, DataSource rhs);

  /**
   * Copies one data source directly to another (device-defined behavior).
   * @param lhs left-hand side data source
   * @param rhs right-hand side data source
   * @return reference to this builder
   */
  DigoutSlotBuilder& DirectSourceToSource(DataSource lhs, DataSource rhs);

  /**
   * Sets this slot to be true if the previous slot has been true for at least the given duration.
   * @param durationMs duration in milliseconds
   * @return reference to this builder
   */
  DigoutSlotBuilder& PrevSlotTrue(units::millisecond_t durationMs);

  /**
   * Sets this slot to be true if the previous chain clause has been true for at least the given duration.
   * @param durationMs duration in milliseconds
   * @return reference to this builder
   */
  DigoutSlotBuilder& PrevChainTrueFor(units::millisecond_t durationMs);

  /**
   * Sets this slot to be true for at least the given duration (device-defined behavior).
   * @param durationMs duration in milliseconds
   * @return reference to this builder
   */
  DigoutSlotBuilder& TrueFor(units::millisecond_t durationMs);

  /**
   * Applies a scaling factor (device-defined) to this slot.
   * @param factor scaling factor
   * @return reference to this builder
   */
  DigoutSlotBuilder& Scale(double factor);

  /**
   * Applies an additive offset (device-defined) to this slot.
   * @param offset offset in normalized units
   * @return reference to this builder
   */
  DigoutSlotBuilder& Add(double offset);

  /**
   * Builds a DigoutSlot with the specified next-slot action.
   * @param nextAction how to combine with the next slot
   * @return constructed DigoutSlot
   */
  DigoutSlot Build(NextSlotAction nextAction);

  /**
   * Builds a DigoutSlot that terminates the chain.
   * @return constructed DigoutSlot
   */
  DigoutSlot BuildTerminateChain();

 private:
  DataSource lhs = DataSource::kZero;
  DataSource rhs = DataSource::kZero;
  DigoutOperation opcode = DigoutOperation::kEquals;
  int32_t additiveImmediate = 0;
  uint8_t scalingImmediate = 0;
  bool invertValue = false;
};

}  // namespace redux::sensors::canandcolor
