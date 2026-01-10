// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/sensors/canandcolor/DigoutSlotBuilder.h"

namespace redux::sensors::canandcolor {

DigoutSlotBuilder::DigoutSlotBuilder() = default;

DigoutSlotBuilder& DigoutSlotBuilder::Equals(DataSource lhs, DataSource rhs) {
  opcode = DigoutOperation::kEquals;
  this->lhs = lhs;
  this->rhs = rhs;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::LessThan(DataSource lhs, DataSource rhs) {
  opcode = DigoutOperation::kLessThan;
  this->lhs = lhs;
  this->rhs = rhs;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::GreaterThan(DataSource lhs, DataSource rhs) {
  opcode = DigoutOperation::kGreaterThan;
  this->lhs = lhs;
  this->rhs = rhs;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::LessThanOrEquals(DataSource lhs, DataSource rhs) {
  opcode = DigoutOperation::kLessThanOrEquals;
  this->lhs = lhs;
  this->rhs = rhs;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::GreaterThanOrEquals(DataSource lhs, DataSource rhs) {
  opcode = DigoutOperation::kGreaterThanOrEquals;
  this->lhs = lhs;
  this->rhs = rhs;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::DirectSourceToSource(DataSource lhs, DataSource rhs) {
  opcode = DigoutOperation::kEquals;
  additiveImmediate = 0;
  this->lhs = lhs;
  this->rhs = rhs;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::PrevSlotTrue(units::millisecond_t durationMs) {
  additiveImmediate = DigoutSlot::ComputeTimingImmediate(durationMs.value());
  opcode = DigoutOperation::kPrevSlotTrue;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::PrevChainTrueFor(units::millisecond_t durationMs) {
  additiveImmediate = DigoutSlot::ComputeTimingImmediate(durationMs.value());
  opcode = DigoutOperation::kPrevClauseTrue;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::TrueFor(units::millisecond_t durationMs) {
  additiveImmediate = DigoutSlot::ComputeTimingImmediate(durationMs.value());
  opcode = DigoutOperation::kEquals;
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::Scale(double factor) {
  scalingImmediate = DigoutSlot::ComputeMultiplicativeImmediate(factor);
  return *this;
}

DigoutSlotBuilder& DigoutSlotBuilder::Add(double offset) {
  additiveImmediate = DigoutSlot::ComputeAdditiveImmediate(offset);
  return *this;
}

DigoutSlot DigoutSlotBuilder::Build(NextSlotAction nextAction) {
  return DigoutSlot(true, nextAction, invertValue, opcode, additiveImmediate, scalingImmediate, lhs, rhs);
}

DigoutSlot DigoutSlotBuilder::BuildTerminateChain() {
  return DigoutSlot(true, NextSlotAction::kTerminateChain, invertValue, opcode, additiveImmediate, scalingImmediate, lhs, rhs);
}

}  // namespace redux::sensors::canandcolor
