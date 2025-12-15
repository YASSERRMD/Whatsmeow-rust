//! Protocol module for high-level WhatsApp operations.
//!
//! Contains the main Client implementation and protocol logic.

mod client;

pub use client::{Client, ClientConfig, ClientError};
