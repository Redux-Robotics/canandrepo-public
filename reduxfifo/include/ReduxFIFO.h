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

/**
 * ReduxFIFO.h: the entire driver API surface 
 * 
 * This gets exposed to the c/cpp api.
 * 
 * All functions in here should be thread safe. Should be.
 * 
 * In general, read and write buffer structures should ONLY have their memory be (de)allocated by the API surface.
 * This is to ensure that any memory management operations performed on the buffer will always be valid/not segfaults.
 * 
 * Redux Robotics is not responsible if your program segfaults from use with memory not allocated from ReduxFIFO.
 * 
 * ## Bus IDs
 * If the device ReduxFIFO is compiled for is the roboRIO, bus index 0 is *guarenteed* to be the Rio's CAN bus.
 * 
 * If the device ReduxFIFO is compiled for is SystemCore, bus indexes 0..4 are guarenteed to be SocketCAN buses 
 * `can_s0` through `can_s4`.
 * 
 */
#ifdef __cplusplus
extern "C" {
#endif

/**
 * Core message struct message.
 */
#ifdef _MSC_VER
#pragma pack(push, 4)
struct ReduxFIFO_Message
#else
struct __attribute__((packed, aligned(4))) ReduxFIFO_Message
#endif
{
    uint32_t message_id; // full 32-bit message id
    uint16_t bus_id; // index of the message bus the message is pulled from.
    uint8_t pad; // pad byte (reserved)
    uint8_t data_size; // length of the data (0-64)
    uint64_t timestamp; // 64-bit timestamp relative to the FPGA clock (microseconds)
    uint8_t data[64]; // CAN packet data
};
#ifdef _MSC_VER
#pragma pack(pop)
#endif

typedef uint64_t ReduxFIFO_Session;
typedef int32_t ReduxFIFO_Status;

/**
 * This represents a FIFO buffer, specifically the metadata half.
 *
 * This struct should ONLY be allocated from ReduxFIFO_AllocateReadBuffer 
 * or handed back from ReduxFIFO_ReadBarrier.
 */
#ifdef _MSC_VER
#pragma pack(push, 4)
struct ReduxFIFO_ReadBufferMeta
#else
struct __attribute__((packed, aligned(4))) ReduxFIFO_ReadBufferMeta
#endif
{
    /** 
     * [const] Session handle associated with the memory.
     * This is written by the session open and allocation functions.
     * You shouldn't need to touch it.
     */
    ReduxFIFO_Session session;
    /** 
     * [output] Where the status value from the last read barrier operation gets written to.
     * If a read barrier call fails on this buffer (e.g. having an invalid session ID in the session field),
     * the error code will be written to this field.
     */
    ReduxFIFO_Status status;
    /**
     * [output] The index of the next message slot to overwwrite.
     * If max_length == valid_length, then this is also where the oldest element is to be read from.
     * Else the oldest element lives at index 0.
     */
    uint32_t next_idx;
    /**
     * [output] The number of valid messages in the buffer.
     */
    uint32_t valid_length;
    /** 
     * [const] The maximum length of the messages buffer, in number of messages.
     */ 
    uint32_t max_length;
};
#ifdef _MSC_VER
#pragma pack(pop)
#endif

/**
 * Write buffer metadata.
 * This must ONLY be allocated by the associated functions.
 */
#ifdef _MSC_VER
#pragma pack(push, 4)
struct ReduxFIFO_WriteBufferMeta
#else
struct __attribute__((packed, aligned(4))) ReduxFIFO_WriteBufferMeta
#endif
{
    /**
     * [input] The bus ID to transmit on.
     * 
     * This can be set by the end-user.
     */
    uint32_t bus_id;

    /** 
     * [output] Where the status value from the last write barrier operation gets written to.
     * 
     * On a write barrier, each message in the buffer will get written out onto bus in order from 0..length.
     * If all messages get sent successfully, this will be set to REDUXFIFO_OK. 
     * If a message fails to send, the rest of the messages will not be sent, and this status will be updated with the relevant error code.
     */
    ReduxFIFO_Status status;

    /**
     * [output] This is where the number of messages that actually get written gets updated.
     */
    uint32_t messages_written;
    /**
     * [input] The number of valid messages in the buffer.
     * Write barriers use this to determine how many messages to write.
     */
    uint32_t length;
};
#ifdef _MSC_VER
#pragma pack(pop)
#endif


#ifdef _MSC_VER
#pragma pack(push, 4)
struct ReduxFIFO_SessionConfig
#else
struct __attribute__((packed, aligned(4))) ReduxFIFO_SessionConfig
#endif
{
    /** The filter ID to match with */
    uint32_t filter_id;
    /** The filter mask to AND incoming messages with */
    uint32_t filter_mask;

};
#ifdef _MSC_VER
#pragma pack(pop)
#endif


#define REDUXFIFO_OK                            0
#define REDUXFIFO_ERR_UNKNOWN                  -1
#define REDUXFIFO_ERR_NOT_INITIALIZED          -2
#define REDUXFIFO_ERR_NULL_POINTER_ARGUMENT    -3
#define REDUXFIFO_ERR_JAVA_INVALID_BYTEBUFFER  -4

#define REDUXFIFO_ERR_INVALID_BUS              -100
#define REDUXFIFO_ERR_BUS_ALREADY_OPENED       -101
#define REDUXFIFO_ERR_MAX_BUSES_OPENED         -102
#define REDUXFIFO_ERR_BUS_NOT_SUPPORTED        -103
#define REDUXFIFO_ERR_BUS_CLOSED               -104
#define REDUXFIFO_ERR_FAILED_TO_OPEN_BUS       -105
#define REDUXFIFO_ERR_BUS_READ_FAIL            -106
#define REDUXFIFO_ERR_BUS_WRITE_FAIL           -107
#define REDUXFIFO_ERR_BUS_BUFFER_FULL          -108

#define REDUXFIFO_ERR_INVALID_SESSION_ID         -200
#define REDUXFIFO_ERR_SESSION_ALREADY_OPENED     -201
#define REDUXFIFO_ERR_MAX_SESSIONS_OPENED        -202
#define REDUXFIFO_ERR_SESSION_CLSOED             -203
#define REDUXFIFO_ERR_MESSAGE_RECEIVE_TIMEOUT    -204

#define REDUXFIFO_ERR_HAL_CAN_OPEN_SESSION_FAIL  -301


/**
 * Returns a statically allocated null-terminated UTF-8 error message string.
 * Will always return a non-null pointer, even if that pointer is empty.
 * 
 * Just...please don't write to the pointers. Please.
 */
const char* ReduxFIFO_ErrorMessage(const ReduxFIFO_Status status);

/** 
 * Starts the ReduxFIFO driver. 
 * 
 * This is idempotent and will do nothing if called multiple times.
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_StartServer();

/** 
 * Shuts down the ReduxFIFO driver. 
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_StopServer();

/**
 * Returns the version code.
 * @return version code. This is encoded as (minor & 0xff) | (major << 8) | (year << 16)
 */
uint32_t ReduxFIFO_GetVersion();

/**
 * Opens a bus or returns a bus ID if a matching "bus address" already exists.
 *
 * bus address (e.g. "halcan" or "socketcan[.fd]:can0" or "gs_usb:16d0.1277/[serial numer]" or "slcan:/dev/ttyUSB0")
 * multiple bus addresses may be passed in with commas delimiting them
 *
 * other backends may be added depending on how we feel that day
 * 
 * @param[in] bus_address Bus address. Device buses will be opened with-best-faith effort 
 *                        and will attempt to transparently retain connection.
 *                        This MUST be valid UTF-8.
 * @param[out] bus_id Bus ID associated with the address.
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_OpenBus(const char* bus_address, uint16_t* bus_id);

/**
 * Close a bus.
 * 
 * This will also close all sessions associated with the bus.
 * 
 * @param[in] bus_id Bus id
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_CloseBus(const uint16_t bus_id);

/**
 * Opens a session.
 *
 * @param[in] bus_id ID of the bus used.
 * @param[in] msg_count the maximum number of messages to read
 * @param[in] session_config The session configuration struct.
 *
 * @return 0 on success, negative on error
 */
ReduxFIFO_Status ReduxFIFO_OpenSession(
    uint16_t bus_id,
    uint32_t msg_count,
    const struct ReduxFIFO_SessionConfig* session_config,
    ReduxFIFO_Session* session_id
);

/**
 * Closes a session handle. 
 * 
 * The currently in-flight read buffer held by the backend is deallocated.
 * 
 * @param[in] ses the session handle
 * @return 0 on success, negative on error
 */
ReduxFIFO_Status ReduxFIFO_CloseSession(ReduxFIFO_Session ses);

/**
 * Read buffer pointer struct.
 */
struct ReduxFIFO_ReadBuffer {
    struct ReduxFIFO_ReadBufferMeta* meta;
    struct ReduxFIFO_Message* data;
};

/** Write buffer pointer struct */
struct ReduxFIFO_WriteBuffer {
    struct ReduxFIFO_WriteBufferMeta* meta;
    struct ReduxFIFO_Message* data;
};

/**
 * Allocates a new read buffer.
 * 
 * # Buffers passed into ReduxFIFO_ReadBarrier MUST be allocated using this method!!!!
 * 
 * Failure to do so may cause segfaults!!!
 * 
 * @param[in] session session handle
 * @param[in] msg_count number of messages to make space for
 * @return a struct with the pointers for allocated data
 */
struct ReduxFIFO_ReadBuffer ReduxFIFO_AllocateReadBuffer(ReduxFIFO_Session session, const uint32_t msg_count);

/**
 * Frees the pointers referenced in the read buffer struct.
 */
void ReduxFIFO_FreeReadBuffer(struct ReduxFIFO_ReadBuffer buffer);

struct ReduxFIFO_WriteBuffer ReduxFIFO_AllocateWriteBuffer(uint16_t bus_id, uint32_t msg_count);
void ReduxFIFO_FreeWriteBuffer(struct ReduxFIFO_WriteBuffer buffer);

/**
 * Serves as a read barrier; this yields filled message buffers to the user program while 
 * accepting new buffers to write to until the next ReduxFIFO_ReadBarrier call.
 * 
 * For each ReduxFIFO_Buffer struct, the driver will swap the buffer currently associated for the session with 
 * the new passed in buffer and will start filling that memory with new messages. 
 * 
 * The previous buffer will have its `valid_length` and `next_idx` and `status` header fields updated with relevant data.
 * These can be used with the `buf.data` pointer to read out messages.
 * 
 * This message buffer swapping technique allows for double-buffering without excessive copy operations. 
 * To ensure memory safety, the user application should not touch a buffer currently in use by the driver; in Rust terms, the memory
 * should be "forgotten" until it is returned via the next read barrier call.
 * 
 * @param[in] bus_id The bus that all read buffers will attempt to get swapped with.
 * @param[in] buffers the array of buffers to swap in
 * @param[in] buffers_len the length of buffers
 * @return current time in microseconds
 */
ReduxFIFO_Status ReduxFIFO_ReadBarrier(
    uint16_t bus_id,
    struct ReduxFIFO_ReadBuffer buffers[],
    size_t buffers_len
);

/**
 * Multibus read buffers.
 * 
 * @param[in] buffers 2d array of read buffer entries. Each row is a single bus's read buffers.
 * @param[in] buffers_lengths 1d array of how long each row is.
 * @param[in] buffer_count the number of buffer rows
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_ReadBarrierMultiBus(
    struct ReduxFIFO_ReadBuffer* buffers[],
    size_t buffers_lengths[],
    size_t buffer_count
);

/**
 * Serves as a write barrier; this queues messages to be sent out and immidiately returns.
 * 
 * For each ReduxFIFO_Buffer struct, the driver will enqueue the messages from the buffer
 * in the struct and write back the `messages_written` and `status` fields.
 * 
 * The driver will not steal the buffer and it is still in user code's ownership on call end.
 * 
 * @param data Array of read barrier data to operate on.
 * @param[in] session_count Session count
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_WriteBarrier(
    struct ReduxFIFO_WriteBuffer* meta[],
    size_t session_count
);

/**
 * Writes a single message onto the bus ID specified by the message body.
 */
ReduxFIFO_Status ReduxFIFO_WriteSingle(ReduxFIFO_Message* msg);

/**
 * 
 * @param[in] session handle
 * @param[in] threshold message count threshold
 * @param[in] timeout_ms timeout in ms. 0 returns immediately.
 * @param[in] messages the number of messages when threshold reached. may be set to NULL. only valid if the return status is 0.
 * @return status
 */
ReduxFIFO_Status ReduxFIFO_WaitForThreshold(ReduxFIFO_Session session, uint32_t threshold, uint64_t timeout_ms, uint32_t* messages);

#ifdef __cplusplus
}  // extern "C"
#endif