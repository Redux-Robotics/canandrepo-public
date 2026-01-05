// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include "CanandDevice.h"

namespace redux::canand {
    /** Adds a CanandDevice to the Canand packet event loop, allowing it to receive and process CAN packets.
     * 
     * This will start the Redux CANLink server if not started already.
     * @param device pointer to a CanandDevice to listen for packets. 
     */
    void AddCANListener(CanandDevice* device);

    /** Removes a CanandDevice to the Canand packet event loop. It is the responsibility of CanandDevice subclasses
     * to call this in a deconstructor.
     * @param device pointer to a CanandDevice to remove from the event loop.
     */
    void RemoveCANListener(CanandDevice* device);

    /**
     * Starts the Redux CANLink server if not started, otherwise does nothing. 
     * 
     * Generally does not need to be called manually if robot code instantiates a CanandDevice subclass as it will be started through AddCANListener.
    */
    void EnsureCANLinkServer();

    /**
     * Set whether to enable device presence warnings to the driver station globally (defaults to true).
     * 
     * @param enabled true to enable, false to suppress
     */
    void SetGlobalDevicePresenceWarnings(bool enabled);

    /**
     * Set whether to enable device presence warnings to the driver station for a single device (defaults to true).
     * 
     * @param device the CanandDevice to consider
     * @param enabled true to enable, false to suppress
     */
    void SetDevicePresenceWarnings(const CanandDevice& device, bool enabled);

    /**
     * Sets the device presence threshold of how many seconds must pass without a message for the
     * device checker to consider them disconnected from bus.
     * 
     * @param device the CanandDevice to act on
     * @param threshold the new threshold, in seconds. 
     */
    void SetDevicePresenceThreshold(const CanandDevice& device, units::second_t threshold);
}