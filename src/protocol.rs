//! Protocol-level handling and validation for MIL-STD-1553B

use crate::core::{Address, Bus};
use crate::error::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// State of a Remote Terminal device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RTState {
    /// Device is idle
    Idle,
    /// Device is busy processing
    Busy,
    /// Device reported an error
    Error,
    /// Device is not responding
    NoResponse,
}

/// Information about a Remote Terminal
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemoteTerminal {
    /// Address of the RT
    pub address: Address,
    /// Current state
    pub state: RTState,
    /// Last communication time
    pub last_seen: Option<Instant>,
    /// Number of errors detected
    pub error_count: u32,
    /// Number of successful transactions
    pub success_count: u32,
}

impl RemoteTerminal {
    /// Create a new Remote Terminal info
    pub fn new(address: Address) -> Self {
        RemoteTerminal {
            address,
            state: RTState::Idle,
            last_seen: None,
            error_count: 0,
            success_count: 0,
        }
    }

    /// Record a successful transaction
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.state = RTState::Idle;
        self.last_seen = Some(Instant::now());
    }

    /// Record a failed transaction
    pub fn record_error(&mut self) {
        self.error_count += 1;
        self.state = RTState::Error;
        self.last_seen = Some(Instant::now());
    }

    /// Check if device is responding (seen within timeout)
    pub fn is_responding(&self, timeout: Duration) -> bool {
        match self.last_seen {
            Some(instant) => instant.elapsed() < timeout,
            None => false,
        }
    }
}

/// Bus Controller state and management
#[derive(Debug)]
pub struct BusController {
    /// Bus identifier
    pub bus: Bus,
    /// Remote terminals on this bus
    remote_terminals: HashMap<u8, RemoteTerminal>,
    /// Expected response timeout
    pub response_timeout: Duration,
}

impl BusController {
    /// Create a new Bus Controller for a bus
    pub fn new(bus: Bus) -> Self {
        BusController {
            bus,
            remote_terminals: HashMap::new(),
            response_timeout: Duration::from_micros(12), // Typical 12 microseconds
        }
    }

    /// Register a Remote Terminal
    pub fn register_rt(&mut self, address: Address) -> Result<()> {
        if !address.is_remote_terminal() {
            return Err(crate::error::ParseError::invalid_address(
                "Address must be a valid RT (0-29)".to_string(),
            ));
        }
        self.remote_terminals
            .insert(address.value(), RemoteTerminal::new(address));
        Ok(())
    }

    /// Register multiple Remote Terminals
    pub fn register_rts(&mut self, addresses: &[u8]) -> Result<()> {
        for &addr in addresses {
            self.register_rt(Address::new(addr)?)?;
        }
        Ok(())
    }

    /// Get Remote Terminal info
    pub fn get_rt(&self, address: Address) -> Option<&RemoteTerminal> {
        self.remote_terminals.get(&address.value())
    }

    /// Get mutable Remote Terminal info
    pub fn get_rt_mut(&mut self, address: Address) -> Option<&mut RemoteTerminal> {
        self.remote_terminals.get_mut(&address.value())
    }

    /// List all registered Remote Terminals
    pub fn list_rts(&self) -> Vec<&RemoteTerminal> {
        self.remote_terminals.values().collect()
    }

    /// Get all responding Remote Terminals
    pub fn get_responding_rts(&self) -> Vec<&RemoteTerminal> {
        self.remote_terminals
            .values()
            .filter(|rt| rt.is_responding(self.response_timeout))
            .collect()
    }

    /// Get total number of RTs
    pub fn rt_count(&self) -> usize {
        self.remote_terminals.len()
    }

    /// Record a successful transaction with an RT
    pub fn record_rt_success(&mut self, address: Address) -> Result<()> {
        if let Some(rt) = self.get_rt_mut(address) {
            rt.record_success();
            Ok(())
        } else {
            Err(crate::error::ParseError::invalid_address(
                "RT not registered".to_string(),
            ))
        }
    }

