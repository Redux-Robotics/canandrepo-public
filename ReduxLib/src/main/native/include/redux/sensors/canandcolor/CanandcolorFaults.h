// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>

namespace redux::sensors::canandcolor {

/**
 * Canandcolor fault flags (active or sticky).
 *
 * Use Canandcolor::GetActiveFaults() and Canandcolor::GetStickyFaults() to retrieve these.
 */
class CanandcolorFaults {
 public:
  /**
   * @param field fault bitfield
   * @param valid true if valid
   */
  constexpr CanandcolorFaults(uint8_t field, bool valid) :
      powerCycle(field & 0b1),
      canIDConflict(field & 0b10),
      canGeneralError(field & 0b100),
      outOfTemperatureRange(field & 0b1000),
      hardwareFaultProximity(field & 0b10000),
      hardwareFaultColor(field & 0b100000),
      i2cBusRecovery(field & 0b1000000),
      faultsValid(valid) {};

  /** The power cycle flag is set on device boot until sticky faults are cleared. */
  bool powerCycle;
  /** CAN ID conflict detected. */
  bool canIDConflict;
  /** CAN general error detected (often wiring/intermittent bus issues). */
  bool canGeneralError;
  /** Temperature out of expected range. */
  bool outOfTemperatureRange;
  /** Proximity sensor hardware fault. */
  bool hardwareFaultProximity;
  /** Color sensor hardware fault. */
  bool hardwareFaultColor;
  /** I2C bus recovery occurred. */
  bool i2cBusRecovery;
  /** Whether this fault set is valid (i.e. a status frame has been received). */
  bool faultsValid;
};

}  // namespace redux::sensors::canandcolor
