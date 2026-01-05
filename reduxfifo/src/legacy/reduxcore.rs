//! ReduxCore emulation core event loop
//!
use std::time::Duration;

use crate::log_error;
use fifocore::{ReadBuffer, ReduxFIFOSessionConfig, Session, fifocore::FIFOCore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BusRequest {
    Open(u16),
    Close(u16),
}
struct BusSession {
    bus_id: u16,
    // gets dropped on done
    _ses: Session,
    buf: ReadBuffer,
}

const BUFFER_SIZE: usize = 128;
pub async fn run_reduxcore(
    fifocore: FIFOCore,
    mut bus_req: tokio::sync::mpsc::Receiver<BusRequest>,
) {
    // ram is free
    //wpihal_rio::threads::set_current_thread_priority(wpihal_rio::threads::ThreadPriority {
    //    priority: 30,
    //    real_time: true,
    //})
    //.ok();

    //let mut rx_notifier = session
    //    .rx_notifier()
    //    .expect("[ReduxCore] FIFO session has already died...?");

    let (send, recv) = tokio::sync::mpsc::channel(BUFFER_SIZE * 4);
    super::put_recv(recv);

    let mut sessions: Vec<BusSession> = Vec::with_capacity(1);
    let mut interval = tokio::time::interval(Duration::from_millis(1));

    loop {
        if let Ok(req) = bus_req.try_recv() {
            match req {
                BusRequest::Open(bus_id) => {
                    if sessions.iter().find(|bs| bs.bus_id == bus_id).is_none() {
                        let (_ses, buf) = open_session(&fifocore, bus_id).await;
                        sessions.push(BusSession { bus_id, _ses, buf });
                    }
                }
                BusRequest::Close(bus_id) => {
                    drop(sessions.extract_if(.., |bs| bs.bus_id == bus_id));
                }
            }
        }

        if let Err(e) = fifocore.read_barrier_multibus(
            sessions
                .iter_mut()
                .map(|bs| &mut core::array::from_mut(&mut bs.buf)[..]),
        ) {
            log_error!("[ReduxCore] failed to read buffer: {e}");
        }

        for bs in &sessions {
            for msg in bs.buf.iter() {
                let _ = send.send(*msg).await;
            }
        }

        interval.tick().await;

        //if rx_notifier.wait_for(|size| *size > 0).await.is_err() {
        //    log_error!("[ReduxCore] Session sender has somehow died!");
        //    break;
        //}
        //let Ok(_) = session.read_barrier(&mut next_buf) else {
        //    break;
        //};
    }
}

async fn open_session(fifocore: &FIFOCore, bus_id: u16) -> (Session, ReadBuffer) {
    let mut tried_to_open = false;
    let session_cfg = ReduxFIFOSessionConfig::new(0x0e0000, 0xff0000);
    let session = loop {
        match fifocore.open_managed_session(bus_id, BUFFER_SIZE as u32, session_cfg) {
            Ok(ses) => {
                break ses;
            }
            Err(e) => {
                if !tried_to_open {
                    log_error!(
                        "[ReduxCore] Failed to open stream session: {e}, retrying until bus comes online..."
                    );
                    tried_to_open = true;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    };
    let next_buf = session.read_buffer(BUFFER_SIZE as u32);
    (session, next_buf)
}
