// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <units/temperature.h>
#include "redux/sensors/canandmag/CanandmagFaults.h"

namespace redux::sensors::canandmag {

/**
 * Struct class representing a Canandmag's status.
*/
struct CanandmagStatus {
  /**
   * Constructor for CanandmagStatus
   * @param activeFaultsRaw raw uint8_t field
   * @param stickyFaultsRaw raw uint8_t field
   * @param faultsValid whether the faults fields have valid adata
   * @param temp MCU temp
   * @param magnetInRange whether the encoder magnet is in range
  */
  constexpr CanandmagStatus(uint8_t activeFaultsRaw, uint8_t stickyFaultsRaw, bool faultsValid, units::celsius_t temp, bool magnetInRange): \
    activeFaults{activeFaultsRaw, faultsValid}, stickyFaults{stickyFaultsRaw, faultsValid}, temperature{temp}, magnetInRange{magnetInRange} {};
  public:
    /** Active faults. */
    CanandmagFaults activeFaults;
    /** Sticky faults. */
    CanandmagFaults stickyFaults;
    /** Encoder MCU temperature (celsius). */
    units::celsius_t temperature;
    /** Whether the magnet is in range. */
    bool magnetInRange;
};
}