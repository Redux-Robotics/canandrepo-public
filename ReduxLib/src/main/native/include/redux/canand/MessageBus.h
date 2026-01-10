// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <stdint.h>
#include <string>

namespace redux::canand {
/**
 * Class representing message buses that may exist on a robot.
 * 
*/
class MessageBus {
  public:

    /**
     * Constructs a new bus from a descriptor value.
     * 
     * 0 is the Rio's onboard bus, and 0 through 4 inclusive are can_s0 through can_s4 on SystemCore.
     * @param descriptorId the bus descriptor value.
    */
    inline MessageBus(uint16_t descriptorId) { fd = descriptorId; }
    virtual ~MessageBus() = default;

    /**
     * Returns the descriptor ID associated with the CAN bus object. 
     * Generally not needed to be used directly.
     * @return the descriptor ID.
     */
    inline uint16_t GetDescriptor() { return fd; }

    /**
     * Returns whether two MessageBus objects refer to the same bus.
     * @param other other MessageBus object to compare against
     * @return whether or not they refer to the same bus
     */
    inline bool Equals(MessageBus other) { return GetDescriptor() == other.GetDescriptor(); }

    /**
     * Constructs or fetches a bus by its bus string.
     * If the bus is not opened, it will attempt to be opened.
     * 
     * If the bus cannot be opened, an error will be thrown.
     * @param busString bus string, e.g. "halcan", "socketcan:can_s0", or "slcan:115200:/dev/ttyAMA0"
     * @return bus instance
     */
    static MessageBus ByBusString(std::string busString);
  private:
    uint16_t fd;
};
}