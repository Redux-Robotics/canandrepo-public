// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cinttypes>
#include <unordered_map>
#include <optional>
#include <units/time.h>
#include <units/angle.h>
#include "redux/canand/CanandSettings.h"
#include "CanandcolorDetails.h"
#include "ColorPeriod.h"
#include "ProximityPeriod.h"
#include "DigoutFrameTrigger.h"
#include "DigoutPinConfig.h"

namespace redux::sensors::canandcolor {

/**
 * Canandcolor configuration settings.
 *
 * Instances of this class act as a settings map: setters populate fields to be sent to the device, while getters return
 * `std::nullopt` if the setting value is not present in the current map.
 *
 * Example code:
 * ```cpp
 * Canandcolor color{0};
 *
 * CanandcolorSettings settings;
 * settings.SetColorFramePeriod(20_ms);
 * settings.SetProximityFramePeriod(20_ms);
 * settings.SetLampLEDBrightness(0.5);
 * settings.SetDigoutPinConfig(0, ActiveHighDigoutPinConfig{});
 * settings.SetDigoutFrameTrigger(0, DigoutFrameTrigger::kRisingAndFalling);
 *
 * color.SetSettings(settings, 20_ms);
 * ```
 */
class CanandcolorSettings : public redux::canand::CanandSettings {
 public:
  CanandcolorSettings() = default;
  const std::vector<uint8_t>& SettingAddresses() const override;

  /**
   * Sets the status frame period.
   * @param period frame period; clamped by the device
   * @return reference to this object
   */
  CanandcolorSettings& SetStatusFramePeriod(units::second_t period);

  /**
   * Sets the proximity frame period.
   * @param period frame period; clamped by the device
   * @return reference to this object
   */
  CanandcolorSettings& SetProximityFramePeriod(units::second_t period);

  /**
   * Sets the color frame period.
   * @param period frame period; clamped by the device
   * @return reference to this object
   */
  CanandcolorSettings& SetColorFramePeriod(units::second_t period);

  /**
   * Sets the digital output frame period.
   * @param period frame period; clamped by the device
   * @return reference to this object
   */
  CanandcolorSettings& SetDigoutFramePeriod(units::second_t period);

  /**
   * Gets the configured status frame period, if present.
   * @return optional frame period
   */
  std::optional<units::second_t> GetStatusFramePeriod();

  /**
   * Gets the configured proximity frame period, if present.
   * @return optional frame period
   */
  std::optional<units::second_t> GetProximityFramePeriod();

  /**
   * Gets the configured color frame period, if present.
   * @return optional frame period
   */
  std::optional<units::second_t> GetColorFramePeriod();

  /**
   * Gets the configured digital output frame period, if present.
   * @return optional frame period
   */
  std::optional<units::second_t> GetDigoutFramePeriod();

  /**
   * Sets the lamp LED brightness.
   * @param brightness desired brightness, clamped to [0..1]
   * @return reference to this object
   */
  CanandcolorSettings& SetLampLEDBrightness(double brightness);

  /**
   * Sets the color integration period.
   * @param period integration period enum
   * @return reference to this object
   */
  CanandcolorSettings& SetColorIntegrationPeriod(ColorPeriod period);

  /**
   * Sets the proximity integration period.
   * @param period integration period enum
   * @return reference to this object
   */
  CanandcolorSettings& SetProximityIntegrationPeriod(ProximityPeriod period);

  /**
   * Gets the lamp LED brightness, if present.
   * @return optional brightness [0..1]
   */
  std::optional<double> GetLampLEDBrightness();

  /**
   * Gets the configured color integration period, if present.
   * @return optional integration period enum
   */
  std::optional<ColorPeriod> GetColorIntegrationPeriod();

  /**
   * Gets the configured proximity integration period, if present.
   * @return optional integration period enum
   */
  std::optional<ProximityPeriod> GetProximityIntegrationPeriod();

  /**
   * Enables aligning proximity frames to the integration period.
   * @param align true to align frames
   * @return reference to this object
   */
  CanandcolorSettings& SetAlignProximityFramesToIntegrationPeriod(bool align);

  /**
   * Enables aligning color frames to the integration period.
   * @param align true to align frames
   * @return reference to this object
   */
  CanandcolorSettings& SetAlignColorFramesToIntegrationPeriod(bool align);

  /**
   * Gets whether proximity frames are aligned to the integration period, if present.
   * @return optional flag
   */
  std::optional<bool> GetAlignProximityFramesToIntegrationPeriod();

  /**
   * Gets whether color frames are aligned to the integration period, if present.
   * @return optional flag
   */
  std::optional<bool> GetAlignColorFramesToIntegrationPeriod();

  /**
   * Sets the digital output pin behavior for one output.
   *
   * Typical usage is one of:
   * - DisabledDigoutPinConfig
   * - ActiveHighDigoutPinConfig
   * - ActiveLowDigoutPinConfig
   * - DataSourcePinConfig (mirror an internal data source)
   *
   * @param digout output index (0 for digout1, 1 for digout2)
   * @param config pin configuration
   * @return reference to this object
   */
  CanandcolorSettings& SetDigoutPinConfig(uint8_t digout, const DigoutPinConfig& config);

  /**
   * Sets whether a digital output frame is emitted on changes to a given output.
   *
   * @param digout output index (0 for digout1, 1 for digout2)
   * @param trigger trigger behavior
   * @return reference to this object
   */
  CanandcolorSettings& SetDigoutFrameTrigger(uint8_t digout, DigoutFrameTrigger trigger);

  /**
   * Gets the raw pin configuration value for one output, if present.
   * @param digout output index (0 for digout1, 1 for digout2)
   * @return optional raw value
   */
  std::optional<uint64_t> GetDigoutPinConfig(uint8_t digout);

  /**
   * Gets the digital output frame trigger for one output, if present.
   * @param digout output index (0 for digout1, 1 for digout2)
   * @return optional trigger
   */
  std::optional<DigoutFrameTrigger> GetDigoutFrameTrigger(uint8_t digout);
};

}  // namespace redux::sensors::canandcolor
