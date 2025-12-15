//! Protocol module for high-level WhatsApp operations.
//!
//! Contains the main Client implementation, QR pairing, message handling,
//! and request/response tracking.

mod client;
mod qr;
mod message;
mod request;

pub use client::{Client, ClientConfig, ClientError};
pub use qr::{QRPairing, QREvent, QRError, QRChannel, start_qr_pairing};
pub use message::*;
pub use request::{RequestTracker, build_iq_get, build_iq_set, build_iq_result};
