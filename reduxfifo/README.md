# ReduxFIFO

This is a message interposer layer used by ReduxLib and Alchemist.
Its job is to maintain connections to message buses and then collate and serve those packets to client applications,
allowing code running on all relevant operating systems to communicate with Redux devices.

## Building



## License

* Most of this project is LGPL v3.0 only.
* The xtask used to build this project is `MIT OR Apache-2.0`.
  You are strongly encouraged to use it in derivative projects 

## Client interfaces

There are three supported client interfaces:

* The Rust API through `reduxfifo::fifocore::FIFOCore`
* The FFI/ReduxCore API, used for ReduxLib
* The CANLink websocket API through port 7244

These each allow packets to get served to end applications.

## Backend buses

A message bus is a channel through which messages can be transmitted and received.
Message buses may not have to exist at time of an open request or during operation; rather, if they _can_ exist, ReduxFIFO will wait until they do and try to maintain that connection.
This allows sessions to persist through unreliable connections.

Supported message backends are:

* roboRIO FRCNetComm (if compiled with the `wpihal-rio` feature)
* SocketCAN (if running on Linux)
* SLCAN (all platforms, please don't use this if you can help it)
* Redux devices using RdxUSB (all platforms)
* WebSocket CANLink servers (all platforms)

Planned backends are:
* Generic gs_usb CAN adapters via nusb

## Things todo:

* [ ] Simulation/replay interposer
* [x] Ability to open a bus from the websocket interface
* [ ] Logging?
* [x] Firmware flashing utility
* [ ] Message dumping ability
* [x] BRS/RTR support (proper)

## Simulation/replay

Basic ideas:
* Record frame read barriers
* Record outbound writes
* Key these to a timestamp key
* ?????