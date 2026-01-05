// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include "MessageBus.h"
#include "CanandUtils.h"
#include "CanandMessage.h"
namespace redux::canand {
/**
 * Class representing the exact combination of CAN bus, product IDs, and device IDs that uniquely correspond to a Redux CAN device on a robot.
 * 
 * <p>
 * The full 29-bit CAN ID that actually gets put on the bus can be broken down into four components:
 * </p>
 * <ol>
 * <li> A 5 bit device type (devType) that is product specific (the Canandmag is 7 for "gear tooth sensor")</li>
 * <li> an 8-bit manufacturer code unique to each vendor (the Redux code is 14) </li>
 * <li> 10 bits of API identifier</li>
 * <li> a 6-bit device number (devId) that is user-configurable so you can have multiple of a device on a bus
 * (this is what the "CAN Id" in robot code and vendor tuners usually refer to)</li>
 * </ol>
 * 
 * <a href="https://docs.wpilib.org/en/stable/docs/software/can-devices/can-addressing.html"> The WPILib docs</a> elaborate on this in a bit more detail. 
 * 
 * Of note is that it breaks down the 10-bit API identifier into a 6-bit API class and 4-bit API 
 * index. Redux products, however, break it down into a 2 bit API page (page) and an 8 bit API index
 * (apiIndex), which actually gets used for a device's message API. 
 * 
 * <p>
 * The breakdown can be seen in the diagram provided below:
 * </p>
 * <pre class="not-code">
 * +-------------------+-------------------------------+-------+-------------------------------+-----------------------+
 * |    Device Type    | Manufacturer ID   (redux=0xE) | Page  |           API Index           |   Device ID (devID)   |
 * +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
 * | 28| 27| 26| 25| 24| 23| 22| 21| 20| 19| 18| 17| 16| 15| 14| 13| 12| 11| 10|  9|  8|  7|  6|  5|  4|  3|  2|  1|  0|
 * +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
 * </pre>
 * Documentation on what each apiIndex value does as well as DBCs for Redux devices is available at 
 * <a href="https://docs.reduxrobotics.com/">https://docs.reduxrobotics.com/</a>
 * 
 * <p>
 * In summary, to check if an incoming CAN message is from the device the CanandAddress represents, it needs to check:
 * </p>
 * <ul>
 * <li> the devType </li>
 * <li> the devId </li>
 * <li> and the CAN bus the message was sent from, as different devices on different buses can share identical IDs</li>
 * </ul>
 * to ensure they all match, leaving the apiIndex to be parsed by device class code. (The manufacturer ID is filtered for in the native driver portion of ReduxLib.)
 * 
 * <p>
 * Subclasses of CanandDevice will use this class for CAN i/o to a device, as this class provides a method to send CAN packets to the device it describes, and 
 * the internal event loop uses it to determine which messages to give to CanandDevice::HandleMessage by checking them against MsgMatches, yielding
 * an asynchronous method of receiving packets.
 * </p>
 * 
 * 
 * 
 * 
 */
class CanandAddress {
  public:
    /**
     * Constructor with explicit CAN bus.
     * @param bus the bus the address is associated with.
     * @param devType the device type
     * @param devId the device CAN id
     */
    CanandAddress(MessageBus& bus, uint8_t devType, uint8_t devId) : bus{bus}, devType{devType}, devId{devId} {};

    /**
     * Constructor with implicit Rio CAN bus.
     * @param devType the device type
     * @param devId the device CAN id
     */
    CanandAddress(uint8_t devType, uint8_t devId) : bus{0}, devType{devType}, devId{devId} {};

    /**
     * Checks if a CAN message matches against the device type, product id, and device can id
     * 
     * @param msg a CanandMessage matching
     * @return if there is a match
     */
    inline bool MsgMatches(CanandMessage& msg) { return utils::idMatches(msg.GetId(), devType, devId) && bus.Equals(msg.GetBus()); }

    /**
     * Sends a CAN message to the CanandAddress.
     * 
     * @param apiIndex the API index the message should have (between 0-1023 inclusive, optionally ORed with 512 or 256)
     * @param data 1-8 bytes of payload, as an array or pointer
     * @param length the length of the the payload buffer
     * @return if the operation was successful
     */
    bool SendCANMessage(uint16_t apiIndex, uint8_t* data, uint8_t length);
    virtual ~CanandAddress() = default;


    /**
     * Returns the 5-bit device type.
     * 
     * @return the device type
     */
    inline uint8_t GetDeviceType() { return devType; }

    /**
     * Returns the user-settable device ID.
     * 
     * @return the device ID
     */
    inline uint8_t GetDeviceId() { return devId; }

  private:
    MessageBus bus;
    uint8_t devType;
    uint8_t devId;
    uint8_t baseMsgId;
};
}