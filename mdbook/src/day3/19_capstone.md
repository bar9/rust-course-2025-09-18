# Chapter 19: Capstone Project - Network Protocol Parser

## Project Overview

Build a TCP-based protocol parser that demonstrates all the Rust concepts you've learned. This project simulates a real-world scenario where you need to handle network communication, parse binary protocols, manage concurrent connections, and ensure robust error handling.

## Learning Objectives
- Apply ownership and borrowing in a real system
- Implement a custom binary protocol
- Handle concurrent TCP connections
- Parse and validate network messages
- Build a complete client-server application
- Optimize for performance and safety

## Protocol Specification

### Message Format

Our custom protocol uses a simple binary format:

```
+--------+--------+--------+--------+
| Magic  | Type   | Length | Payload|
| 2 bytes| 1 byte | 2 bytes| N bytes|
+--------+--------+--------+--------+
```

### Message Types

```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Ping = 0x01,
    Pong = 0x02,
    Echo = 0x03,
    Data = 0x04,
    Error = 0xFF,
}

// Protocol constants
const MAGIC_BYTES: [u8; 2] = [0xCA, 0xFE];
const HEADER_SIZE: usize = 5;
const MAX_PAYLOAD_SIZE: usize = 65535;
```

## Project Structure

```
protocol-parser/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Library with protocol implementation
│   ├── protocol.rs      # Protocol definitions
│   ├── parser.rs        # Message parser
│   ├── server.rs        # TCP server
│   ├── client.rs        # TCP client
│   └── bin/
│       ├── server.rs    # Server binary
│       └── client.rs    # Client binary
└── tests/
    └── integration.rs   # Integration tests
```

## Implementation Guide

### Step 1: Protocol Message Structure

```rust
// src/protocol.rs
use std::io::{self, Read, Write};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Message {
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> Result<Self, String> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(format!("Payload too large: {} bytes", payload.len()));
        }
        Ok(Message { msg_type, payload })
    }
    
    pub fn ping() -> Self {
        Message {
            msg_type: MessageType::Ping,
            payload: vec![],
        }
    }
    
    pub fn pong() -> Self {
        Message {
            msg_type: MessageType::Pong,
            payload: vec![],
        }
    }
    
    pub fn echo(data: Vec<u8>) -> Result<Self, String> {
        Self::new(MessageType::Echo, data)
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        bytes.extend_from_slice(&MAGIC_BYTES);
        bytes.push(self.msg_type as u8);
        bytes.extend_from_slice(&(self.payload.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

impl TryFrom<u8> for MessageType {
    type Error = String;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageType::Ping),
            0x02 => Ok(MessageType::Pong),
            0x03 => Ok(MessageType::Echo),
            0x04 => Ok(MessageType::Data),
            0xFF => Ok(MessageType::Error),
            _ => Err(format!("Invalid message type: 0x{:02x}", value)),
        }
    }
}
```

### Step 2: Message Parser

```rust
// src/parser.rs
use crate::protocol::{Message, MessageType, MAGIC_BYTES, HEADER_SIZE};
use std::io::{self, Read};
use std::convert::TryFrom;

pub struct Parser {
    buffer: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            buffer: Vec::with_capacity(1024),
        }
    }
    
    pub fn parse_message<R: Read>(reader: &mut R) -> io::Result<Message> {
        let mut header = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header)?;
        
        // Validate magic bytes
        if &header[0..2] != MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic bytes",
            ));
        }
        
        // Parse message type
        let msg_type = MessageType::try_from(header[2])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // Parse payload length
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;
        
        // Read payload
        let mut payload = vec![0u8; length];
        reader.read_exact(&mut payload)?;
        
        Ok(Message { msg_type, payload })
    }
    
    pub fn parse_from_bytes(bytes: &[u8]) -> Result<Message, String> {
        if bytes.len() < HEADER_SIZE {
            return Err("Message too short".to_string());
        }
        
        if &bytes[0..2] != MAGIC_BYTES {
            return Err("Invalid magic bytes".to_string());
        }
        
        let msg_type = MessageType::try_from(bytes[2])?;
        let length = u16::from_be_bytes([bytes[3], bytes[4]]) as usize;
        
        if bytes.len() != HEADER_SIZE + length {
            return Err("Invalid message length".to_string());
        }
        
        Ok(Message {
            msg_type,
            payload: bytes[HEADER_SIZE..].to_vec(),
        })
    }
}
```

