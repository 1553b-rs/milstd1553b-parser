//! Message types and structures for MIL-STD-1553B protocol

use crate::core::{Address, Word, WordType};
use crate::error::{ParseError, Result};

/// Sub-address for Read/Write operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubAddress(u8);

impl SubAddress {
    /// Create a new sub-address (0-31)
    pub fn new(addr: u8) -> Result<Self> {
        if addr > 31 {
            return Err(ParseError::invalid_address(format!(
                "Sub-address {} out of range [0, 31]",
                addr
            )));
        }
        Ok(SubAddress(addr))
    }

    /// Get the raw sub-address value
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// Command type in a command word
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandType {
    /// Transmit (Remote Terminal sends data)
    Transmit,
    /// Receive (Remote Terminal receives data)
    Receive,
}

/// Mode code command (special commands sent to specific addresses)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModeCode {
    /// Synchronize (broadcast mode code)
    Synchronize = 0,
    /// Transmit Status Word
    TransmitStatusWord = 1,
    /// Initiate Self Test
    InitiateSelfTest = 2,
    /// Transmit Last Command Word
    TransmitLastCommandWord = 3,
    /// Transmit Built-In Test Result
    TransmitBuiltInTestResult = 4,
    /// Synchronize (alternate)
    SynchronizeAlt = 5,
    /// Transmit Vector Word
    TransmitVectorWord = 6,
    /// Synchronize (alternate 2)
    SynchronizeAlt2 = 7,
    /// Transmit Last Data Word
    TransmitLastDataWord = 8,
}

impl TryFrom<u8> for ModeCode {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(ModeCode::Synchronize),
            1 => Ok(ModeCode::TransmitStatusWord),
            2 => Ok(ModeCode::InitiateSelfTest),
            3 => Ok(ModeCode::TransmitLastCommandWord),
            4 => Ok(ModeCode::TransmitBuiltInTestResult),
            5 => Ok(ModeCode::SynchronizeAlt),
            6 => Ok(ModeCode::TransmitVectorWord),
            7 => Ok(ModeCode::SynchronizeAlt2),
            8 => Ok(ModeCode::TransmitLastDataWord),
            _ => Err(ParseError::invalid_message_type(format!(
                "Unknown mode code: {}",
                value
            ))),
        }
    }
}

/// A MIL-STD-1553B command word
///
/// Format:
/// - Bits 19-16: Address (0-31)
/// - Bit 15: Transmit/Receive flag
/// - Bits 14-10: Sub-address or Mode Code
/// - Bits 9-0: Data word count or mode code data
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Command {
    /// Address of the target device
    pub address: Address,
    /// Transmit or Receive
    pub command_type: CommandType,
    /// Sub-address (5 bits)
    pub sub_address: SubAddress,
    /// Data word count (10 bits, 0-20, 0 means 32 words)
    pub word_count: u16,
}

impl Command {
    /// Create a new command
    pub fn new(
        address: Address,
        command_type: CommandType,
        sub_address: SubAddress,
        word_count: u16,
    ) -> Result<Self> {
        if word_count > 32 {
            return Err(ParseError::invalid_command(format!(
                "Word count {} exceeds maximum of 32",
                word_count
            )));
        }

        Ok(Command {
            address,
            command_type,
            sub_address,
            word_count,
        })
    }

    /// Encode command as a word
    pub fn to_word(&self) -> Result<Word> {
        let mut word = 0u32;

        // Address (bits 15-12, occupying the high nibble of data)
        word |= (self.address.value() as u32 & 0x0F) << 12;

        // Transmit/Receive bit (bit 11)
        word |= match self.command_type {
            CommandType::Transmit => 0x0800,
            CommandType::Receive => 0x0000,
        };

        // Sub-address (bits 10-6)
        word |= (self.sub_address.value() as u32 & 0x1F) << 6;

        // Word count (bits 5-0)
        word |= (self.word_count & 0x3F) as u32;

        // Shift to data position (bits 16-1) and add parity
        let data_in_position = word << 1; // Now in bits 16-1
        let parity = Word::calculate_parity(word as u16) as u32;
        let final_word = data_in_position | (parity << 17);

        Ok(Word::new_unchecked(final_word, WordType::Command))
    }

    /// Decode command from a word
    pub fn from_word(word: &Word) -> Result<Self> {
        if word.word_type() != WordType::Command {
            return Err(ParseError::invalid_message_type(
                "Expected command word".to_string(),
            ));
        }

        let data = word.data() >> 1; // Remove start bit
        let address = Address::new(((data >> 12) & 0x0F) as u8)?;
        let command_type = if (data & 0x0800) != 0 {
            CommandType::Transmit
        } else {
            CommandType::Receive
        };
        let sub_address = SubAddress::new(((data >> 6) & 0x1F) as u8)?;
        let word_count = (data & 0x3F) as u16;

        Ok(Command {
            address,
            command_type,
            sub_address,
            word_count: if word_count == 0 { 32 } else { word_count },
        })
    }
}

/// A MIL-STD-1553B status word
///
/// Format (from Remote Terminal):
/// - Bits 19-16: Address
/// - Bits 15-11: Status flags
/// - Bits 10-0: Message error code (11 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatusWord {
    /// Address of the responding device
    pub address: Address,
    /// Status flags
    pub flags: StatusFlags,
    /// Message error code
    pub error_code: u16,
}

