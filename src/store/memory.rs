//! In-memory store implementation for development and testing.
//!
//! For production use, consider using SQLite or another persistent store.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::types::JID;
use crate::store::{
    Device, ContactInfo, ChatSettings, PreKeyRecord,
    IdentityStore, SessionStore, PreKeyStore, SenderKeyStore, 
    ContactStore, ChatSettingsStore, DeviceStore,
    StoreError, StoreResult,
};

/// In-memory implementation of all store traits.
pub struct MemoryStore {
    devices: RwLock<HashMap<String, Device>>,
    identities: RwLock<HashMap<String, [u8; 32]>>,
    sessions: RwLock<HashMap<String, Vec<u8>>>,
    pre_keys: RwLock<HashMap<u32, PreKeyRecord>>,
    sender_keys: RwLock<HashMap<String, Vec<u8>>>,
    contacts: RwLock<HashMap<String, ContactInfo>>,
    chat_settings: RwLock<HashMap<String, ChatSettings>>,
}

impl MemoryStore {
    /// Create a new in-memory store.
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
            identities: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            pre_keys: RwLock::new(HashMap::new()),
            sender_keys: RwLock::new(HashMap::new()),
            contacts: RwLock::new(HashMap::new()),
            chat_settings: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceStore for MemoryStore {
    fn get_device(&self, jid: &JID) -> StoreResult<Option<Device>> {
        let devices = self.devices.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(devices.get(&jid.to_string()).cloned())
    }

    fn put_device(&self, device: &Device) -> StoreResult<()> {
        if let Some(ref jid) = device.jid {
            let mut devices = self.devices.write()
                .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
            devices.insert(jid.to_string(), device.clone());
        }
        Ok(())
    }

    fn delete_device(&self, jid: &JID) -> StoreResult<()> {
        let mut devices = self.devices.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        devices.remove(&jid.to_string());
        Ok(())
    }

    fn get_first_device(&self) -> StoreResult<Option<Device>> {
        let devices = self.devices.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(devices.values().next().cloned())
    }
}

impl IdentityStore for MemoryStore {
    fn put_identity(&self, address: &str, key: [u8; 32]) -> StoreResult<()> {
        let mut identities = self.identities.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        identities.insert(address.to_string(), key);
        Ok(())
    }

    fn get_identity(&self, address: &str) -> StoreResult<Option<[u8; 32]>> {
        let identities = self.identities.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(identities.get(address).copied())
    }

    fn is_trusted_identity(&self, address: &str, key: &[u8; 32]) -> StoreResult<bool> {
        let identities = self.identities.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        match identities.get(address) {
            Some(stored) => Ok(stored == key),
            None => Ok(true), // Trust on first use
        }
    }

    fn delete_identity(&self, address: &str) -> StoreResult<()> {
        let mut identities = self.identities.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        identities.remove(address);
        Ok(())
    }
}

impl SessionStore for MemoryStore {
    fn get_session(&self, address: &str) -> StoreResult<Option<Vec<u8>>> {
        let sessions = self.sessions.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(sessions.get(address).cloned())
    }

    fn has_session(&self, address: &str) -> StoreResult<bool> {
        let sessions = self.sessions.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(sessions.contains_key(address))
    }

    fn put_session(&self, address: &str, session: &[u8]) -> StoreResult<()> {
        let mut sessions = self.sessions.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        sessions.insert(address.to_string(), session.to_vec());
        Ok(())
    }

    fn delete_session(&self, address: &str) -> StoreResult<()> {
        let mut sessions = self.sessions.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        sessions.remove(address);
        Ok(())
    }
}

impl PreKeyStore for MemoryStore {
    fn get_pre_key(&self, id: u32) -> StoreResult<Option<PreKeyRecord>> {
        let pre_keys = self.pre_keys.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(pre_keys.get(&id).cloned())
    }

    fn put_pre_key(&self, record: &PreKeyRecord) -> StoreResult<()> {
        let mut pre_keys = self.pre_keys.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        pre_keys.insert(record.key_id, record.clone());
        Ok(())
    }

    fn remove_pre_key(&self, id: u32) -> StoreResult<()> {
        let mut pre_keys = self.pre_keys.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        pre_keys.remove(&id);
        Ok(())
    }

    fn uploaded_pre_key_count(&self) -> StoreResult<usize> {
        let pre_keys = self.pre_keys.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(pre_keys.values().filter(|pk| pk.uploaded).count())
    }

    fn mark_pre_keys_uploaded(&self, up_to_id: u32) -> StoreResult<()> {
        let mut pre_keys = self.pre_keys.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        for pk in pre_keys.values_mut() {
            if pk.key_id <= up_to_id {
                pk.uploaded = true;
            }
        }
        Ok(())
    }
}

impl SenderKeyStore for MemoryStore {
    fn get_sender_key(&self, group: &str, user: &str) -> StoreResult<Option<Vec<u8>>> {
        let key = format!("{}:{}", group, user);
        let sender_keys = self.sender_keys.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(sender_keys.get(&key).cloned())
    }

    fn put_sender_key(&self, group: &str, user: &str, session: &[u8]) -> StoreResult<()> {
        let key = format!("{}:{}", group, user);
        let mut sender_keys = self.sender_keys.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        sender_keys.insert(key, session.to_vec());
        Ok(())
    }
}

impl ContactStore for MemoryStore {
    fn get_contact(&self, jid: &JID) -> StoreResult<Option<ContactInfo>> {
        let contacts = self.contacts.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(contacts.get(&jid.to_string()).cloned())
    }

    fn put_contact(&self, contact: &ContactInfo) -> StoreResult<()> {
        let mut contacts = self.contacts.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        contacts.insert(contact.jid.to_string(), contact.clone());
        Ok(())
    }

    fn get_all_contacts(&self) -> StoreResult<Vec<ContactInfo>> {
        let contacts = self.contacts.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(contacts.values().cloned().collect())
    }
}

impl ChatSettingsStore for MemoryStore {
    fn get_chat_settings(&self, chat: &JID) -> StoreResult<Option<ChatSettings>> {
        let settings = self.chat_settings.read()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        Ok(settings.get(&chat.to_string()).cloned())
    }

    fn put_chat_settings(&self, chat: &JID, settings: &ChatSettings) -> StoreResult<()> {
        let mut chat_settings = self.chat_settings.write()
            .map_err(|_| StoreError::DatabaseError("lock poisoned".to_string()))?;
        chat_settings.insert(chat.to_string(), settings.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_identity() {
        let store = MemoryStore::new();
        let key = [0xab; 32];
        
        store.put_identity("test@domain", key).unwrap();
        
        let retrieved = store.get_identity("test@domain").unwrap();
        assert_eq!(retrieved, Some(key));
    }

    #[test]
    fn test_memory_store_session() {
        let store = MemoryStore::new();
        let session = vec![1, 2, 3, 4];
        
        store.put_session("user@domain", &session).unwrap();
        
        assert!(store.has_session("user@domain").unwrap());
        assert_eq!(store.get_session("user@domain").unwrap(), Some(session));
    }

    #[test]
    fn test_memory_store_contact() {
        let store = MemoryStore::new();
        let contact = ContactInfo {
            jid: JID::new("123", "s.whatsapp.net"),
            first_name: "Test".to_string(),
            full_name: "Test User".to_string(),
            ..Default::default()
        };
        
        store.put_contact(&contact).unwrap();
        
        let retrieved = store.get_contact(&contact.jid).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().full_name, "Test User");
    }
}
