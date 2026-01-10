// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>
#include "DigoutPinConfig.h"

namespace redux::sensors::canandcolor {

/**
 * Internal data sources available to the device.
 *
 * These are primarily used for configuring digital output logic.
 */
enum class DataSource : uint8_t {
  kZero = 0,
  kProximity = 1,
  kRed = 2,
  kGreen = 3,
  kBlue = 4,
  kHue = 5,
  kSaturation = 6,
  kValue = 7,
};

/**
 * Digital output pin config that mirrors an internal data source.
 *
 * This config is used with CanandcolorSettings::SetDigoutPinConfig().
 */
class DataSourcePinConfig final : public DigoutPinConfig {
 public:
  /**
   * Constructs a DataSourcePinConfig.
   * @param dataSource data source to mirror
   */
  explicit DataSourcePinConfig(DataSource dataSource);

  uint64_t ToOutputSettingData() const override;
  bool Equals(const DigoutPinConfig& other) const override;

 private:
  DataSource dataSource_;
};

}  // namespace redux::sensors::canandcolor
