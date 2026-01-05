use std::mem::ManuallyDrop;

/// Contains definitions of the error type.
pub mod error;

/// Core FIFO event loop
pub mod fifocore;

/// Backends to the FIFO event loop
pub mod backends;

/// Data structures shared between this and FFI
pub mod data;
pub use data::*;

/// Timing
pub mod timebase;

/// Loggers
pub mod logger;

mod log;
pub use crate::fifocore::FIFOCore;
pub(crate) use crate::log::*;

/// Struct representing data that ReduxFIFO will write onto bus.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WriteBuffer {
    pub(crate) meta: Box<ReduxFIFOWriteBuffer>,
    pub(crate) msgs: Vec<ReduxFIFOMessage>,
}

impl WriteBuffer {
    pub fn new(bus_id: u16, mut messages: Vec<ReduxFIFOMessage>) -> Self {
        messages.shrink_to_fit();
        Self {
            meta: Box::new(ReduxFIFOWriteBuffer {
                bus_id: bus_id as u32,
                status: 0,
                messages_written: 0,
                length: messages.len() as u32,
            }),
            msgs: messages,
        }
    }
    pub(crate) fn ready_for_write(&mut self) {
        self.meta.messages_written = 0;
        self.meta.status = 0;
    }
    pub(crate) fn set_status(&mut self, status: Result<(), error::Error>) {
        self.meta.status = match status {
            Ok(()) => error::REDUXFIFO_OK,
            Err(e) => e as i32,
        };
    }

    pub fn empty(bus_id: u16, count: usize) -> Self {
        Self::new(bus_id, vec![ReduxFIFOMessage::default(); count])
    }

    /// Conjure a write buffer from raw pointers.
    /// Useful with FFI.
    ///
    /// # Safety
    ///
    /// Both pointers should be allocated by ReduxFIFO, or at least via the [`Box`] and [`Vec`] semantics.
    /// If they are not, things will probably segfault very quickly.
    /// These aren't even _null-checked_.
    pub unsafe fn from_parts(
        metadata: *mut ReduxFIFOWriteBuffer,
        messages: *mut ReduxFIFOMessage,
    ) -> Self {
        unsafe {
            let metadata = Box::from_raw(metadata);
            let length = metadata.length as usize;
            let messages = Vec::from_raw_parts(messages, length, length);
            Self {
                meta: metadata,
                msgs: messages,
            }
        }
    }

    /// Split a write buffer into raw pointery bits.
    /// Useful with FFI.
    ///
    pub unsafe fn into_parts(self) -> (*mut ReduxFIFOWriteBuffer, *mut ReduxFIFOMessage, usize) {
        let mut messages = ManuallyDrop::new(self.msgs);
        let len = messages.len();
        let msg_ptr = messages.as_mut_ptr();

        (Box::into_raw(self.meta), msg_ptr, len)
    }

    /// Slice view of messages.
    pub fn messages(&mut self) -> &[ReduxFIFOMessage] {
        &self.msgs
    }

    /// Mutable slice view of messages.
    /// Can be used to rewrite the buffer payload
    pub fn messages_mut(&mut self) -> &mut [ReduxFIFOMessage] {
        &mut self.msgs
    }

    /// Split the write buffer into its components.
    pub fn split(self) -> (Box<ReduxFIFOWriteBuffer>, Vec<ReduxFIFOMessage>) {
        (self.meta, self.msgs)
    }

    pub fn messages_written(&self) -> usize {
        self.meta.messages_written as usize
    }

