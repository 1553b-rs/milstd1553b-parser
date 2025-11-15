//! High-level message parser for MIL-STD-1553B protocol

use crate::core::{Bus, Word, WordType};
use crate::encoding::{ManchesterDecoder, ManchesterEncoder};
use crate::error::Result;
use crate::message::{Command, Message, StatusWord};

/// A parsed MIL-STD-1553B transaction
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transaction {
    /// Bus on which the transaction occurred
    pub bus: Bus,
    /// The command/status message
    pub message: Message,
    /// Timestamp of the transaction (microseconds, if available)
    pub timestamp_us: Option<u64>,
}

/// MIL-STD-1553B protocol parser
pub struct Parser {
    /// Current bus context
    pub bus: Bus,
}

impl Parser {
    /// Create a new parser
    pub fn new(bus: Bus) -> Self {
        Parser { bus }
    }

    /// Parse a single word from Manchester-encoded bytes
    ///
    /// Expects 5 bytes (40 bits) of Manchester-encoded data representing 20 bits
    pub fn parse_word(&self, data: &[u8]) -> Result<Word> {
        let word_value = ManchesterDecoder::decode_word(data)?;
        // Try to determine word type from context or structure
        self.identify_word_type_and_create(word_value)
    }

    /// Parse multiple words from raw data
    pub fn parse_words(&self, data: &[u8]) -> Result<Vec<Word>> {
        let mut words = Vec::new();
        let mut offset = 0;

        while offset + 5 <= data.len() {
            let word = self.parse_word(&data[offset..offset + 5])?;
            words.push(word);
            offset += 5;
        }

        Ok(words)
    }

    /// Parse a command-response transaction
    ///
    /// A typical transaction consists of:
    /// 1. Command word (from Bus Controller)
    /// 2. Optional data words (if receive command)
    /// 3. Status word (from Remote Terminal)
    /// 4. Optional response data words
    pub fn parse_transaction(&self, data: &[u8]) -> Result<Transaction> {
        let words = self.parse_words(data)?;

        if words.is_empty() {
            return Err(crate::error::ParseError::insufficient_data(
                "No words to parse".to_string(),
            ));
        }

        // Identify the message structure
        let message = self.parse_message(&words)?;

        Ok(Transaction {
            bus: self.bus,
            message,
            timestamp_us: None,
        })
    }

    /// Parse a message from a sequence of words
    fn parse_message(&self, words: &[Word]) -> Result<Message> {
        if words.is_empty() {
            return Err(crate::error::ParseError::insufficient_data(
                "Empty word sequence".to_string(),
            ));
        }

        let first_word = words[0];

        match first_word.word_type() {
            WordType::Command => {
                let command = Command::from_word(&first_word)?;

                // Check if there are data words following
                if words.len() > 1 {
                    let mut data_words = Vec::new();
                    for word in &words[1..] {
                        if word.word_type() == WordType::Data {
                            data_words.push(*word);
                        } else {
                            break; // Stop at non-data word
                        }
                    }

                    if !data_words.is_empty() {
                        Ok(Message::CommandData {
                            command,
                            data_words,
                        })
                    } else {
                        Ok(Message::CommandOnly(command))
                    }
                } else {
                    Ok(Message::CommandOnly(command))
                }
            }
            WordType::Status => {
                let status = StatusWord::from_word(&first_word)?;
                Ok(Message::Status(status))
            }
            _ => Err(crate::error::ParseError::invalid_message_type(
                "Message must start with command or status word".to_string(),
            )),
        }
    }

    /// Identify word type and create a Word with appropriate type
    fn identify_word_type_and_create(&self, word_value: u32) -> Result<Word> {
        // Simple heuristic: analyze the word structure
        // In a real implementation, this might be passed as a parameter
        // or inferred from protocol context

        // For now, create as data word - caller should specify type
        Word::new(word_value, WordType::Data)
    }

    /// Encode and transmit a command
    pub fn encode_command(&self, command: &Command) -> Result<Vec<u8>> {
        let word = command.to_word()?;
        let encoded = ManchesterEncoder::encode_word(word.data());
        Ok(encoded)
    }

    /// Encode a status word
    pub fn encode_status(&self, status: &StatusWord) -> Result<Vec<u8>> {
        let word = status.to_word()?;
        let encoded = ManchesterEncoder::encode_word(word.data());
        Ok(encoded)
    }

    /// Encode data words
    pub fn encode_data_words(&self, data: &[u16]) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();

        for &value in data {
            let parity = Word::calculate_parity(value) as u32;
            let word_value = (parity << 17) | ((value as u32) << 1);
            let word = Word::new(word_value, WordType::Data)?;

            let word_encoded = ManchesterEncoder::encode_word(word.data());
            encoded.extend(word_encoded);
        }

        Ok(encoded)
    }
}

/// Builder for parsing MIL-STD-1553B data streams
pub struct ParserBuilder {
    bus: Bus,
}

impl ParserBuilder {
    /// Create a new parser builder
    pub fn new() -> Self {
        ParserBuilder { bus: Bus::BusA }
    }

    /// Set the bus
    pub fn with_bus(mut self, bus: Bus) -> Self {
        self.bus = bus;
        self
    }

    /// Build the parser
    pub fn build(self) -> Parser {
        Parser::new(self.bus)
    }
}

impl Default for ParserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Address;
    use crate::message::{CommandType, SubAddress};

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(Bus::BusA);
        assert_eq!(parser.bus, Bus::BusA);
    }

    #[test]
    fn test_parser_builder() {
        let parser = ParserBuilder::new().with_bus(Bus::BusB).build();
        assert_eq!(parser.bus, Bus::BusB);
    }

    #[test]
    fn test_encode_command() -> Result<()> {
        let parser = Parser::new(Bus::BusA);
        let cmd = Command::new(
            Address::new(5)?,
            CommandType::Transmit,
            SubAddress::new(10)?,
            16,
        )?;

        let encoded = parser.encode_command(&cmd)?;
        assert!(!encoded.is_empty());
        Ok(())
    }

    #[test]
    fn test_parse_word_roundtrip() -> Result<()> {
        let parser = Parser::new(Bus::BusA);

        // Create a word
        let original_data = 0x12345u32;
        let parity = Word::calculate_parity(original_data as u16) as u32;
        let word_value = (parity << 17) | (original_data << 1);
        let original_word = Word::new(word_value, WordType::Data)?;

        // Encode it
        let encoded = ManchesterEncoder::encode_word(original_word.data());

        // Decode it
        let decoded_word = parser.parse_word(&encoded)?;

        // Verify
        assert_eq!(decoded_word.data(), original_word.data());
        Ok(())
    }
}
