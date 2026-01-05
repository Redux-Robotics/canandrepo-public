# ReduxFIFO Usage Guide

## Overview

ReduxFIFO is a message interposer layer that maintains connections to CAN message buses and serves packets to client applications. It supports multiple backend types including WebSocket, USB, SocketCAN, and HAL CAN.

## Core Concepts

### FIFOCore
The main entry point for ReduxFIFO operations. Manages buses, sessions, and message routing.

```rust
use reduxfifo::{fifocore::FIFOCore, ReduxFIFOSessionConfig};

let fifocore = FIFOCore::new(None);
```

### Bus Types
ReduxFIFO supports multiple bus backends:

- **WebSocket**: `websocket:ws://host:port/path` or `websocket:wss://host:port/path`
- **USB**: `rdxusb:channel.vid.pid.serial`
- **SocketCAN**: `socketcan:bus_name` (Linux only)
- **HAL CAN**: `halcan` (roboRIO only)

### Sessions
Sessions represent message filters and buffers for a specific bus. Each session has:
- Filter ID and mask for message filtering
- Read buffer for incoming messages
- Write buffer for outgoing messages

## WebSocket Backend Usage

### Opening a WebSocket Bus

```rust
// Open a WebSocket bus
let bus_id = fifocore.open_or_get_bus("websocket:ws://localhost:7244/ws")?;

// For secure WebSocket connections
let bus_id = fifocore.open_or_get_bus("websocket:wss://example.com:7244/ws")?;
```

### Creating a Session

```rust
// Create session configuration
let config = ReduxFIFOSessionConfig {
    filter_id: 0x0,      // Accept all messages (0x0 = no filter)
    filter_mask: 0x0,    // No filtering
};

// For specific message filtering
let config = ReduxFIFOSessionConfig {
    filter_id: 0x123,    // Only accept messages with ID 0x123
    filter_mask: 0x7FF,  // 11-bit mask for standard CAN IDs
};

// Open session with 100 message buffer
let session = fifocore.open_session(bus_id, 100, config)?;
```

### Reading Messages

```rust
use reduxfifo::{ReadBuffer, ReduxFIFOMessage};

// Create read buffer
let mut read_buf = ReadBuffer::new(session, 100);

// Perform read barrier to get messages
fifocore.read_barrier(bus_id, &mut [&mut read_buf])?;

// Iterate through received messages
for msg in read_buf.messages() {
    println!("Message ID: 0x{:X}, Data: {:?}", msg.message_id, msg.data_slice());
}
```

### Writing Messages

```rust
use reduxfifo::{WriteBuffer, ReduxFIFOMessage};

// Create write buffer
let mut write_buf = WriteBuffer::new(session, 10);

// Add messages to write buffer
let msg = ReduxFIFOMessage {
    message_id: 0x123,
    bus_id: bus_id,
    flags: 0,
    data_size: 8,
    timestamp: 0, // Will be set automatically
    data: [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};
write_buf.add_message(msg);

// Perform write barrier to send messages
fifocore.write_barrier(&mut [&mut write_buf]);
```

### Single Message Writing

```rust
// Write a single message directly
let msg = ReduxFIFOMessage {
    message_id: 0x123,
    bus_id: bus_id,
    flags: 0,
    data_size: 8,
    timestamp: 0,
    data: [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

fifocore.write_single(&msg)?;
```

## CANLink WebSocket API

ReduxFIFO provides a web server interface for remote access via WebSocket.

### Starting the Server

```rust
use reduxfifo::canlink;

// Start the CANLink web server
let (shutdown_send, shutdown_recv) = tokio::sync::watch::channel(false);
let web_task = fifocore.runtime().spawn(canlink::run_web_server(shutdown_recv, fifocore));

// Server runs on port 7244 by default
```

### API Endpoints

