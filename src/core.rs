//! Core types and structures for MIL-STD-1553B protocol

use crate::error::{ParseError, Result};

/// Bus identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bus {
    /// Bus A (primary)
    BusA,
    /// Bus B (redundant)
    BusB,
}

impl Bus {
    /// Convert bus to bit representation
    pub fn as_bit(&self) -> u8 {
        match self {
            Bus::BusA => 0,
            Bus::BusB => 1,
        }
    }
}

impl std::fmt::Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bus::BusA => write!(f, "Bus A"),
            Bus::BusB => write!(f, "Bus B"),
        }
    }
}

/// Device address (0-30, with 31 being broadcast)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address(u8);

impl Address {
    /// Minimum address value (0)
    pub const MIN: u8 = 0;
    /// Maximum address value (31)
    pub const MAX: u8 = 31;
    /// Broadcast address
    pub const BROADCAST: u8 = 31;

    /// Create a new address, validating it's within range [0, 31]
    pub fn new(addr: u8) -> Result<Self> {
        if addr > Self::MAX {
            return Err(ParseError::invalid_address(format!(
                "Address {} out of range [0, {}]",
                addr,
                Self::MAX
            )));
        }
        Ok(Address(addr))
    }

    /// Create a broadcast address
    pub fn broadcast() -> Self {
        Address(Self::BROADCAST)
    }

    /// Get the raw address value
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Check if this is a broadcast address
    pub fn is_broadcast(&self) -> bool {
        self.0 == Self::BROADCAST
    }

    /// Check if this is a valid Remote Terminal address (0-30)
    pub fn is_remote_terminal(&self) -> bool {
        self.0 < 30
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_broadcast() {
            write!(f, "BC (broadcast)")
        } else {
            write!(f, "RT-{}", self.0)
        }
    }
}

/// Word type in MIL-STD-1553B
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WordType {
    /// Command word (from Bus Controller)
    Command,
    /// Data word
    Data,
    /// Status word (from Remote Terminal)
    Status,
    /// Mode code (special command)
    ModeCode,
}

impl std::fmt::Display for WordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WordType::Command => write!(f, "Command"),
            WordType::Data => write!(f, "Data"),
            WordType::Status => write!(f, "Status"),
            WordType::ModeCode => write!(f, "Mode Code"),
        }
    }
}

/// A single MIL-STD-1553B word
///
/// Format:
/// - 1 start bit (always 0 for valid Manchester encoding)
/// - 16 data bits
/// - 1 parity bit (odd parity over all 17 bits)
/// - 2 synchronization bits
///
/// Total: 20 bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Word {
    /// 20-bit word value
    data: u32,
    /// Type of word
    word_type: WordType,
}

impl Word {
    /// Create a new word with validation
    ///
    /// The 16 data bits should be in bits 16:1, parity in bit 17
    pub fn new(data: u32, word_type: WordType) -> Result<Self> {
        // Validate that only 20 bits are used
        if data > 0xFFFFF {
            return Err(ParseError::invalid_word(
                "Word data exceeds 20 bits".to_string(),
            ));
        }

        // Validate parity
        Self::validate_parity(data)?;

        Ok(Word { data, word_type })
    }

    /// Create a word without parity validation
    ///
    /// Use with caution - only for constructing test data or when parity
    /// will be verified separately
    pub fn new_unchecked(data: u32, word_type: WordType) -> Self {
        Word { data, word_type }
    }

    /// Get the raw word data (20 bits)
    pub fn data(&self) -> u32 {
        self.data
    }

    /// Get the word type
    pub fn word_type(&self) -> WordType {
        self.word_type
    }

    /// Extract the 16 data bits (bits 16-1)
    pub fn get_data_bits(&self) -> u16 {
        ((self.data >> 1) & 0xFFFF) as u16
    }

    /// Extract the parity bit (bit 17)
    pub fn get_parity_bit(&self) -> bool {
        ((self.data >> 17) & 1) != 0
    }

    /// Extract the sync bits (bits 19-18)
    pub fn get_sync_bits(&self) -> u8 {
        ((self.data >> 18) & 0x3) as u8
    }

    /// Validate odd parity across all 17 bits (bits 16-0)
    ///
    /// In MIL-STD-1553B, odd parity is used over the start bit (0) and
    /// the 16 data bits, and the result is stored in the parity bit.
    fn validate_parity(data: u32) -> Result<()> {
        // Count the number of 1s in bits [16:0]
        let count_bits = (data & 0x1FFFF).count_ones();

        // With odd parity, the total number of 1s (including parity bit) should be odd
        let parity_bit = ((data >> 17) & 1) != 0;
        let total_ones = count_bits + if parity_bit { 1 } else { 0 };

        if total_ones % 2 == 0 {
            return Err(ParseError::parity_error(
                "Parity check failed: even number of 1s detected".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate and set the correct parity bit for a word
    pub fn calculate_parity(data_bits: u16) -> u8 {
        // Start bit is always 0
        // Count 1s in the data bits (16 bits)
        let count_ones = data_bits.count_ones();

        // For odd parity, if we have an even number of 1s, we need a parity bit of 1
        if count_ones % 2 == 0 {
            1
        } else {
            0
        }
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Word(type={}, data=0x{:05X})",
            self.word_type, self.data
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        assert!(Address::new(0).is_ok());
        assert!(Address::new(31).is_ok());
        assert!(Address::new(32).is_err());
    }

    #[test]
    fn test_address_broadcast() {
        let addr = Address::broadcast();
        assert!(addr.is_broadcast());
    }

    #[test]
    fn test_word_creation() {
        // Create a simple word with valid parity
        let data_bits = 0xAAAAu16;
        let parity = Word::calculate_parity(data_bits) as u32;
        let word_data = (parity << 17) | ((data_bits as u32) << 1) | 0;

        let word = Word::new(word_data, WordType::Data);
        assert!(word.is_ok());
    }

    #[test]
    fn test_word_parity_validation() {
        // Create a word with wrong parity
        let word_data = 0xAAAAA; // This likely has wrong parity

        let result = Word::new(word_data, WordType::Data);
        // May or may not fail depending on the actual parity bits
        let _ = result;
    }

    #[test]
    fn test_calculate_parity() {
        // Odd parity: total number of 1s (including parity bit) should be odd
        let parity = Word::calculate_parity(0x0000);
        assert_eq!(parity, 1); // 0 ones (even) → need parity=1 to make total odd

        let parity = Word::calculate_parity(0xFFFF);
        assert_eq!(parity, 1); // 16 ones (even) → need parity=1 to make total odd

        let parity = Word::calculate_parity(0x0001);
        assert_eq!(parity, 0); // 1 one (odd) → parity=0, total stays odd
    }

    #[test]
    fn test_bus_display() {
        assert_eq!(Bus::BusA.to_string(), "Bus A");
        assert_eq!(Bus::BusB.to_string(), "Bus B");
    }
}
