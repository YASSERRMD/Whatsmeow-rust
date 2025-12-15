//! Cryptographic primitives for WhatsApp protocol.
//!
//! This module provides all cryptographic operations needed for:
//! - Noise Protocol (handshake with WhatsApp servers)
//! - Signal Protocol (end-to-end encryption)

mod keypair;
mod hkdf;
mod cipher;
mod noise;

pub use keypair::{KeyPair, PreKey};
pub use hkdf::{Hkdf, derive_noise_keys};
pub use cipher::{Cipher, CipherError};
pub use noise::{NoiseHandshake, HandshakeError, NOISE_PROTOCOL_NAME};
