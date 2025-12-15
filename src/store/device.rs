//! Device storage for WhatsApp protocol.
//!
//! Stores device identity, keys, and session data required for WhatsApp connection.

use crate::types::JID;
use crate::crypto::{KeyPair, PreKey};
use std::collections::HashMap;

/// Device represents a WhatsApp device/session.
#[derive(Clone)]
pub struct Device {
    /// Noise Protocol static key pair
    pub noise_key: Option<KeyPair>,
    /// Signal identity key pair
    pub identity_key: Option<KeyPair>,
    /// Signed pre-key for Signal Protocol
    pub signed_pre_key: Option<PreKey>,
    /// Registration ID
    pub registration_id: u32,
    /// Advanced secret key
    pub adv_secret_key: Option<Vec<u8>>,
    /// Device JID
    pub jid: Option<JID>,
    /// Linked ID (LID)
    pub lid: Option<JID>,
    /// Platform identifier (e.g., "android", "web")
    pub platform: String,
    /// Business name (if business account)
    pub business_name: Option<String>,
    /// Push name
    pub push_name: Option<String>,
    /// Whether the device has been initialized
    pub initialized: bool,
}

impl Device {
    /// Create a new uninitialized device.
    pub fn new() -> Self {
        Self {
            noise_key: None,
            identity_key: None,
            signed_pre_key: None,
            registration_id: 0,
            adv_secret_key: None,
            jid: None,
            lid: None,
            platform: String::new(),
            business_name: None,
            push_name: None,
            initialized: false,
        }
    }

    /// Initialize device with fresh keys.
    pub fn initialize(&mut self) {
        self.noise_key = Some(KeyPair::generate());
        self.identity_key = Some(KeyPair::generate());
        
        // Generate signed pre-key signed by identity key
        if let Some(ref identity) = self.identity_key {
            self.signed_pre_key = Some(PreKey::new_signed(1, identity));
        }
        
        // Generate advertisement secret key (32 bytes)
        let adv_secret: [u8; 32] = rand::random();
        self.adv_secret_key = Some(adv_secret.to_vec());
        
        self.registration_id = rand::random::<u32>() & 0x3FFF; // 14 bits
        self.initialized = true;
    }

    /// Get the device JID.
    pub fn get_jid(&self) -> Option<&JID> {
        self.jid.as_ref()
    }

    /// Check if device is registered (has a JID).
    pub fn is_registered(&self) -> bool {
        self.jid.is_some()
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

/// Contact information.
#[derive(Debug, Clone, Default)]
pub struct ContactInfo {
    pub jid: JID,
    pub first_name: String,
    pub full_name: String,
    pub push_name: Option<String>,
    pub business_name: Option<String>,
}

/// Chat settings.
#[derive(Debug, Clone, Default)]
pub struct ChatSettings {
    pub muted_until: Option<i64>,
    pub pinned: bool,
    pub archived: bool,
}

/// Pre-key record for storage.
#[derive(Debug, Clone)]
pub struct PreKeyRecord {
    pub key_id: u32,
    pub public_key: [u8; 32],
    pub private_key: [u8; 32],
    pub signature: Option<[u8; 64]>,
    pub uploaded: bool,
}

impl From<&PreKey> for PreKeyRecord {
    fn from(pk: &PreKey) -> Self {
        Self {
            key_id: pk.key_id,
            public_key: pk.key_pair.public,
            private_key: pk.key_pair.private,
            signature: pk.signature,
            uploaded: false,
        }
    }
}

/// Session record for Signal Protocol sessions.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub address: String,
    pub data: Vec<u8>,
}

/// Identity key record.
#[derive(Debug, Clone)]
pub struct IdentityRecord {
    pub address: String,
    pub public_key: [u8; 32],
    pub trusted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_initialization() {
        let mut device = Device::new();
        assert!(!device.initialized);
        
        device.initialize();
        
        assert!(device.initialized);
        assert!(device.noise_key.is_some());
        assert!(device.identity_key.is_some());
        assert!(device.signed_pre_key.is_some());
        assert!(device.registration_id > 0);
    }

    #[test]
    fn test_device_not_registered() {
        let device = Device::new();
        assert!(!device.is_registered());
    }
}
