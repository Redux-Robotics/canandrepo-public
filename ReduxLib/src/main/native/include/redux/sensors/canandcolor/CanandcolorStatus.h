// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <units/temperature.h>
#include "redux/sensors/canandcolor/CanandcolorFaults.h"

namespace redux::sensors::canandcolor {

/**
 * Status packet contents decoded from the Canandcolor.
 *
 * Contains active faults, sticky faults, and temperature.
 */
struct CanandcolorStatus {
  /**
   * @param activeFaultsRaw active faults bitfield
   * @param stickyFaultsRaw sticky faults bitfield
   * @param faultsValid true if value is valid
   * @param temp temperature (celsius)
   */
  constexpr CanandcolorStatus(
      uint8_t activeFaultsRaw,
      uint8_t stickyFaultsRaw,
      bool faultsValid,
      units::celsius_t temp
    ) :
      activeFaults{activeFaultsRaw, faultsValid},
      stickyFaults{stickyFaultsRaw, faultsValid},
      temperature{temp} {};

  /** Active faults (valid if faultsValid is true). */
  CanandcolorFaults activeFaults;
  /** Sticky faults (valid if faultsValid is true). */
  CanandcolorFaults stickyFaults;
  /** Temperature in degrees Celsius. */
  units::celsius_t temperature;
};

}  // namespace redux::sensors::canandcolor