- **WebSocket Connection**: `ws://localhost:7244/ws/{bus_id}/{filter_id}/{filter_mask}`
- **List Buses**: `GET http://localhost:7244/buses`
- **Open Bus**: `POST http://localhost:7244/buses/open/{params}`
- **Version**: `GET http://localhost:7244/version`

### Opening WebSocket Bus via API

```bash
# Open a WebSocket bus
curl -X POST "http://localhost:7244/buses/open/websocket:ws://remote-server:7244/ws"

# Response: {"id": {"Ok": 1}, "params": "websocket:ws://remote-server:7244/ws"}
```

## Error Handling

ReduxFIFO uses a custom `Error` enum for error handling:

```rust
use reduxfifo::error::Error;

match fifocore.open_or_get_bus("websocket:ws://localhost:7244/ws") {
    Ok(bus_id) => println!("Bus opened: {}", bus_id),
    Err(Error::InvalidBus) => println!("Invalid bus parameters"),
    Err(Error::BusNotSupported) => println!("Bus type not supported"),
    Err(Error::BusBufferFull) => println!("Bus buffer is full"),
    Err(e) => println!("Other error: {:?}", e),
}
```

## Common Patterns

### Complete WebSocket Example

```rust
use reduxfifo::{fifocore::FIFOCore, ReduxFIFOSessionConfig, ReadBuffer, WriteBuffer, ReduxFIFOMessage};

fn main() -> anyhow::Result<()> {
    // Initialize FIFOCore
    let fifocore = FIFOCore::new(None);
    
    // Open WebSocket bus
    let bus_id = fifocore.open_or_get_bus("websocket:ws://localhost:7244/ws")?;
    
    // Create session
    let config = ReduxFIFOSessionConfig {
        filter_id: 0x0,
        filter_mask: 0x0,
    };
    let session = fifocore.open_session(bus_id, 100, config)?;
    
    // Create buffers
    let mut read_buf = ReadBuffer::new(session, 100);
    let mut write_buf = WriteBuffer::new(session, 10);
    
    // Send a message
    let msg = ReduxFIFOMessage {
        message_id: 0x123,
        bus_id: bus_id,
        flags: 0,
        data_size: 8,
        timestamp: 0,
        data: [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };
    write_buf.add_message(msg);
    fifocore.write_barrier(&mut [&mut write_buf]);
    
    // Read messages
    fifocore.read_barrier(bus_id, &mut [&mut read_buf])?;
    for msg in read_buf.messages() {
        println!("Received: ID=0x{:X}, Data={:?}", msg.message_id, msg.data_slice());
    }
    
    Ok(())
}
```

### Async Usage with Tokio

```rust
use reduxfifo::{fifocore::FIFOCore, ReduxFIFOSessionConfig};
use tokio::time::{sleep, Duration};

async fn async_example() -> anyhow::Result<()> {
    let fifocore = FIFOCore::new(None);
    let bus_id = fifocore.open_or_get_bus("websocket:ws://localhost:7244/ws")?;
    
    // Use FIFOCore in async context
    let runtime = fifocore.runtime();
    runtime.spawn(async move {
        // Async operations here
        sleep(Duration::from_secs(1)).await;
    });
    
    Ok(())
}
```

## Important Notes

1. **WebSocket Backend**: Automatically reconnects when connections are lost
2. **Message Format**: Uses `rdxcanlink_protocol` for WebSocket message serialization
3. **Thread Safety**: FIFOCore is thread-safe and can be shared across threads
4. **Buffer Management**: Read/Write buffers are managed by the session system
5. **Error Recovery**: Most errors are recoverable; check error types for retry logic

## Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
reduxfifo = { path = "path/to/reduxfifo" }
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Troubleshooting

- **Connection failures**: Check WebSocket URL format and server availability
- **Buffer full errors**: Increase buffer sizes or process messages faster
- **Session errors**: Ensure session is opened before use
- **Bus errors**: Verify bus parameters and backend support 