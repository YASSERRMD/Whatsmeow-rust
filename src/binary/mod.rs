//! Binary XML encoding and decoding for WhatsApp protocol.
//!
//! WhatsApp uses a custom binary XML format for efficient message encoding.
//! This module provides encoding and decoding of Node structures.

mod node;
mod token;
mod encoder;
mod decoder;

pub use node::*;
pub use token::{get_token, get_token_index, SINGLE_BYTE_TOKENS};
pub use encoder::{encode, Encoder};
pub use decoder::{decode, Decoder, DecodeError};
