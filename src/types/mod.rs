//! Types module for WhatsApp protocol types.
//!
//! This module contains all the core types used in the WhatsApp protocol,
//! including JIDs, message IDs, and event types.

mod jid;
mod events;

pub use jid::*;
pub use events::*;
