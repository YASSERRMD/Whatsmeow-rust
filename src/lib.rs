//! Whatsmeow-rust: WhatsApp Web Protocol Library
//!
//! A Rust implementation of the WhatsApp Web protocol, ported from the
//! [whatsmeow](https://github.com/tulir/whatsmeow) Go library.
//!
//! ## Modules
//!
//! - `types` - Core types like JID, MessageID, and events
//! - `binary` - Binary XML encoding/decoding
//! - `client` - High-level client API (coming in Phase 3)
//! - `config` - Configuration management
//! - `state` - Session state management

pub mod types;
pub mod binary;

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
