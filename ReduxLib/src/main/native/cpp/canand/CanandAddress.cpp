// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#include "redux/canand/CanandAddress.h"
#include "ReduxCore.h"

namespace redux::canand {
    bool CanandAddress::SendCANMessage(uint16_t apiIndex, uint8_t* data, uint8_t length) {
        return ReduxCore_EnqueueCANMessage(bus.GetDescriptor(), utils::constructMessageId(devType, devId, apiIndex), data, length) == 0;
    }
}