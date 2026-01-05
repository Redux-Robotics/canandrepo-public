// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <units/temperature.h>
#include "redux/sensors/canandgyro/CanandgyroFaults.h"

namespace redux::sensors::canandgyro {

/**
 * Struct class representing a Canandgyro's status.
*/
struct CanandgyroStatus {
  /**
   * Constructor for CanandgyroStatus
   * @param activeFaultsRaw raw uint8_t field
   * @param stickyFaultsRaw raw uint8_t field
   * @param faultsValid whether the faults fields have valid data
   * @param temp MCU temp
  */
  constexpr CanandgyroStatus(uint8_t activeFaultsRaw, uint8_t stickyFaultsRaw, bool faultsValid, units::celsius_t temp): \
    activeFaults{activeFaultsRaw, faultsValid}, stickyFaults{stickyFaultsRaw, faultsValid}, temperature{temp} {};
  public:
    /** Active faults. */
    CanandgyroFaults activeFaults;
    /** Sticky faults. */
    CanandgyroFaults stickyFaults;
    /** Device MCU temperature (celsius). */
    units::celsius_t temperature;
};
}