# MIL-STD-1553B Protocol Parser

<a href="https://crates.io/crates/milstd1553b-parser">
    <img style="display: inline!important" src="https://img.shields.io/crates/v/damilstd1553b-parsertaviz.svg"></img>
</a>
<a href="https://docs.rs/milstd1553b-parser">
    <img style="display: inline!important" src="https://docs.rs/milstd1553b-parser/badge.svg"></img>
</a>
<a href="https://docs.rs/milstd1553b-parser">
    <img style="display: inline!important" src="https://img.shields.io/crates/d/milstd1553b-parser"></img>
</a>

A comprehensive Rust library for parsing and handling MIL-STD-1553B military data bus protocol. This library provides robust encoding/decoding, message parsing, and protocol validation for aerospace and military avionics systems.

## Overview

MIL-STD-1553B is a serial synchronous data bus specification used in military avionics and aerospace systems. Key characteristics:

- **Clock Frequency**: 1 MHz
- **Word Length**: 20 bits (1 start + 16 data + 1 parity + 2 sync bits)
- **Encoding**: Manchester encoding (Thomas variant)
- **Bus Architecture**: Dual-redundant (Bus A and Bus B)
- **Network Topology**: 1 Bus Controller (BC) + up to 30 Remote Terminals (RTs) + Bus Monitors (BMs)
- **Protocol**: Command/Response based synchronous communication

## Project Structure

```
src/
├── lib.rs                 # Library root and module definitions
├── core.rs                # Core types: Word, Address, Bus, WordType
├── encoding.rs            # Manchester encoding/decoding
├── error.rs               # Error types and result handling
├── message.rs             # Message types: Command, Status, StatusFlags
├── parser.rs              # High-level message parsing
├── protocol.rs            # Protocol validation and bus management
└── main.rs                # Example usage
```

## Module Descriptions

### `core` Module
Defines the fundamental data structures:
- **`Word`**: 20-bit word with validation and parity checking
- **`Address`**: Device address (0-31, with 31 as broadcast)
- **`Bus`**: Bus identification (BusA or BusB)
- **`WordType`**: Word classification (Command, Data, Status, ModeCode)

**Key Features**:
- Odd parity validation over 17 bits (start + 16 data bits)
- Parity calculation utilities
- Safe address construction with bounds checking

### `encoding` Module
Manchester encoding/decoding for MIL-STD-1553B:
- **`ManchesterEncoder`**: Converts bits to Manchester-encoded bytes
- **`ManchesterDecoder`**: Decodes Manchester-encoded bytes back to bits
- **`ManchesterType`**: Configurable encoding variants (IEEE, Thomas)

**Features**:
- Bit-level encoding/decoding
- Word-level (20-bit) encoding/decoding
- Error detection for invalid Manchester patterns

### `error` Module
Comprehensive error handling:
- **`ParseError`**: Custom error type with specific error variants
- **`Result<T>`**: Type alias for convenient error handling

**Error Types**:
- `InvalidWord`: Malformed word structure
- `ParityError`: Parity validation failure
- `InvalidAddress`: Address out of range
- `InvalidMessageType`: Unexpected message type
- `InsufficientData`: Not enough data to parse
- `InvalidManchesterEncoding`: Invalid Manchester pattern

### `message` Module
Protocol message definitions:
- **`Command`**: Command word from Bus Controller
  - Address (4 bits)
  - T/R bit (Transmit/Receive)
  - Sub-address (5 bits)
  - Word count (6 bits)

- **`StatusWord`**: Status word from Remote Terminal
  - Address (4 bits)
  - Status flags (5 bits)
  - Error code (7 bits)

- **`StatusFlags`**: Individual status indicators
  - Reserved, Subsystem, Busy, Broadcast, Parity Error

- **`ModeCode`**: Special mode commands
  - Synchronize, SelfTest, VectorWord, etc.

- **`Message`**: Complete message envelope
  - CommandData: Command with optional data words
  - Status: Status word response
  - CommandOnly: Command without data

### `parser` Module
High-level message parsing:
- **`Parser`**: Main parser for converting raw data to messages
  - Bus-specific context
  - Word parsing from Manchester-encoded bytes
  - Transaction parsing (command + response)
  - Message encoding/decoding

- **`ParserBuilder`**: Fluent builder pattern for parser configuration

- **`Transaction`**: Parsed transaction with timestamp and context

### `protocol` Module
Protocol-level handling and validation:
- **`BusController`**: Manages bus operations and RT state
  - Remote Terminal registration and tracking
  - Transaction recording and statistics
  - Response timeout management

- **`RemoteTerminal`**: RT state information
  - Address, state, error count, success count
  - Last seen timestamp
  - Response status checking

- **`RTStats`**: Statistics for Remote Terminals
  - Error rates and transaction counts
  - Current state and responsiveness

- **`MessageValidator`**: Protocol validation utilities
  - Address validation
  - Word count limits
  - Sub-address range checking

## Design Principles

### 1. **Type Safety**
- Strong typing for addresses, word types, and message components
- Enum-based variants instead of magic numbers
- Result-based error handling avoiding panics

### 2. **Validation**
- Parity checking on word construction
- Address bounds validation
- Redundant format checks

