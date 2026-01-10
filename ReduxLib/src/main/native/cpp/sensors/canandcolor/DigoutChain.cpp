// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/canandcolor/DigoutChain.h"

namespace redux::sensors::canandcolor {

DigoutChain& DigoutChain::Add(const DigoutSlot& slot) {
  if (length >= 16) {
    return *this;
  }
  slots[length++] = slot;
  return *this;
}

DigoutSlot DigoutChain::GetSlot(size_t index) {
  if (index >= length) {
    return DigoutSlot::Disabled();
  }
  return slots[index];
}

size_t DigoutChain::GetLength() {
  return length;
}

}  // namespace redux::sensors::canandcolor