### Step 3: TCP Server Implementation

```rust
// src/server.rs
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use crate::protocol::{Message, MessageType};
use crate::parser::Parser;

pub struct Server {
    addr: String,
    connections: Arc<Mutex<Vec<TcpStream>>>,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Server {
            addr: addr.to_string(),
            connections: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)?;
        println!("Server listening on {}", self.addr);
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let connections = Arc::clone(&self.connections);
                    thread::spawn(move || {
                        handle_client(stream, connections);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
        
        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, connections: Arc<Mutex<Vec<TcpStream>>>) {
    let peer_addr = stream.peer_addr().unwrap();
    println!("Client connected: {}", peer_addr);
    
    // Add to connections
    {
        let mut conns = connections.lock().unwrap();
        conns.push(stream.try_clone().unwrap());
    }
    
    loop {
        match Parser::parse_message(&mut stream) {
            Ok(message) => {
                println!("Received {:?} from {}", message.msg_type, peer_addr);
                
                let response = match message.msg_type {
                    MessageType::Ping => Message::pong(),
                    MessageType::Echo => Message::echo(message.payload).unwrap(),
                    MessageType::Data => {
                        // Process data
                        println!("Data: {:?}", String::from_utf8_lossy(&message.payload));
                        continue;
                    }
                    _ => continue,
                };
                
                if let Err(e) = stream.write_all(&response.to_bytes()) {
                    eprintln!("Failed to send response: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error parsing message from {}: {}", peer_addr, e);
                break;
            }
        }
    }
    
    println!("Client disconnected: {}", peer_addr);
}
```

### Step 4: TCP Client Implementation

```rust
// src/client.rs
use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use crate::protocol::{Message, MessageType};
use crate::parser::Parser;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect(addr: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        Ok(Client { stream })
    }
    
    pub fn send_message(&mut self, message: Message) -> std::io::Result<()> {
        self.stream.write_all(&message.to_bytes())?;
        self.stream.flush()
    }
    
    pub fn receive_message(&mut self) -> std::io::Result<Message> {
        Parser::parse_message(&mut self.stream)
    }
    
    pub fn ping(&mut self) -> std::io::Result<bool> {
        self.send_message(Message::ping())?;
        let response = self.receive_message()?;
        Ok(response.msg_type == MessageType::Pong)
    }
    
    pub fn echo(&mut self, data: Vec<u8>) -> std::io::Result<Vec<u8>> {
        self.send_message(Message::echo(data.clone()).unwrap())?;
        let response = self.receive_message()?;
        
        if response.msg_type == MessageType::Echo {
            Ok(response.payload)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected Echo response",
            ))
        }
    }
}
```

### Step 5: Binary Executables

