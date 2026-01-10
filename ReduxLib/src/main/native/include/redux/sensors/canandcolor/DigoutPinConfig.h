// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once

#include <cstdint>

namespace redux::sensors::canandcolor {

/**
 * Base class for digital output pin behavior configuration.
 *
 * Use CanandcolorSettings::SetDigoutPinConfig() to apply these to the device.
 */
class DigoutPinConfig {
 public:
  virtual ~DigoutPinConfig() = default;

  /**
   * Converts this config into the raw setting payload expected by the device.
   * @return raw setting payload
   */
  virtual uint64_t ToOutputSettingData() const = 0;

  /**
   * Compares this config with another config for logical equality.
   * @param other other config
   * @return true if logically equal
   */
  virtual bool Equals(const DigoutPinConfig& other) const = 0;
};

/**
 * Disables a digital output pin.
 */
class DisabledDigoutPinConfig final : public DigoutPinConfig {
 public:
  uint64_t ToOutputSettingData() const override {
    return 0;
  }
  bool Equals(const DigoutPinConfig& other) const override {
    return dynamic_cast<const DisabledDigoutPinConfig*>(&other) != nullptr;
  }
};

/**
 * Configures a digital output pin as active-high.
 */
class ActiveHighDigoutPinConfig final : public DigoutPinConfig {
 public:
  uint64_t ToOutputSettingData() const override {
    return 1;
  }
  bool Equals(const DigoutPinConfig& other) const override {
    return dynamic_cast<const ActiveHighDigoutPinConfig*>(&other) != nullptr;
  }
};

/**
 * Configures a digital output pin as active-low.
 */
class ActiveLowDigoutPinConfig final : public DigoutPinConfig {
 public:
  uint64_t ToOutputSettingData() const override {
    return 2;
  }
  bool Equals(const DigoutPinConfig& other) const override {
    return dynamic_cast<const ActiveLowDigoutPinConfig*>(&other) != nullptr;
  }
};

}  // namespace redux::sensors::canandcolor
