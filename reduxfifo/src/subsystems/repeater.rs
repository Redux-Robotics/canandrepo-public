use std::{time::Duration, u64};

use tokio::{sync::watch, task::JoinHandle};

use fifocore::{FIFOCore, ReduxFIFOMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeaterState {
    /// The message to send
    message: ReduxFIFOMessage,
    /// How often to send it
    period: Duration,
    /// How many times the message will be sent before the repeater task runs out.
    times: u64,
}

pub struct Repeater {
    control: watch::Sender<RepeaterState>,
    handle: JoinHandle<()>,
}

impl Drop for Repeater {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl Repeater {
    pub fn new_stopped(fifocore: FIFOCore) -> Repeater {
        Self::new(
            ReduxFIFOMessage::default(),
            Duration::from_secs(u32::MAX as u64),
            0,
            fifocore,
        )
    }

    pub fn new(
        message: ReduxFIFOMessage,
        period: Duration,
        times: u64,
        fifocore: FIFOCore,
    ) -> Repeater {
        let (control, watcher) = watch::channel(RepeaterState {
            message,
            period,
            times,
        });
        let handle = fifocore
            .runtime()
            .spawn(run_repeater(fifocore.clone(), watcher));
        Repeater { control, handle }
    }

    pub fn update(&self, message: ReduxFIFOMessage, period: Duration, times: u64) {
        self.control.send_replace(RepeaterState {
            message,
            period,
            times,
        });
    }
}

pub async fn run_repeater(fifocore: FIFOCore, mut watcher: watch::Receiver<RepeaterState>) {
    let mut state = *watcher.borrow_and_update();
    loop {
        tokio::select! {
            _ = tokio::time::sleep(state.period) => {
                state.times = state.times.saturating_sub(1);
            }
            maybe_state = watcher.changed() => {
                if maybe_state.is_err() {
                    return;
                }
                state = *watcher.borrow_and_update();
            }
        }
        if state.times > 0 {
            let _ = fifocore.write_single(&state.message);
        } else {
            state.period = Duration::from_secs(u32::MAX as u64);
        }
    }
}
