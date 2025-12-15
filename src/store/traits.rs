//! Store traits for WhatsApp protocol data persistence.
//!
//! These traits define the interface for storing various types of data
//! needed by the WhatsApp client.

use crate::types::JID;
use crate::store::{Device, ContactInfo, ChatSettings, PreKeyRecord, SessionRecord, IdentityRecord};
use std::future::Future;

/// Error type for store operations.
#[derive(Debug, Clone)]
pub enum StoreError {
    NotFound,
    DatabaseError(String),
    SerializationError(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound => write!(f, "not found"),
            StoreError::DatabaseError(e) => write!(f, "database error: {}", e),
            StoreError::SerializationError(e) => write!(f, "serialization error: {}", e),
        }
    }
}

impl std::error::Error for StoreError {}

pub type StoreResult<T> = Result<T, StoreError>;

/// Identity store for Signal Protocol identity keys.
pub trait IdentityStore: Send + Sync {
    /// Store an identity key for an address.
    fn put_identity(&self, address: &str, key: [u8; 32]) -> StoreResult<()>;
    
    /// Get an identity key for an address.
    fn get_identity(&self, address: &str) -> StoreResult<Option<[u8; 32]>>;
    
    /// Check if an identity is trusted.
    fn is_trusted_identity(&self, address: &str, key: &[u8; 32]) -> StoreResult<bool>;
    
    /// Delete an identity.
    fn delete_identity(&self, address: &str) -> StoreResult<()>;
}

/// Session store for Signal Protocol sessions.
pub trait SessionStore: Send + Sync {
    /// Get a session for an address.
    fn get_session(&self, address: &str) -> StoreResult<Option<Vec<u8>>>;
    
    /// Check if a session exists.
    fn has_session(&self, address: &str) -> StoreResult<bool>;
    
    /// Store a session.
    fn put_session(&self, address: &str, session: &[u8]) -> StoreResult<()>;
    
    /// Delete a session.
    fn delete_session(&self, address: &str) -> StoreResult<()>;
}

/// Pre-key store for Signal Protocol pre-keys.
pub trait PreKeyStore: Send + Sync {
    /// Get a pre-key by ID.
    fn get_pre_key(&self, id: u32) -> StoreResult<Option<PreKeyRecord>>;
    
    /// Store a pre-key.
    fn put_pre_key(&self, record: &PreKeyRecord) -> StoreResult<()>;
    
    /// Remove a pre-key.
    fn remove_pre_key(&self, id: u32) -> StoreResult<()>;
    
    /// Get count of uploaded pre-keys.
    fn uploaded_pre_key_count(&self) -> StoreResult<usize>;
    
    /// Mark pre-keys as uploaded up to a given ID.
    fn mark_pre_keys_uploaded(&self, up_to_id: u32) -> StoreResult<()>;
}

/// Sender key store for group messaging.
pub trait SenderKeyStore: Send + Sync {
    /// Get a sender key.
    fn get_sender_key(&self, group: &str, user: &str) -> StoreResult<Option<Vec<u8>>>;
    
    /// Store a sender key.
    fn put_sender_key(&self, group: &str, user: &str, key: &[u8]) -> StoreResult<()>;
}

/// Contact store for contact information.
pub trait ContactStore: Send + Sync {
    /// Get contact info for a JID.
    fn get_contact(&self, jid: &JID) -> StoreResult<Option<ContactInfo>>;
    
    /// Store contact info.
    fn put_contact(&self, contact: &ContactInfo) -> StoreResult<()>;
    
    /// Get all contacts.
    fn get_all_contacts(&self) -> StoreResult<Vec<ContactInfo>>;
}

/// Chat settings store.
pub trait ChatSettingsStore: Send + Sync {
    /// Get chat settings for a JID.
    fn get_chat_settings(&self, chat: &JID) -> StoreResult<Option<ChatSettings>>;
    
    /// Store chat settings.
    fn put_chat_settings(&self, chat: &JID, settings: &ChatSettings) -> StoreResult<()>;
}

/// Device container for storing device data.
pub trait DeviceStore: Send + Sync {
    /// Get a device by JID.
    fn get_device(&self, jid: &JID) -> StoreResult<Option<Device>>;
    
    /// Store a device.
    fn put_device(&self, device: &Device) -> StoreResult<()>;
    
    /// Delete a device.
    fn delete_device(&self, jid: &JID) -> StoreResult<()>;
    
    /// Get the first/default device.
    fn get_first_device(&self) -> StoreResult<Option<Device>>;
}

/// Combined store interface for all stores.
pub trait Store: DeviceStore + IdentityStore + SessionStore + PreKeyStore + SenderKeyStore + ContactStore + ChatSettingsStore {
}

// Blanket implementation for any type that implements all store traits
impl<T> Store for T 
where 
    T: DeviceStore + IdentityStore + SessionStore + PreKeyStore + SenderKeyStore + ContactStore + ChatSettingsStore 
{}
