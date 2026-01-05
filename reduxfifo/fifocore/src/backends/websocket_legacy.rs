use std::{sync::Arc, time::Duration};

use crate::backends::{Backend, BackendOpen, SessionTable};
use crate::error::Error;
use crate::{ReduxFIFOMessage, ReduxFIFOSessionConfig, log_debug, log_error, log_trace, timebase};
use futures::{SinkExt, StreamExt};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use url::Url;

#[derive(Debug)]
pub struct WebSocketBackend {
    base_url: String,
    #[allow(unused)]
    bus_id: u16,
    tx_sender: mpsc::Sender<ReduxFIFOMessage>,
    read_task: tokio::task::JoinHandle<()>,
}

impl WebSocketBackend {
    fn parse_params(s: &str) -> Result<String, Error> {
        // websocket:ws://host:port/path or websocket:wss://host:port/path
        let (backend_type, url) = s.split_once(':').ok_or(Error::InvalidBus)?;
        if backend_type != "websocket" {
            return Err(Error::BusNotSupported);
        }
        Ok(url.to_string())
    }

    pub fn open(
        bus_id: u16,
        params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<Mutex<SessionTable<()>>>,
    ) -> Result<Self, Error> {
        log_debug!("[ws] open websocket: {bus_id}");
        let url = Self::parse_params(params)?;

        log_debug!("websocket url: {url}");

        // Validate URL format
        let _parsed_url = Url::parse(&url).map_err(|_| Error::InvalidBus)?;

        let (tx_sender, tx_receiver) = mpsc::channel::<ReduxFIFOMessage>(100);

        let read_task = runtime.spawn(Self::websocket_loop(
            url.clone(),
            bus_id,
            ses_table,
            tx_receiver,
        ));

        Ok(Self {
            base_url: url,
            bus_id,
            tx_sender,
            read_task,
        })
    }

    async fn websocket_loop(
        base_url: String,
        bus_id: u16,
        ses_table: Arc<Mutex<SessionTable<()>>>,
        mut tx_receiver: mpsc::Receiver<ReduxFIFOMessage>,
    ) {
        log_trace!("websocket: start new eventloop for {}", base_url);

        // This is a simplified approach - we'll connect to the base URL
        // and handle filtering in the RX loop based on session configurations
        loop {
            let Ok((ws_stream, _)) = connect_async(&base_url).await else {
                log_error!("websocket: Failed to connect to {}", base_url);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            };

            log_trace!("websocket: connected to {}", base_url);

            let (ws_tx, ws_rx) = ws_stream.split();

            // Create a channel for the TX task to communicate back
            let (tx_done_tx, tx_done_rx) = tokio::sync::oneshot::channel();

            // Spawn TX task
            let tx_task = tokio::spawn(Self::websocket_tx_loop(ws_tx, tx_receiver, tx_done_tx));

            // Spawn RX task
            let rx_task = tokio::spawn(Self::websocket_rx_loop(ws_rx, ses_table.clone(), bus_id));

            // Wait for either task to complete
            tokio::select! {
                result = tx_task => {
                    if let Err(e) = result {
                        log_error!("websocket: TX task failed: {:?}", e);
                    }
                }
                result = rx_task => {
                    if let Err(e) = result {
                        log_error!("websocket: RX task failed: {:?}", e);
                    }
                }
            }

            // Get the receiver back from the TX task
            tx_receiver = match tx_done_rx.await {
                Ok(receiver) => receiver,
                Err(_) => {
                    log_error!("websocket: Failed to get receiver back from TX task");
                    break;
                }
            };

            log_trace!("websocket: connection lost, reconnecting...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn websocket_tx_loop(
        mut ws_tx: futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            WsMessage,
        >,
        mut tx_receiver: mpsc::Receiver<ReduxFIFOMessage>,
        tx_done_tx: tokio::sync::oneshot::Sender<mpsc::Receiver<ReduxFIFOMessage>>,
    ) {
        while let Some(msg) = tx_receiver.recv().await {
            let mut message_bytes = Vec::with_capacity(msg.data_size.min(8) as usize + 4);
            message_bytes.extend_from_slice(&msg.message_id.to_le_bytes());
            message_bytes.extend_from_slice(&msg.data[0..msg.data_size.min(8) as usize]);

            let ws_msg = WsMessage::Binary(message_bytes.into());
            if let Err(e) = ws_tx.send(ws_msg).await {
                log_error!("websocket: Failed to send message: {:?}", e);
                break;
            }
        }

        // Send the receiver back to the main loop
        let _ = tx_done_tx.send(tx_receiver);
    }

    async fn websocket_rx_loop(
        mut ws_rx: futures::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        ses_table: Arc<Mutex<SessionTable<()>>>,
        bus_id: u16,
    ) {
        while let Some(msg_result) = ws_rx.next().await {
            let Ok(msg) = msg_result else {
                log_error!("websocket: Failed to receive message");
                continue;
            };

            let data = msg.into_data();
            let timestamp = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let message_id = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            let data = &data[8..];
            let mut array = [0u8; 64];
            let bytes_to_copy = data.len().min(array.len());
            array[..bytes_to_copy].copy_from_slice(&data[..bytes_to_copy]);

            let mut redux_msg = ReduxFIFOMessage {
                message_id: message_id,
                bus_id: bus_id, // Use our bus_id, not the one from the message
                flags: 0,
                data_size: data.len() as u8,
                timestamp: timestamp as u64,
                data: array,
            };

            // Update timestamp if not provided
            if redux_msg.timestamp == 0 {
                redux_msg.timestamp = timebase::now_us() as u64;
            }

            let mut ses_lock = ses_table.lock();
            ses_lock.ingest_message(redux_msg);
            drop(ses_lock);
        }
    }
}

impl Backend for WebSocketBackend {
    type State = ();

    fn start_session(
        &mut self,
        _msg_count: u32,
        _config: &ReduxFIFOSessionConfig,
    ) -> Result<Self::State, Error> {
        Ok(())
    }

    fn write_single(&mut self, msg: &ReduxFIFOMessage) -> Result<(), Error> {
        self.tx_sender
            .try_send(*msg)
            .map_err(|_| Error::BusBufferFull)
    }

    fn params_match(&self, params: &str) -> bool {
        if let Ok(url) = Self::parse_params(params) {
            url == self.base_url
        } else {
            false
        }
    }

    fn max_packet_size(&self) -> usize {
        64
    }
}

impl BackendOpen for WebSocketBackend {
    fn open(
        bus_id: u16,
        params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<Mutex<SessionTable<Self::State>>>,
    ) -> Result<Self, Error> {
        Self::open(bus_id, params, runtime, ses_table)
    }
}

impl Drop for WebSocketBackend {
    fn drop(&mut self) {
        self.read_task.abort();
    }
}