```rust
// src/bin/server.rs
use protocol_parser::server::Server;

fn main() {
    let server = Server::new("127.0.0.1:8080");
    
    if let Err(e) = server.run() {
        eprintln!("Server error: {}", e);
    }
}

// src/bin/client.rs
use protocol_parser::client::Client;
use protocol_parser::protocol::Message;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut client = Client::connect("127.0.0.1:8080")?;
    println!("Connected to server");
    
    // Test ping
    if client.ping()? {
        println!("Ping successful!");
    }
    
    // Test echo
    let data = b"Hello, Rust!";
    let echoed = client.echo(data.to_vec())?;
    println!("Echo: {}", String::from_utf8_lossy(&echoed));
    
    // Interactive mode
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        match input {
            "quit" => break,
            "ping" => {
                if client.ping()? {
                    println!("Pong!");
                }
            }
            _ => {
                let echoed = client.echo(input.as_bytes().to_vec())?;
                println!("Server: {}", String::from_utf8_lossy(&echoed));
            }
        }
    }
    
    Ok(())
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_serialization() {
        let msg = Message::ping();
        let bytes = msg.to_bytes();
        assert_eq!(bytes[0..2], MAGIC_BYTES);
        assert_eq!(bytes[2], MessageType::Ping as u8);
    }
    
    #[test]
    fn test_message_parsing() {
        let msg = Message::echo(b"test".to_vec()).unwrap();
        let bytes = msg.to_bytes();
        let parsed = Parser::parse_from_bytes(&bytes).unwrap();
        assert_eq!(parsed.msg_type, MessageType::Echo);
        assert_eq!(parsed.payload, b"test");
    }
}
```

## Performance Optimizations

1. **Buffer Reuse**: Use a single buffer per connection
2. **Zero-Copy Parsing**: Parse messages without copying when possible
3. **Connection Pooling**: Reuse client connections
4. **Async I/O**: Convert to async for better scalability

## Extension Ideas

1. **Authentication**: Add message signing
2. **Compression**: Compress large payloads
3. **Metrics**: Track message rates and latencies
4. **TLS Support**: Add encryption
5. **Protocol Versioning**: Support multiple protocol versions

## Evaluation Criteria

✅ **Correctness**: Messages are parsed and handled correctly
✅ **Error Handling**: No panics, graceful error recovery
✅ **Concurrency**: Multiple clients handled simultaneously
✅ **Performance**: Efficient message processing
✅ **Code Quality**: Idiomatic Rust, proper abstractions
✅ **Testing**: Comprehensive test coverage

## Key Takeaways

✅ **Real-world application** of ownership and borrowing
✅ **Binary protocol parsing** with type safety
✅ **Concurrent network programming** without data races
✅ **Error handling** in network applications
✅ **Testing strategies** for network code
✅ **Performance considerations** in systems programming

---

Congratulations! You've built a complete network application in Rust!: Network Protocol Parser

## Learning Objectives
- Apply all course concepts in a comprehensive, production-ready project
- Design and implement a custom network protocol parser from scratch
- Demonstrate proper error handling, testing, and documentation practices
- Use async/await for concurrent connection handling
- Implement performance monitoring and optimization techniques
- Structure code for maintainability and extensibility

## Project Overview

You'll build a **TCP-based chat protocol parser** that handles multiple concurrent connections, implements a custom binary protocol, and provides both server and client implementations. This project integrates concepts from all previous chapters.

### Project Requirements

1. **Custom Binary Protocol**: Design a simple but extensible chat protocol
2. **Async TCP Server**: Handle multiple concurrent clients
3. **Protocol Parser**: Parse incoming messages with proper error handling
4. **Client Implementation**: Command-line chat client
5. **Comprehensive Testing**: Unit tests, integration tests, and benchmarks
6. **Documentation**: Complete API documentation and usage examples
7. **Monitoring**: Basic metrics and logging

## Protocol Specification

### Message Format

Our chat protocol uses a simple binary format:

```
+--------+--------+--------+--------+
| Version|  Type  |     Length      |
| (1 byte)(1 byte)|    (2 bytes)    |
+--------+--------+--------+--------+
|            Payload              |
|         (Length bytes)          |
+--------+--------+--------+--------+
|          Checksum               |
|         (4 bytes)               |
+--------+--------+--------+--------+
```

**C++/C# Comparison:**
- **C++**: Would use structs with packed attributes, manual endianness handling
- **C#**: BinaryReader/Writer with careful type marshaling
- **Rust**: Type-safe parsing with automatic memory safety and zero-copy optimizations

