//! Error types for MIL-STD-1553B parsing

use thiserror::Error;

/// Result type for MIL-STD-1553B operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// Error types encountered during MIL-STD-1553B parsing and validation
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid word format or structure
    #[error("Invalid word: {0}")]
    InvalidWord(String),

    /// Parity check failed
    #[error("Parity error: {0}")]
    ParityError(String),

    /// Invalid address specified
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid message type
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),

    /// Insufficient data to parse
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    /// Invalid Manchester encoding
    #[error("Invalid Manchester encoding: {0}")]
    InvalidManchesterEncoding(String),

    /// Invalid command format
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// Invalid response format
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Status word error
    #[error("Status error: {0}")]
    StatusError(String),

    /// Bus error detected
    #[error("Bus error: {0}")]
    BusError(String),

    /// Generic parsing error
    #[error("Parse error: {0}")]
    ParseFailed(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl ParseError {
    /// Create a new InvalidWord error
    pub fn invalid_word(msg: impl Into<String>) -> Self {
        ParseError::InvalidWord(msg.into())
    }

    /// Create a new ParityError
    pub fn parity_error(msg: impl Into<String>) -> Self {
        ParseError::ParityError(msg.into())
    }

    /// Create a new InvalidAddress error
    pub fn invalid_address(msg: impl Into<String>) -> Self {
        ParseError::InvalidAddress(msg.into())
    }

    /// Create a new InvalidMessageType error
    pub fn invalid_message_type(msg: impl Into<String>) -> Self {
        ParseError::InvalidMessageType(msg.into())
    }

    /// Create a new InsufficientData error
    pub fn insufficient_data(msg: impl Into<String>) -> Self {
        ParseError::InsufficientData(msg.into())
    }

    /// Create a new InvalidManchesterEncoding error
    pub fn invalid_manchester(msg: impl Into<String>) -> Self {
        ParseError::InvalidManchesterEncoding(msg.into())
    }

    /// Create a new ParseFailed error
    pub fn parse_failed(msg: impl Into<String>) -> Self {
        ParseError::ParseFailed(msg.into())
    }

    /// Create a new ValidationError
    pub fn validation_error(msg: impl Into<String>) -> Self {
        ParseError::ValidationError(msg.into())
    }

    /// Create a new InvalidCommand error
    pub fn invalid_command(msg: impl Into<String>) -> Self {
        ParseError::InvalidCommand(msg.into())
    }

    /// Create a new InvalidResponse error
    pub fn invalid_response(msg: impl Into<String>) -> Self {
        ParseError::InvalidResponse(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ParseError::invalid_word("test");
        assert!(err.to_string().contains("Invalid word"));
    }
}
