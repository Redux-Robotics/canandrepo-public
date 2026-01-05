/// OTAv1 indexes.
///
/// OTAv1 is only used by older ESP32-based Canandmags.
/// Newer firmwares implement the faster and more robust OTAv2 transport protocol.
pub mod index {
    /// OTA version index
    pub const OTA_VERSION: u8 = 1;

    pub mod command {
        /// OTA version command.
        pub const VERSION: u8 = 0;
        /// OTA start command
        pub const START: u8 = 1;
        /// Transition to next state
        pub const NEXT: u8 = 2;
        /// Cancel the current operation
        pub const CANCEL: u8 = 3;
        /// Transmit how many bytes have been sent
        pub const TELL: u8 = 4;
    }

    pub mod response {
        /// Operation complete
        pub const COMPLETE: u8 = 1;
        /// Operation also complete
        pub const CONTINUE: u8 = 2;
        /// Error
        pub const ERR: u8 = 3;
    }
}