### Message Types

```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    // Client -> Server
    Connect = 0x01,
    Disconnect = 0x02,
    SendMessage = 0x03,
    JoinRoom = 0x04,
    LeaveRoom = 0x05,
    
    // Server -> Client
    ConnectAck = 0x81,
    MessageBroadcast = 0x82,
    UserJoined = 0x83,
    UserLeft = 0x84,
    Error = 0xFF,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MessageType::Connect),
            0x02 => Some(MessageType::Disconnect),
            0x03 => Some(MessageType::SendMessage),
            0x04 => Some(MessageType::JoinRoom),
            0x05 => Some(MessageType::LeaveRoom),
            0x81 => Some(MessageType::ConnectAck),
            0x82 => Some(MessageType::MessageBroadcast),
            0x83 => Some(MessageType::UserJoined),
            0x84 => Some(MessageType::UserLeft),
            0xFF => Some(MessageType::Error),
            _ => None,
        }
    }
}
```

## Project Structure

```
chat-protocol/
├── Cargo.toml                 # Workspace configuration
├── README.md
├── LICENSE
├── src/
│   └── lib.rs                # Re-exports
├── protocol/                 # Core protocol library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── message.rs        # Message types and parsing
│   │   ├── parser.rs         # Protocol parser
│   │   ├── error.rs          # Error types
│   │   └── codec.rs          # Encoding/decoding
│   ├── tests/
│   │   └── integration_tests.rs
│   └── benches/
│       └── parser_bench.rs
├── server/                   # Chat server
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── server.rs         # Server implementation
│   │   ├── room.rs           # Chat room management
│   │   ├── client.rs         # Client connection handling
│   │   └── metrics.rs        # Performance metrics
│   └── tests/
│       └── server_tests.rs
├── client/                   # Chat client
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── client.rs         # Client implementation
│   │   └── ui.rs            # Command-line interface
│   └── tests/
│       └── client_tests.rs
└── examples/
    ├── basic_client.rs
    └── stress_test.rs
```

## Starter Code Structure

### Workspace Configuration

```toml
# Cargo.toml (root)
[workspace]
members = ["protocol", "server", "client"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
bytes = "1.0"
crc32fast = "1.3"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
```

### Protocol Library

```rust
// protocol/src/lib.rs
//! # Chat Protocol Library
//! 
//! A high-performance, type-safe implementation of a custom chat protocol
//! designed for real-time communication systems.

pub mod message;
pub mod parser;
pub mod error;
pub mod codec;

pub use message::{Message, MessageType, MessagePayload};
pub use parser::{ProtocolParser, ParseResult};
pub use error::{ProtocolError, ParseError};
pub use codec::{MessageCodec, FrameCodec};

/// Protocol version - increment for breaking changes
pub const PROTOCOL_VERSION: u8 = 1;

/// Maximum message size to prevent DoS attacks
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB

/// Default buffer size for parsing
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_constants() {
        assert_eq!(PROTOCOL_VERSION, 1);
        assert!(MAX_MESSAGE_SIZE > 0);
    }
}
```