    pub fn status(&self) -> Result<(), error::Error> {
        error::Error::from_code(self.meta.status)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReadBuffer {
    pub(crate) session: ReduxFIFOSession,
    pub(crate) meta: Box<ReduxFIFOReadBuffer>,
    pub(crate) msgs: Vec<ReduxFIFOMessage>,
}

impl ReadBuffer {
    pub fn new(session: ReduxFIFOSession, size: u32) -> Self {
        Self {
            session,
            meta: Box::new(ReduxFIFOReadBuffer {
                session,
                status: error::REDUXFIFO_OK,
                next_idx: 0,
                valid_length: 0,
                max_length: size,
            }),
            msgs: vec![ReduxFIFOMessage::default(); size as usize],
        }
    }

    /// Conjure a read buffer from raw pointers.
    /// Useful with FFI.
    ///
    /// # Safety
    ///
    /// Both pointers should be allocated by ReduxFIFO, or at least via the [`Box`] and [`Vec`] semantics.
    /// If they are not, things will probably segfault very quickly.
    /// These aren't even _null-checked_.
    pub unsafe fn from_parts(
        metadata: *mut ReduxFIFOReadBuffer,
        messages: *mut ReduxFIFOMessage,
    ) -> Self {
        unsafe {
            let metadata = Box::from_raw(metadata);
            let length = metadata.max_length as usize;
            let messages = Vec::from_raw_parts(messages, length, length);
            Self {
                session: metadata.session,
                meta: metadata,
                msgs: messages,
            }
        }
    }

    /// Split a read buffer into raw pointery bits.
    /// Useful with FFI.
    ///
    pub unsafe fn into_parts(self) -> (*mut ReduxFIFOReadBuffer, *mut ReduxFIFOMessage, usize) {
        let mut messages = ManuallyDrop::new(self.msgs);
        let len = messages.len();
        let msg_ptr = messages.as_mut_ptr();

        (Box::into_raw(self.meta), msg_ptr, len)
    }

    pub(crate) fn ready_for_read(&mut self) {
        self.meta.max_length = self.msgs.len() as u32;
        self.meta.next_idx = 0;
        self.meta.session = self.session;
        self.meta.status = error::REDUXFIFO_OK;
        self.meta.valid_length = 0;
    }

    pub fn set_status(&mut self, status: Result<(), error::Error>) {
        self.meta.status = match status {
            Ok(()) => error::REDUXFIFO_OK,
            Err(e) => e as i32,
        };
    }
    /// add a message to the ringbuffer
    pub fn add_message(&mut self, msg: ReduxFIFOMessage) {
        self.msgs[self.meta.next_idx as usize] = msg;
        self.meta.valid_length = self.meta.max_length.min(self.meta.valid_length + 1);
        self.meta.next_idx = (self.meta.next_idx + 1) % self.meta.max_length;
    }

    pub fn clear_messages(&mut self) {
        self.msgs.clear();
        self.meta.valid_length = 0;
    }

    pub fn session(&self) -> ReduxFIFOSession {
        self.session
    }

    /// Returns a slice over just the valid messages, regardless of message chronology.
    pub fn unordered_valid_messages(&self) -> &[ReduxFIFOMessage] {
        &self.msgs[..self.meta.valid_length as usize]
    }

    pub fn iter(&self) -> ValidMessages<'_> {
        let valid_length = self.meta.valid_length;
        if valid_length < self.meta.max_length {
            ValidMessages {
                buf: self,
                pos: 0,
                left: valid_length as usize,
            }
        } else {
            ValidMessages {
                buf: self,
                pos: self.meta.next_idx as usize,
                left: valid_length as usize,
            }
        }
    }
}

/// Iterator over a [`ReduxFIFOReadBuffer`]'s valid messages, from oldest to newest.
pub struct ValidMessages<'a> {
    /// The buffer reference
    buf: &'a ReadBuffer,
    /// The next position to read from
    pos: usize,
    /// The number of elements left to read.
    left: usize,
}

impl<'a> Iterator for ValidMessages<'a> {
    type Item = &'a ReduxFIFOMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if self.left == 0 {
            None
        } else {
            let pos = self.pos;
            self.left -= 1;
            self.pos = (self.pos + 1) % (self.buf.meta.valid_length as usize);
            Some(&self.buf.unordered_valid_messages()[pos])
        }
    }
}

/// Managed session handle.
/// When dropped, it will be closed.
pub struct Session {
    fifocore: FIFOCore,
    session: ReduxFIFOSession,
}
impl Session {
    pub unsafe fn wrap(fifocore: FIFOCore, session: ReduxFIFOSession) -> Self {
        Self { fifocore, session }
    }

    pub fn read_buffer(&self, size: u32) -> ReadBuffer {
        ReadBuffer::new(self.session, size)
    }

    pub fn read_barrier(&self, data: &mut ReadBuffer) -> Result<(), error::Error> {
        self.fifocore
            .read_barrier(self.session.bus_id(), core::array::from_mut(data))
    }

    pub fn rx_notifier(&self) -> Result<tokio::sync::watch::Receiver<u32>, error::Error> {
        self.fifocore.rx_notifier(self.session)
    }

    pub fn session(&self) -> ReduxFIFOSession {
        self.session
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.fifocore.close_session(self.session);
    }
}
