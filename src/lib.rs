//! # MIL-STD-1553B Protocol Parser
//!
//! A comprehensive Rust library for parsing and handling MIL-STD-1553B military data bus protocol.
//!
//! MIL-STD-1553B is a serial synchronous data bus specification used in military avionics
//! and aerospace systems. This library provides:
//!
//! - Encoding/decoding of Manchester-encoded data
//! - Message parsing and construction
//! - Protocol validation
//! - Error handling
//!
//! ## Features
//!
//! - `serde`: Enable serialization/deserialization support
//!
//! ## Example
//!
//! ```
//! use milstd1553b_parser::{Word, WordType};
//!
//! // Create a data word
//! let word = Word::new(0x1234, WordType::Data)?;
//! println!("Word: {:?}", word);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod core;
pub mod encoding;
pub mod error;
pub mod message;
pub mod parser;
pub mod protocol;

pub use core::{Address, Word, WordType};
pub use error::{ParseError, Result};
pub use message::{Command, Message};
pub use parser::Parser;

/// The MIL-STD-1553B specification constants
pub mod spec {
    /// Clock frequency in Hz
    pub const CLOCK_FREQUENCY: u32 = 1_000_000; // 1 MHz

    /// Word length in bits
    pub const WORD_LENGTH: usize = 20;

    /// Maximum number of Remote Terminals
    pub const MAX_REMOTE_TERMINALS: u8 = 30;

    /// Manchester encoding uses 2 bits per data bit
    pub const MANCHESTER_BITS_PER_WORD: usize = WORD_LENGTH * 2;

    /// Maximum data word rate in bits per second
    pub const MAX_DATA_WORD_RATE: u32 = 1_000_000; // 1 Mbps
}
