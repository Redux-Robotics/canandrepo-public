// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/canandcolor/DataSource.h"

namespace redux::sensors::canandcolor {

DataSourcePinConfig::DataSourcePinConfig(DataSource dataSource)
    : dataSource_{dataSource} {}

uint64_t DataSourcePinConfig::ToOutputSettingData() const {
  return static_cast<uint64_t>(dataSource_);
}

bool DataSourcePinConfig::Equals(const DigoutPinConfig& other) const {
  const auto* otherConfig = dynamic_cast<const DataSourcePinConfig*>(&other);
  return otherConfig != nullptr && dataSource_ == otherConfig->dataSource_;
}

}  // namespace redux::sensors::canandcolor