### 3. **Modularity**
- Clean separation of concerns (encoding, parsing, protocol)
- No cross-module dependencies on private details
- Extensible through public interfaces

### 4. **Manchester Encoding**
- Proper Thomas variant implementation (0=high-to-low, 1=low-to-high)
- Bit-accurate encoding/decoding
- Roundtrip consistency

### 5. **Protocol State Management**
- Bus Controller tracks RT status
- Supports both successful and failed transactions
- Statistics collection for monitoring

## Usage Examples

### Creating and Encoding a Command

```rust
use milstd1553b_parser::{Address, Command, CommandType, SubAddress, Parser, core::Bus};

// Create a parser
let parser = Parser::new(Bus::BusA);

// Build a command
let command = Command::new(
    Address::new(5)?,                          // Target RT address
    CommandType::Transmit,                     // Transmit data
    SubAddress::new(10)?,                      // Sub-address
    16,                                        // 16 data words
)?;

// Encode to Manchester bytes
let encoded_bytes = parser.encode_command(&command)?;
println!("Encoded: {:?}", encoded_bytes);
```

### Parsing Status Words

```rust
use milstd1553b_parser::{Word, WordType, message::{StatusWord, StatusFlags}, Address};

// Create a status word
let flags = StatusFlags::new(false, true, false, false, false);
let status = StatusWord::new(Address::new(3)?, flags, 0x42)?;

// Encode it
let word = status.to_word()?;
println!("Status Word: {:?}", word);

// Decode it back
let decoded = StatusWord::from_word(&word)?;
assert_eq!(status, decoded);
```

### Bus Controller State Management

```rust
use milstd1553b_parser::protocol::BusController;
use milstd1553b_parser::core::{Bus, Address};

// Create a bus controller
let mut bc = BusController::new(Bus::BusA);

// Register remote terminals
bc.register_rts(&[0, 5, 10, 15])?;

// Record transactions
bc.record_rt_success(Address::new(5)?)?;
bc.record_rt_error(Address::new(5)?)?;

// Get statistics
let stats = bc.get_rt_stats(Address::new(5)?);
println!("RT-5 Error Rate: {:.1}%", stats.unwrap().error_rate * 100.0);
```

## Features

- ✅ Full MIL-STD-1553B word structure support
- ✅ Manchester encoding/decoding (Thomas variant)
- ✅ Odd parity calculation and validation
- ✅ Command and Status word parsing
- ✅ Bus Controller and Remote Terminal management
- ✅ Transaction-level parsing
- ✅ Comprehensive error handling
- ✅ Optional serialization support (with `serde` feature)

## Optional Features

### Serialization
Enable JSON serialization/deserialization:
```bash
cargo build --features serde
```

This adds `serde::Serialize` and `serde::Deserialize` derives to data structures.

## Testing

Run the comprehensive test suite:
```bash
cargo test
```

Tests cover:
- Word creation and parity validation
- Manchester encoding/decoding roundtrips
- Command/Status word encode/decode
- Parser functionality
- Bus Controller operations
- Error handling

## Design Decisions

### Word Encoding Format
The 20-bit word is structured as:
- **Bit 0**: Start bit (always 0)
- **Bits 16-1**: Data (16 bits)
- **Bit 17**: Parity (odd parity over bits 16-0)
- **Bits 19-18**: Sync/Reserved

### Parity Scheme
Uses **odd parity** over 17 bits (start bit + 16 data bits). This ensures:
- Even number of 0s in the data → parity bit = 1
- Odd number of 0s in the data → parity bit = 0
- Total count of 1s is always odd

### Command/Status Format
Maps protocol data into 16-bit data field:
- **Bits 15-12**: Address (4 bits)
- **Bits 11-7**: Function bits (5 bits)
- **Bits 6-0**: Operand (7 bits)

Note: This limits some fields compared to full MIL-STD-1553B. Can be extended for specific implementations.

## Constants

The library exports key specification constants:

```rust
pub mod spec {
    pub const CLOCK_FREQUENCY: u32 = 1_000_000;        // 1 MHz
    pub const WORD_LENGTH: usize = 20;                  // 20 bits
    pub const MAX_REMOTE_TERMINALS: u8 = 30;            // 30 RTs + BC
    pub const MANCHESTER_BITS_PER_WORD: usize = 40;    // 2x encoding
    pub const MAX_DATA_WORD_RATE: u32 = 1_000_000;     // 1 Mbps
}
```

## Future Enhancements

- [ ] Real-time bus monitoring capabilities
- [ ] Support for Bus Monitor parsing
- [ ] Advanced error recovery mechanisms
- [ ] Performance optimizations for high-speed parsing
- [ ] Additional serialization formats (CBOR, MessagePack)
- [ ] Async/await support for bus operations
- [ ] Extended mode codes implementation
- [ ] Signal integrity analysis

## License

This project is provided as-is for educational and commercial use in MIL-STD-1553B implementations.

## Contributing

Contributions are welcome! Areas for improvement:
- Additional test coverage
- Performance optimizations
- Extended protocol features
- Documentation improvements

## References

- MIL-STD-1553B: Aircraft Internal Time Division Command/Response Multiplex Data Bus
- Manchester Encoding and Decoding
- Aerospace data bus systems and avionics integration
