//! Lightweight, documented scaffolding for a Rust port of the Whatsmeow client.
//!
//! The upstream Whatsmeow project is a full-featured Go library for interacting
//! with WhatsApp. This crate does not attempt to mirror every feature. Instead,
//! it exposes a small set of building blocks—configuration, client state, and a
//! simple client façade—that can be extended into a larger implementation.

pub mod client;
pub mod config;
pub mod state;

pub use client::{ClientError, WhatsmeowClient};
pub use config::WhatsmeowConfig;
pub use state::{MessageStatus, SessionState};
