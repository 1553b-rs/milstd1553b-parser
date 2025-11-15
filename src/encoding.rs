//! Manchester encoding and decoding for MIL-STD-1553B

use crate::error::{ParseError, Result};

/// Manchester encoding type for MIL-STD-1553B
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManchesterType {
    /// IEEE 802.3 Manchester: 0 = low-to-high, 1 = high-to-low
    Ieee,
    /// Thomas Manchester: 0 = high-to-low, 1 = low-to-high
    Thomas,
}

impl ManchesterType {
    /// Get the default Manchester encoding for MIL-STD-1553B
    pub fn milstd() -> Self {
        ManchesterType::Thomas
    }
}

/// Manchester encoder for MIL-STD-1553B
pub struct ManchesterEncoder;

impl ManchesterEncoder {
    /// Encode a single bit using Thomas Manchester encoding (MIL-STD-1553B standard)
    ///
    /// 0 = high-to-low transition (1, 0)
    /// 1 = low-to-high transition (0, 1)
    pub fn encode_bit(bit: bool) -> u8 {
        match bit {
            false => 0b10, // high-to-low
            true => 0b01,  // low-to-high
        }
    }

    /// Encode multiple bits (little-endian bit order)
    ///
    /// Returns a vector of bytes representing the Manchester-encoded data
    pub fn encode_bits(data: &[bool]) -> Vec<u8> {
        let mut result = Vec::with_capacity((data.len() + 3) / 4);
        let mut byte = 0u8;
        let mut bit_pos = 0;

        for &bit in data {
            let encoded = Self::encode_bit(bit);
            byte |= (encoded & 0x3) << bit_pos;
            bit_pos += 2;

            if bit_pos == 8 {
                result.push(byte);
                byte = 0;
                bit_pos = 0;
            }
        }

        if bit_pos > 0 {
            result.push(byte);
        }

        result
    }

    /// Encode a word (20 bits) into Manchester-encoded data
    pub fn encode_word(word: u32) -> Vec<u8> {
        let mut bits = Vec::with_capacity(20);
        for i in 0..20 {
            bits.push(((word >> i) & 1) != 0);
        }
        Self::encode_bits(&bits)
    }
}

/// Manchester decoder for MIL-STD-1553B
pub struct ManchesterDecoder;

impl ManchesterDecoder {
    /// Decode a single Manchester-encoded bit pair (Thomas encoding)
    ///
    /// Returns Ok(bit) on valid encoding, Err on invalid pattern
    pub fn decode_bit(pair: u8) -> Result<bool> {
        match pair & 0x3 {
            0b01 => Ok(true),   // low-to-high = 1
            0b10 => Ok(false),  // high-to-low = 0
            _ => Err(ParseError::invalid_manchester(
                format!("Invalid Manchester pattern: {:#04b}", pair),
            )),
        }
    }

    /// Decode a sequence of Manchester-encoded bits
    ///
    /// Each byte contains 4 Manchester-encoded bits (2 bits per bit)
    pub fn decode_bits(data: &[u8], num_bits: usize) -> Result<Vec<bool>> {
        let mut result = Vec::with_capacity(num_bits);

        for &byte in data {
            for shift in (0..8).step_by(2) {
                if result.len() >= num_bits {
                    break;
                }
                let pair = (byte >> shift) & 0x3;
                result.push(Self::decode_bit(pair)?);
            }

            if result.len() >= num_bits {
                break;
            }
        }

        if result.len() < num_bits {
            return Err(ParseError::insufficient_data(
                format!("Expected {} bits, got {}", num_bits, result.len()),
            ));
        }

        Ok(result)
    }

    /// Decode a Manchester-encoded word (20 bits)
    ///
    /// Expects 5 bytes (40 bits) of Manchester-encoded data
    pub fn decode_word(data: &[u8]) -> Result<u32> {
        if data.len() < 5 {
            return Err(ParseError::insufficient_data(
                format!("Expected 5 bytes for word, got {}", data.len()),
            ));
        }

        let bits = Self::decode_bits(data, 20)?;
        let mut word = 0u32;

        for (i, &bit) in bits.iter().enumerate() {
            if bit {
                word |= 1 << i;
            }
        }

        Ok(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manchester_encode_bit() {
        assert_eq!(ManchesterEncoder::encode_bit(false), 0b10);
        assert_eq!(ManchesterEncoder::encode_bit(true), 0b01);
    }

    #[test]
    fn test_manchester_decode_bit() {
        assert_eq!(ManchesterDecoder::decode_bit(0b10).unwrap(), false);
        assert_eq!(ManchesterDecoder::decode_bit(0b01).unwrap(), true);
        assert!(ManchesterDecoder::decode_bit(0b00).is_err());
        assert!(ManchesterDecoder::decode_bit(0b11).is_err());
    }

    #[test]
    fn test_manchester_encode_decode_roundtrip() {
        let original_bits = vec![true, false, true, false, true, true, false, false];
        let encoded = ManchesterEncoder::encode_bits(&original_bits);
        let decoded = ManchesterDecoder::decode_bits(&encoded, original_bits.len()).unwrap();

        assert_eq!(decoded, original_bits);
    }

    #[test]
    fn test_manchester_word_encode_decode() {
        let original_word = 0x12345u32;
        let encoded = ManchesterEncoder::encode_word(original_word);
        let decoded = ManchesterDecoder::decode_word(&encoded).unwrap();

        assert_eq!(decoded, original_word);
    }

    #[test]
    fn test_manchester_invalid_pattern() {
        let invalid_data = vec![0b00, 0b11];
        let result = ManchesterDecoder::decode_bits(&invalid_data, 2);
        assert!(result.is_err());
    }
}