```rust
// protocol/src/message.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete protocol message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub version: u8,
    pub message_type: MessageType,
    pub payload: MessagePayload,
}

impl Message {
    pub fn new(message_type: MessageType, payload: MessagePayload) -> Self {
        Self {
            version: crate::PROTOCOL_VERSION,
            message_type,
            payload,
        }
    }
    
    /// Calculate the total message size including headers
    pub fn size(&self) -> usize {
        // Version (1) + Type (1) + Length (2) + Payload + Checksum (4)
        8 + self.payload.serialized_size()
    }
    
    /// Validate message constraints
    pub fn validate(&self) -> Result<(), crate::ProtocolError> {
        if self.size() > crate::MAX_MESSAGE_SIZE {
            return Err(crate::ProtocolError::MessageTooLarge {
                size: self.size(),
                max_size: crate::MAX_MESSAGE_SIZE,
            });
        }
        
        self.payload.validate()?;
        Ok(())
    }
}

/// All possible message payloads
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    Connect { username: String },
    Disconnect,
    SendMessage { room: String, content: String },
    JoinRoom { room: String },
    LeaveRoom { room: String },
    
    // Server responses
    ConnectAck { 
        user_id: u32,
        server_info: String,
    },
    MessageBroadcast { 
        user_id: u32,
        username: String,
        room: String,
        content: String,
        timestamp: u64,
    },
    UserJoined { 
        user_id: u32,
        username: String,
        room: String,
    },
    UserLeft { 
        user_id: u32,
        username: String,
        room: String,
    },
    Error { 
        code: u16,
        message: String,
    },
}

impl MessagePayload {
    /// Calculate serialized size for this payload
    pub fn serialized_size(&self) -> usize {
        // This is a simplified calculation
        match self {
            MessagePayload::Connect { username } => 1 + username.len(),
            MessagePayload::Disconnect => 1,
            MessagePayload::SendMessage { room, content } => {
                1 + room.len() + content.len()
            }
            MessagePayload::JoinRoom { room } => 1 + room.len(),
            MessagePayload::LeaveRoom { room } => 1 + room.len(),
            MessagePayload::ConnectAck { user_id: _, server_info } => {
                5 + server_info.len()
            }
            MessagePayload::MessageBroadcast { 
                user_id: _, username, room, content, timestamp: _
            } => {
                13 + username.len() + room.len() + content.len()
            }
            MessagePayload::UserJoined { user_id: _, username, room } => {
                5 + username.len() + room.len()
            }
            MessagePayload::UserLeft { user_id: _, username, room } => {
                5 + username.len() + room.len()
            }
            MessagePayload::Error { code: _, message } => {
                3 + message.len()
            }
        }
    }
    
    /// Validate payload constraints
    pub fn validate(&self) -> Result<(), crate::ProtocolError> {
        match self {
            MessagePayload::Connect { username } => {
                if username.is_empty() || username.len() > 32 {
                    return Err(crate::ProtocolError::InvalidUsername);
                }
            }
            MessagePayload::SendMessage { room, content } => {
                if room.is_empty() || room.len() > 64 {
                    return Err(crate::ProtocolError::InvalidRoomName);
                }
                if content.is_empty() || content.len() > 512 {
                    return Err(crate::ProtocolError::InvalidMessageContent);
                }
            }
            MessagePayload::JoinRoom { room } | MessagePayload::LeaveRoom { room } => {
                if room.is_empty() || room.len() > 64 {
                    return Err(crate::ProtocolError::InvalidRoomName);
                }
            }
            _ => {} // Server messages don't need client-side validation
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let payload = MessagePayload::Connect {
            username: "testuser".to_string(),
        };
        let message = Message::new(MessageType::Connect, payload);
        
        assert_eq!(message.version, crate::PROTOCOL_VERSION);
        assert_eq!(message.message_type, MessageType::Connect);
        assert!(message.validate().is_ok());
    }
    
    #[test]
    fn test_payload_validation() {
        // Valid payload
        let payload = MessagePayload::Connect {
            username: "validuser".to_string(),
        };
        assert!(payload.validate().is_ok());
        
        // Invalid payload - empty username
        let payload = MessagePayload::Connect {
            username: "".to_string(),
        };
        assert!(payload.validate().is_err());
        
        // Invalid payload - username too long
        let payload = MessagePayload::Connect {
            username: "a".repeat(50),
        };
        assert!(payload.validate().is_err());
    }
}
```

