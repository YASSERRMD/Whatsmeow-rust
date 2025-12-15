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
//! - `client` - High-level client API
//! - `config` - Configuration management
//! - `state` - Session state management

pub mod types;
pub mod binary;
pub mod crypto;
pub mod socket;

// Re-export existing modules (from scaffold)
mod client;
mod config;
mod state;

pub use client::{WhatsmeowClient, ClientError};
pub use config::WhatsmeowConfig;
pub use state::{
    Contact, IncomingMessage, MediaItem, MessageStatus, NetworkState, OutgoingMessage, PairingCode,
    QrLogin, SessionEvent, SessionState,
};

// Re-export new protocol types
pub use types::{JID, MessageID};
pub use binary::{Node, encode, decode};
