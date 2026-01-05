use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};

use crate::log::log_error;
use fifocore::{FIFOCore, ReduxFIFOMessage, ReduxFIFOSessionConfig};

pub async fn handle_socket(socket: WebSocket, fifocore: FIFOCore, bus_id: u16) {
    let (sender, receiver) = socket.split();

    let config = ReduxFIFOSessionConfig::new(0x0e0000, 0xff0000);

    let rx = tokio::task::spawn(websocket_tx(sender, fifocore.clone(), bus_id, config));
    let tx = tokio::task::spawn(websocket_rx(receiver, fifocore.clone(), bus_id));

    let _ = futures::future::join(rx, tx).await;
}

pub async fn websocket_tx(
    mut ws_tx: SplitSink<WebSocket, Message>,
    fifocore: FIFOCore,
    bus_id: u16,
    config: ReduxFIFOSessionConfig,
) {
    let session = match fifocore.open_managed_session(bus_id, 256, config) {
        Ok(session) => session,
        Err(e) => {
            log_error!("[ReduxCore] Failed to open websocket session: {e}");
            let _ = ws_tx.close().await.ok();
            return;
        }
    };
    let mut read_buf = session.read_buffer(256);

    let mut interval = tokio::time::interval(Duration::from_millis(5));
    loop {
        interval.tick().await;
        if let Err(e) = session.read_barrier(&mut read_buf) {
            log_error!("[ReduxCore] Read session failed: {e}");
            let _ = ws_tx.close().await;
            return;
        }
        let mut errored = None;

        for msg in read_buf.iter() {
            let rx_msg = rdxcanlink_protocol::CANLinkRxMessage {
                message_id: msg.message_id,
                bus_id: msg.bus_id,
                flags: msg.flags as u16,
                timestamp: msg.timestamp,
                data: msg.data,
                data_size: msg.data_size as usize,
            };
            let outbound = Message::binary::<Vec<u8>>(rx_msg.into());
            if let Err(e) = ws_tx.feed(outbound).await {
                errored = Some(e);
                break;
            }
        }

        if let Some(e) = errored.or(ws_tx.flush().await.err()) {
            log_error!("[ReduxCore] Websocket TX closed: {e}");
            let _ = ws_tx.close().await;
            // session gets dropped on close
            return;
        }
    }
}

pub async fn websocket_rx(mut ws_rx: SplitStream<WebSocket>, fifocore: FIFOCore, bus_id: u16) {
    loop {
        match ws_rx.next().await {
            Some(Ok(Message::Binary(msg))) => {
                let Ok(data) = rdxcanlink_protocol::CANLinkTxMessage::try_from(&*msg) else {
                    continue;
                };

                // we force the bus id to avoid footguns
                let msg = ReduxFIFOMessage::id_data(
                    bus_id,
                    data.message_id,
                    data.data,
                    data.data_size as u8,
                    data.flags as u8,
                );
                let _ = fifocore.write_single(&msg);
            }
            Some(Err(e)) => {
                log_error!("[ReduxCore] Websocket RX closed: {e}");
                return;
            }
            Some(Ok(Message::Close(..))) | None => {
                return;
            }
            Some(Ok(..)) => {
                continue;
            }
        }
    }
}