```rust
// protocol/src/error.rs
use thiserror::Error;

/// Errors that can occur during protocol operations
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Message too large: {size} bytes (max: {max_size})")]
    MessageTooLarge { size: usize, max_size: usize },
    
    #[error("Invalid protocol version: {version} (expected: {expected})")]
    InvalidVersion { version: u8, expected: u8 },
    
    #[error("Unknown message type: {message_type:#x}")]
    UnknownMessageType { message_type: u8 },
    
    #[error("Invalid username")]
    InvalidUsername,
    
    #[error("Invalid room name")]
    InvalidRoomName,
    
    #[error("Invalid message content")]
    InvalidMessageContent,
    
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Specific parsing errors
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid message length: {length}")]
    InvalidLength { length: u16 },
    
    #[error("Checksum mismatch: expected {expected:#x}, got {actual:#x}")]
    ChecksumMismatch { expected: u32, actual: u32 },
    
    #[error("Invalid UTF-8 string")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    
    #[error("Buffer too small: need {needed} bytes, have {available}")]
    BufferTooSmall { needed: usize, available: usize },
}

/// Type alias for results
pub type Result<T> = std::result::Result<T, ProtocolError>;
pub type ParseResult<T> = std::result::Result<T, ParseError>;
```

## Implementation Guidance

### 1. Protocol Parser Implementation

```rust
// protocol/src/parser.rs
use bytes::{Buf, BytesMut};
use crate::{Message, MessageType, MessagePayload, ParseError, ParseResult};

pub struct ProtocolParser {
    buffer: BytesMut,
}

impl ProtocolParser {
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::with_capacity(crate::DEFAULT_BUFFER_SIZE),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }
    
    /// Add data to the parser buffer
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }
    
    /// Try to parse a complete message from the buffer
    pub fn try_parse(&mut self) -> ParseResult<Option<Message>> {
        // TODO: Implement frame parsing
        // 1. Check if we have enough bytes for the header (4 bytes)
        // 2. Parse version, type, and length
        // 3. Check if we have the complete message
        // 4. Parse payload and validate checksum
        // 5. Return parsed message and advance buffer
        unimplemented!("Implement message parsing")
    }
    
    /// Parse message header to determine expected length
    fn parse_header(&self) -> ParseResult<(u8, MessageType, u16)> {
        if self.buffer.len() < 4 {
            return Err(ParseError::UnexpectedEof);
        }
        
        let version = self.buffer[0];
        let message_type = MessageType::from_u8(self.buffer[1])
            .ok_or(ParseError::InvalidMessageType { message_type: self.buffer[1] })?;
        let length = u16::from_be_bytes([self.buffer[2], self.buffer[3]]);
        
        Ok((version, message_type, length))
    }
    
    /// Calculate CRC32 checksum
    fn calculate_checksum(&self, data: &[u8]) -> u32 {
        crc32fast::hash(data)
    }
}

impl Default for ProtocolParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parser = ProtocolParser::new();
        assert!(parser.buffer.is_empty());
    }
    
    #[test]
    fn test_feed_data() {
        let mut parser = ProtocolParser::new();
        parser.feed(b"test data");
        assert_eq!(parser.buffer.len(), 9);
    }
    
    // TODO: Add comprehensive parsing tests
}
```

### 2. Async Server Implementation