    /// Record a failed transaction with an RT
    pub fn record_rt_error(&mut self, address: Address) -> Result<()> {
        if let Some(rt) = self.get_rt_mut(address) {
            rt.record_error();
            Ok(())
        } else {
            Err(crate::error::ParseError::invalid_address(
                "RT not registered".to_string(),
            ))
        }
    }

    /// Get statistics for a Remote Terminal
    pub fn get_rt_stats(&self, address: Address) -> Option<RTStats> {
        self.get_rt(address).map(|rt| RTStats {
            address: rt.address,
            state: rt.state,
            success_count: rt.success_count,
            error_count: rt.error_count,
            error_rate: if rt.success_count + rt.error_count > 0 {
                rt.error_count as f32 / (rt.success_count + rt.error_count) as f32
            } else {
                0.0
            },
            is_responding: rt.is_responding(self.response_timeout),
        })
    }

    /// Get statistics for all Remote Terminals
    pub fn get_all_stats(&self) -> Vec<RTStats> {
        self.list_rts()
            .into_iter()
            .filter_map(|rt| self.get_rt_stats(rt.address))
            .collect()
    }
}

/// Statistics for a Remote Terminal
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RTStats {
    /// Address of the RT
    pub address: Address,
    /// Current state
    pub state: RTState,
    /// Number of successful transactions
    pub success_count: u32,
    /// Number of failed transactions
    pub error_count: u32,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f32,
    /// Whether the RT is currently responding
    pub is_responding: bool,
}

/// Message validator for protocol compliance
pub struct MessageValidator;

impl MessageValidator {
    /// Validate message addressing
    pub fn validate_address(address: Address) -> Result<()> {
        // All addresses 0-31 are valid in different contexts
        // This is a placeholder for more complex validation
        let _ = address;
        Ok(())
    }

    /// Validate word count
    pub fn validate_word_count(count: u16) -> Result<()> {
        if count > 32 {
            return Err(crate::error::ParseError::validation_error(
                "Word count exceeds maximum of 32".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate sub-address
    pub fn validate_sub_address(sub_addr: u8) -> Result<()> {
        if sub_addr > 31 {
            return Err(crate::error::ParseError::validation_error(
                "Sub-address out of range [0, 31]".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rt_creation() {
        let rt = RemoteTerminal::new(Address::new(5).unwrap());
        assert_eq!(rt.address.value(), 5);
        assert_eq!(rt.state, RTState::Idle);
        assert_eq!(rt.error_count, 0);
        assert_eq!(rt.success_count, 0);
    }

    #[test]
    fn test_bc_register_rt() -> Result<()> {
        let mut bc = BusController::new(Bus::BusA);
        bc.register_rt(Address::new(5)?)?;
        assert_eq!(bc.rt_count(), 1);
        assert!(bc.get_rt(Address::new(5)?).is_some());
        Ok(())
    }

    #[test]
    fn test_bc_register_multiple_rts() -> Result<()> {
        let mut bc = BusController::new(Bus::BusA);
        bc.register_rts(&[0, 5, 10, 15])?;
        assert_eq!(bc.rt_count(), 4);
        Ok(())
    }

    #[test]
    fn test_rt_recording() -> Result<()> {
        let mut rt = RemoteTerminal::new(Address::new(5)?);
        rt.record_success();
        assert_eq!(rt.success_count, 1);
        assert_eq!(rt.state, RTState::Idle);

        rt.record_error();
        assert_eq!(rt.error_count, 1);
        assert_eq!(rt.state, RTState::Error);
        Ok(())
    }

    #[test]
    fn test_message_validator() -> Result<()> {
        MessageValidator::validate_word_count(16)?;
        assert!(MessageValidator::validate_word_count(33).is_err());

        MessageValidator::validate_sub_address(31)?;
        assert!(MessageValidator::validate_sub_address(32).is_err());
        Ok(())
    }
}