/// Status flags in a MIL-STD-1553B status word
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatusFlags {
    /// Reserved flag
    pub reserved: bool,
    /// Subsystem flag
    pub subsystem_flag: bool,
    /// Busy flag
    pub busy: bool,
    /// BCast (broadcast) flag
    pub broadcast: bool,
    /// Parity error flag
    pub parity_error: bool,
}

impl StatusFlags {
    /// Create a new status flags struct
    pub fn new(reserved: bool, subsystem: bool, busy: bool, broadcast: bool, parity: bool) -> Self {
        StatusFlags {
            reserved,
            subsystem_flag: subsystem,
            busy,
            broadcast,
            parity_error: parity,
        }
    }

    /// Encode flags as bits
    fn encode(&self) -> u8 {
        let mut flags = 0u8;
        if self.reserved {
            flags |= 0x10;
        }
        if self.subsystem_flag {
            flags |= 0x08;
        }
        if self.busy {
            flags |= 0x04;
        }
        if self.broadcast {
            flags |= 0x02;
        }
        if self.parity_error {
            flags |= 0x01;
        }
        flags
    }

    /// Decode flags from bits
    fn decode(bits: u8) -> Self {
        StatusFlags {
            reserved: (bits & 0x10) != 0,
            subsystem_flag: (bits & 0x08) != 0,
            busy: (bits & 0x04) != 0,
            broadcast: (bits & 0x02) != 0,
            parity_error: (bits & 0x01) != 0,
        }
    }
}

impl StatusWord {
    /// Create a new status word
    pub fn new(address: Address, flags: StatusFlags, error_code: u16) -> Result<Self> {
        if error_code > 0x7FF {
            return Err(ParseError::invalid_response(format!(
                "Error code {} exceeds 11 bits",
                error_code
            )));
        }

        Ok(StatusWord {
            address,
            flags,
            error_code,
        })
    }

    /// Encode status word as a word
    pub fn to_word(&self) -> Result<Word> {
        let mut word = 0u32;

        // Address (bits 15-12)
        word |= (self.address.value() as u32 & 0x0F) << 12;

        // Status flags (bits 11-7)
        word |= (self.flags.encode() as u32 & 0x1F) << 7;

        // Error code (bits 6-0)
        word |= (self.error_code & 0x7F) as u32;

        // Shift to data position (bits 16-1) and add parity
        let data_in_position = word << 1; // Now in bits 16-1
        let parity = Word::calculate_parity(word as u16) as u32;
        let final_word = data_in_position | (parity << 17);

        Ok(Word::new_unchecked(final_word, WordType::Status))
    }

    /// Decode status word from a word
    pub fn from_word(word: &Word) -> Result<Self> {
        if word.word_type() != WordType::Status {
            return Err(ParseError::invalid_message_type(
                "Expected status word".to_string(),
            ));
        }

        let data = word.data() >> 1; // Remove start bit
        let address = Address::new(((data >> 12) & 0x0F) as u8)?;
        let flags = StatusFlags::decode(((data >> 7) & 0x1F) as u8);
        let error_code = (data & 0x7F) as u16;

        Ok(StatusWord {
            address,
            flags,
            error_code,
        })
    }
}

/// A complete message in MIL-STD-1553B protocol
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Message {
    /// Command followed by optional data words
    CommandData {
        command: Command,
        data_words: Vec<Word>,
    },
    /// Status word response
    Status(StatusWord),
    /// Just a command word (for transmit commands)
    CommandOnly(Command),
}

impl Message {
    /// Get the address associated with this message
    pub fn address(&self) -> Address {
        match self {
            Message::CommandData { command, .. } => command.address,
            Message::Status(status) => status.address,
            Message::CommandOnly(command) => command.address,
        }
    }

    /// Get the number of data words if present
    pub fn data_word_count(&self) -> Option<usize> {
        match self {
            Message::CommandData { data_words, .. } => Some(data_words.len()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subaddress_creation() {
        assert!(SubAddress::new(0).is_ok());
        assert!(SubAddress::new(31).is_ok());
        assert!(SubAddress::new(32).is_err());
    }

    #[test]
    fn test_command_encode_decode() {
        let cmd = Command::new(
            Address::new(5).unwrap(),
            CommandType::Transmit,
            SubAddress::new(10).unwrap(),
            16,
        )
        .unwrap();

        let word = cmd.to_word().unwrap();
        let decoded = Command::from_word(&word).unwrap();

        assert_eq!(cmd, decoded);
    }

    #[test]
    fn test_status_word_encode_decode() {
        let flags = StatusFlags::new(false, true, false, false, false);
        // Error code limited to 7 bits (0-127) due to word structure
        let status = StatusWord::new(Address::new(3).unwrap(), flags, 0x42).unwrap();

        let word = status.to_word().unwrap();
        let decoded = StatusWord::from_word(&word).unwrap();

        assert_eq!(status, decoded);
    }

    #[test]
    fn test_mode_code_conversion() {
        let code: ModeCode = 1u8.try_into().unwrap();
        assert_eq!(code, ModeCode::TransmitStatusWord);

        let result: Result<ModeCode> = 99u8.try_into();
        assert!(result.is_err());
    }
}