```rust
// server/src/server.rs
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{info, warn, error};

pub struct ChatServer {
    listener: TcpListener,
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    clients: Arc<RwLock<HashMap<u32, ClientInfo>>>,
    next_client_id: Arc<std::sync::atomic::AtomicU32>,
    shutdown_tx: broadcast::Sender<()>,
}

#[derive(Debug, Clone)]
struct ClientInfo {
    id: u32,
    username: String,
    addr: SocketAddr,
    rooms: Vec<String>,
}

impl ChatServer {
    pub async fn bind(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let (shutdown_tx, _) = broadcast::channel(1);
        
        info!("Chat server starting on {}", addr);
        
        Ok(Self {
            listener,
            rooms: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: Arc::new(std::sync::atomic::AtomicU32::new(1)),
            shutdown_tx,
        })
    }
    
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        
        loop {
            tokio::select! {
                result = self.listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            let client_id = self.next_client_id
                                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            
                            info!("New connection from {}: client_id={}", addr, client_id);
                            
                            self.handle_client(client_id, stream, addr).await;
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Server shutdown requested");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_client(&self, client_id: u32, stream: TcpStream, addr: SocketAddr) {
        // TODO: Implement client connection handling
        // 1. Create ClientConnection struct
        // 2. Spawn async task for this client
        // 3. Handle incoming messages
        // 4. Broadcast messages to appropriate rooms
        // 5. Clean up on disconnect
        unimplemented!("Implement client handling")
    }
    
    pub fn shutdown(&self) -> Result<(), broadcast::error::SendError<()>> {
        self.shutdown_tx.send(())
    }
}

// TODO: Implement Room and ClientConnection structs
```

### 3. Testing Strategy

```rust
// protocol/tests/integration_tests.rs
use chat_protocol::{Message, MessageType, MessagePayload, ProtocolParser};

#[test]
fn test_round_trip_parsing() {
    // TODO: Test message serialization -> parsing -> deserialization
}

#[test]
fn test_partial_message_handling() {
    // TODO: Test parsing with incomplete data
}

#[test]
fn test_malformed_message_handling() {
    // TODO: Test parsing with corrupted data
}

#[tokio::test]
async fn test_server_client_communication() {
    // TODO: Integration test with actual server and client
}
```

### 4. Performance Benchmarks

```rust
// protocol/benches/parser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chat_protocol::{Message, MessageType, MessagePayload, ProtocolParser};

fn benchmark_message_parsing(c: &mut Criterion) {
    let message = Message::new(
        MessageType::SendMessage,
        MessagePayload::SendMessage {
            room: "general".to_string(),
            content: "Hello, world!".to_string(),
        }
    );
    
    // TODO: Implement serialization and benchmark parsing
    
    c.bench_function("parse_message", |b| {
        b.iter(|| {
            // TODO: Benchmark message parsing
            black_box(())
        })
    });
}

criterion_group!(benches, benchmark_message_parsing);
criterion_main!(benches);
```

## Performance Considerations

### 1. Zero-Copy Parsing
```rust
// Use Bytes for zero-copy parsing where possible
use bytes::{Bytes, Buf};

pub struct ZeroCopyMessage<'a> {
    pub version: u8,
    pub message_type: MessageType,
    pub payload: &'a [u8],  // Reference to original buffer
}

impl<'a> ZeroCopyMessage<'a> {
    pub fn parse_from_bytes(data: &'a [u8]) -> ParseResult<Self> {
        // TODO: Parse without allocating new strings/vectors
        unimplemented!()
    }
}
```

### 2. Connection Pooling
```rust
// Reuse connections and buffers
pub struct ConnectionPool {
    available: Vec<ClientConnection>,
    max_size: usize,
}

impl ConnectionPool {
    pub fn get_connection(&mut self) -> ClientConnection {
        self.available.pop().unwrap_or_else(|| {
            ClientConnection::new()
        })
    }
    
    pub fn return_connection(&mut self, mut conn: ClientConnection) {
        if self.available.len() < self.max_size {
            conn.reset(); // Clear state but keep allocated buffers
            self.available.push(conn);
        }
        // Otherwise drop the connection
    }
}
```

### 3. Metrics Collection
```rust
// server/src/metrics.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct ServerMetrics {
    pub messages_processed: AtomicU64,
    pub clients_connected: AtomicU64,
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub parse_errors: AtomicU64,
}

impl ServerMetrics {
    pub fn record_message_processed(&self) {
        self.messages_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_client_connected(&self) {
        self.clients_connected.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn record_parse_error(&self) {
        self.parse_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            messages_processed: self.messages_processed.load(Ordering::Relaxed),
            clients_connected: self.clients_connected.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            parse_errors: self.parse_errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub messages_processed: u64,
    pub clients_connected: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub parse_errors: u64,
}
```

