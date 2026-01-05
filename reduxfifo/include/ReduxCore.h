// Copyright (c) 2022-2026 Bagholders of Redux Robotics
// 
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, version 3 of the License.
// 
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Lesser Public License for more
// details.
// 
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.


#pragma once
#include <stdint.h>
#include <stddef.h>
#include <ReduxFIFO.h>

/**
 * ReduxCore.h: the entire driver API surface (and the only header to be included with the driver)
 * 
 * This gets exposed to the c/cpp api.
*/
#ifdef __cplusplus
extern "C" {
#endif

/**
 * Returns the version number. This number is unique per version.
 * Minor version is bits 0-7
 * Major version is bits 8-15
 * Year is bits 16-30
 * 
 * @return version number integer
*/
int ReduxCore_GetVersion();

/**
 * Inits the Redux CANLink server that serves the frontend's websocket and provides messages to the vendordep.
 * This is generally called by the CanandEventLoop in either C++ or Java and doesn't need to be directly called.
 * This function is idempotent and will do nothing if called multiple times.
 * 
 * @return 0 on success, -1 on already started
*/
int ReduxCore_InitServer();

/**
 * Stops the Redux CANLink server. 
 * This is called by CanandEventLoop to stop CANLink.
 * 
 * @return 0 on success, -1 on already started
*/
int ReduxCore_StopServer();

/**
 * Sends a message to the bus with the specified handle ID. 
 * 
 * Bus IDs can begotten with the appropriate headers.
 * 
 * @param[in] busID bus id to send to
 * @param[in] messageID message ID to send
 * @param[in] data the data associated with the message
 * @param[out] dataSize the message data size (0-64)
 * @return 0 on success, negative on failure.
*/
int ReduxCore_EnqueueCANMessage(uint16_t busID, uint32_t messageID, const uint8_t* data, uint8_t dataSize);

/**
 * Sends multiple messages to the bus with the specified handle ID. 
 * 
 * @param[in] messages array of messages to send
 * @param[in] messageCount number of messages to queue
 * @param[out] messagesSent number of messages actually sent
 * @return 0 on success, negative on failure.
*/
int ReduxCore_BatchEnqueueCANMessages(struct ReduxFIFO_Message* messages, size_t messageCount, size_t* messagesSent);

/**
 * Sends multiple messages to the bus with the specified handle ID. 
 * 
 * @param[out] messages array of messages to receive into
 * @param[in] messageCount the maximum number of messages to receive
 * @param[out] messagesSent number of messages actually received
 * @return 0 on success, negative on failure.
*/
int ReduxCore_BatchWaitForCANMessages(struct ReduxFIFO_Message* messages, size_t messageCount, size_t* messagesRead);

/**
 * Blocks until a message has been received by CANLink server and writes the result to msgBuf.
 * 
 * @param[out] msgBuf message pointer to receive into
 * @return 0 on success, negative on failure. A value of -1 indicates the server has shut down.
*/
int ReduxCore_WaitForCANMessage(struct ReduxFIFO_Message* msgBuf);

/**
 * Allocates a buffer via the driver's memory allocator.
 */
struct ReduxFIFO_Message* ReduxCore_AllocateBuffer(size_t messageCount);

/**
 * Deallocates a buffer via the driver's memory allocator.
 * This can cause segfaults if not careful.
 */
struct ReduxFIFO_Message* ReduxCore_DeallocateBuffer(struct ReduxFIFO_Message* messages, size_t messageCount);

/**
 * Opens a bus for the ReduxCore adapter backend to read from by id.
 * The ID must be previously opened with ReduxFIFO_OpenBus().
 */
int ReduxCore_OpenBusById(uint16_t bus_id);

/**
 * Opens a bus for the ReduxCore adapter backend to read from by name.
 */
int ReduxCore_OpenBusByString(const char* bus_str);

/**
 * Close a bus from ReduxCore by id.
 * Note that this does not close the bus in the underlying system.
 */
int ReduxCore_CloseBus(uint16_t bus_id);

#ifdef __cplusplus
}  // extern "C"
#endif
