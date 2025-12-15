//! Whatsmeow-rust: WhatsApp Web Protocol Library
//!
//! A Rust implementation of the WhatsApp Web protocol, ported from the
//! [whatsmeow](https://github.com/tulir/whatsmeow) Go library.
//!
//! ## Modules
//!
//! - `types` - Core types like JID, MessageID, and events
//! - `binary` - Binary XML encoding/decoding
//! - `crypto` - Cryptographic primitives (Curve25519, AES-GCM, HKDF, Noise)
//! - `socket` - WebSocket transport with Noise Protocol
//! - `store` - Device storage and session management
//! - `protocol` - High-level client implementation

pub mod types;
pub mod binary;
pub mod crypto;
pub mod socket;
pub mod store;
pub mod protocol;

// Re-export existing scaffold modules (for backwards compat)
mod client;
mod config;
mod state;

pub use client::{WhatsmeowClient, ClientError as ScaffoldClientError};
pub use config::WhatsmeowConfig;
pub use state::{
    Contact, IncomingMessage, MediaItem, MessageStatus, NetworkState, OutgoingMessage, PairingCode,
    QrLogin, SessionEvent, SessionState,
};

// Re-export new protocol types
pub use types::{JID, MessageID};
pub use binary::{Node, encode, decode};
pub use store::{Device, MemoryStore};
pub use protocol::{Client, ClientConfig, ClientError};