## Common Pitfalls and Solutions

### 1. Buffer Management
```rust
// BAD: Allocating new buffers for each message
fn parse_message_inefficient(data: &[u8]) -> Result<Message, ParseError> {
    let mut owned_data = data.to_vec(); // Unnecessary allocation
    // ... parsing logic
}

// GOOD: Reuse buffers and parse in-place
pub struct ReusableParser {
    buffer: BytesMut,
    temp_string_buffer: String,
}

impl ReusableParser {
    fn parse_message(&mut self, data: &[u8]) -> Result<Message, ParseError> {
        // Reuse internal buffers
        self.buffer.clear();
        self.buffer.extend_from_slice(data);
        // ... parsing logic that reuses buffers
    }
}
```

### 2. Error Handling in Async Context
```rust
// Handle client disconnections gracefully
async fn handle_client_messages(mut stream: TcpStream) {
    let mut parser = ProtocolParser::new();
    let mut buffer = [0u8; 1024];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                // Client disconnected
                info!("Client disconnected gracefully");
                break;
            }
            Ok(n) => {
                parser.feed(&buffer[..n]);
                
                while let Ok(Some(message)) = parser.try_parse() {
                    if let Err(e) = handle_message(message).await {
                        error!("Failed to handle message: {}", e);
                        // Decide whether to disconnect or continue
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }
}
```

## Exercises

### Exercise 1: Complete the Protocol Parser

Implement the missing parts of `ProtocolParser::try_parse()`:

```rust
pub fn try_parse(&mut self) -> ParseResult<Option<Message>> {
    // TODO: Your implementation here
    // Requirements:
    // 1. Parse header (version, type, length)
    // 2. Validate we have complete message
    // 3. Parse payload based on message type
    // 4. Verify checksum
    // 5. Advance buffer and return message
    unimplemented!()
}
```

### Exercise 2: Implement Client Connection Handling

Complete the server's client handling logic:

```rust
struct ClientConnection {
    id: u32,
    stream: TcpStream,
    parser: ProtocolParser,
    username: Option<String>,
    rooms: Vec<String>,
}

impl ClientConnection {
    async fn handle_message(&mut self, message: Message) -> Result<(), ProtocolError> {
        // TODO: Implement message handling logic
        // 1. Validate message based on current state
        // 2. Update client state (username, rooms)
        // 3. Broadcast to appropriate clients
        // 4. Send response back to client
        unimplemented!()
    }
}
```

### Exercise 3: Add Load Testing

Create a stress test client that:

```rust
// examples/stress_test.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Create stress test that:
    // 1. Connects multiple clients simultaneously
    // 2. Sends messages at high rate
    // 3. Measures latency and throughput
    // 4. Reports performance metrics
    // 5. Handles connection failures gracefully
    Ok(())
}
```

## Key Takeaways

1. **Protocol Design Matters**: Well-designed protocols with proper framing prevent parsing ambiguities
2. **Async is Essential**: Modern network services require async/await for scalability
3. **Error Handling is Critical**: Network services must gracefully handle all types of failures
4. **Performance Testing is Mandatory**: Always benchmark and profile network code
5. **Security First**: Validate all inputs and limit resource usage to prevent attacks
6. **Documentation Enables Adoption**: Clear examples and API docs are essential
7. **Monitoring Provides Visibility**: Metrics and logging help debug production issues
8. **Testing Builds Confidence**: Comprehensive tests prevent regressions and ensure reliability

This capstone project demonstrates how to build production-ready network services in Rust, combining performance, safety, and maintainability. The skills learned here apply directly to building web servers, microservices, and distributed systems.

**Congratulations!** You've completed a comprehensive journey through systems programming with Rust, from basic ownership concepts to building production network services.
