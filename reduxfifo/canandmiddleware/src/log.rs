#![allow(unused)]

macro_rules! log_trace {
    ($($arg:expr),*) => (log::trace!(target: "canandmiddleware", $($arg),*));
}
pub(crate) use log_trace;

macro_rules! log_debug {
    ($($arg:expr),*) => (log::debug!(target: "canandmiddleware", $($arg),*));
}
pub(crate) use log_debug;

macro_rules! log_info {
    ($($arg:expr),*) => (log::info!(target: "canandmiddleware", $($arg),*));
}
pub(crate) use log_info;

macro_rules! log_warn {
    ($($arg:expr),*) => (log::warn!(target: "canandmiddleware", $($arg),*));
}
pub(crate) use log_warn;

macro_rules! log_error {
    ($($arg:expr),*) => (log::error!(target: "canandmiddleware", $($arg),*));
}
pub(crate) use log_error;
