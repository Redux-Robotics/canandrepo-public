// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <stdint.h>
#include <units/time.h>
#include <array>
#include "MessageBus.h"
#include "CanandUtils.h"
namespace redux::canand {

/**
 * 
 * Class that represents a CAN message received from the Redux CanandEventLoop
 * 
 * This class is generally initialized by the event loop from a raw struct packet from the driver 
 * that gets read out and parsed. From there, it is then passed into CanandDevice::HandleMessage if it matches the address.
 * 
 * <p>
 * Of particular note are GetData() to get a handle to the packet's data and GetApiIndex() to see
 * what type of packet it is.
 * </p>
 * 
 * 
 */
class CanandMessage {
  public:

  /**
   * Construct a new CanandMessage from a bunch of parts -- not intended to be instantiated directly
   * @param busDescriptor bus descriptor value
   * @param id arbitration message id
   * @param timestamp message timestamp (milliseconds)
   * @param dataLen data size (0-64)
   * @param dataBuf pointer to data buffer
  */
  CanandMessage(uint16_t busDescriptor, uint32_t id, uint64_t timestamp, uint8_t dataLen, uint8_t* dataBuf) : \
    id{id}, timestamp{timestamp}, bus{busDescriptor} {
      dataSize = (dataLen > 64) ? 64 : dataLen;
      memcpy(data, dataBuf, 64);
  };
  virtual ~CanandMessage() = default;


  /**
   * Gets the full 29-bit CAN message id.
   * 
   * A summary of how the CAN message id works is described in {@link CanandAddress}.
   * @return The full 29-bit message id.
   */
  inline uint32_t GetId() { return id; }

  /**
   * Gets the 8-bit CAN API index. 
   * 
   * This is the value that generally describes what type of CAN message was sent.
   * @return the CAN API index.
   */
  inline uint8_t GetApiIndex() { return utils::getApiIndex(id); }

  /**
   * Gets the 6-bit CAN Device id.
   * 
   * This is the user-adjustible "CAN Id" of the associated CAN device in question.
   * @return the device id.
   */
  inline uint8_t GetDeviceId() { return utils::getDeviceId(id); }

  /**
   * Gets the 2-bit API page.
   * 
   * API page distinguishes between different API index banks.
   * @return the product id.
   */
  inline uint8_t GetApiPage() { return utils::getApiPage(id); }

  /**
   * Gets the 5-bit device type code
   * 
   * Product ID/ device type combinations will be unique to a Redux product.
   * @return the device type code.
   */
  inline uint8_t GetDeviceType() { return utils::getDeviceType(id); }

  /**
   * Gets the CAN message payload (up to 8 bytes).
   * 
   * The length of the array is determined by how many bytes were in the original CAN packet.
   * @return pointer to bytes that is 1-8 bytes long.
   */
  inline uint8_t* GetData() { return data; }

  /**
   * Gets the length of the CAN message's data in bytes.
   * @return length (1-8)
   */
  inline uint8_t GetLength() { return dataSize; }

  /**
   * Gets the CAN message timestamp, in seconds.
   * The time base is relative to the FPGA timestamp.
   * @return timestamp in seconds.
   */
  inline units::second_t GetTimestamp() { return units::microsecond_t{static_cast<double>(timestamp)}; }

  /**
   * Gets an object representing the CAN bus that received the message
   * @return bus object
   */
  inline MessageBus GetBus() { return bus; }

  private:
  uint32_t id;
  uint64_t timestamp;
  uint8_t dataSize;
  uint8_t data[64];
  MessageBus bus;
};
}