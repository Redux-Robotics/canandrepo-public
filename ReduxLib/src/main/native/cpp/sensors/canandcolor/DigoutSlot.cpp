// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/canandcolor/DigoutSlot.h"

namespace redux::sensors::canandcolor {

int32_t DigoutSlot::ComputeAdditiveImmediate(double value) {
  return static_cast<int32_t>(std::clamp(value, -1.0, 1.0) * 0x1fffff);
}

uint8_t DigoutSlot::ComputeMultiplicativeImmediate(double value) {
  return static_cast<uint8_t>(std::clamp(value, 0.0, 1.0) * 255.0);
}

int32_t DigoutSlot::ComputeTimingImmediate(double value) {
  double clamped = std::clamp(value, 0.0, 65535.0);
  return static_cast<int32_t>(clamped);
}

uint64_t DigoutSlot::ToSettingData() const {
  if (!enabled) {
    return 0;
  }

  uint64_t data = 0;

  data |= static_cast<uint64_t>(enabled ? 1 : 0);
  data |= static_cast<uint64_t>(static_cast<uint8_t>(nextSlotAction)) << 1;
  data |= static_cast<uint64_t>(invertValue ? 1 : 0) << 3;
  data |= static_cast<uint64_t>(opcode) << 4;
  data |= static_cast<uint64_t>(static_cast<uint32_t>(additiveImmediate & 0x1fffff)) << 11;
  data |= static_cast<uint64_t>(scalingImmediate) << 32;
  data |= static_cast<uint64_t>(static_cast<uint8_t>(lhsDataSource)) << 40;
  data |= static_cast<uint64_t>(static_cast<uint8_t>(rhsDataSource)) << 44;

  return data;
}

DigoutSlot DigoutSlot::FromSettingData(uint64_t data) {
  bool enabled = (data & 0x1) != 0;
  NextSlotAction nextSlotAction = static_cast<NextSlotAction>((data >> 1) & 0x3);
  bool invertValue = ((data >> 3) & 0x1) != 0;
  DigoutOperation opcode = static_cast<DigoutOperation>((data >> 4) & 0x7f);
  int32_t additiveImmediate = static_cast<int32_t>((data >> 11) & 0x1fffff);
  // additiveImmediate is a signed 21-bit two's-complement field; sign-extend it.
  if ((additiveImmediate & (1 << 20)) != 0) {
    additiveImmediate |= ~0x1fffff;
  }
  uint8_t scalingImmediate = static_cast<uint8_t>((data >> 32) & 0xff);
  DataSource lhsDataSource = static_cast<DataSource>((data >> 40) & 0xf);
  DataSource rhsDataSource = static_cast<DataSource>((data >> 44) & 0xf);

  return DigoutSlot(enabled, nextSlotAction, invertValue, opcode, additiveImmediate, scalingImmediate, lhsDataSource, rhsDataSource);
}

DigoutSlot DigoutSlot::Disabled() {
  return DigoutSlot(false, NextSlotAction::kTerminateChain, false, DigoutOperation::kEquals, 0, 0, DataSource::kZero, DataSource::kZero);
}

}  // namespace redux::sensors::canandcolor
