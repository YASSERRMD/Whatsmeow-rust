use serde::{Deserialize, Serialize};

/// Minimal session state used to simulate device registration and key storage.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SessionState {
    /// JID associated with the registered account, if any.
    pub registered_jid: Option<String>,
    /// Placeholder encryption key identifiers.
    pub encryption_keys: Vec<String>,
    /// Human-readable device name.
    pub device_name: String,
}

impl SessionState {
    /// Create a new session with a user-specified device name.
    pub fn with_device_name(device_name: impl Into<String>) -> Self {
        Self {
            device_name: device_name.into(),
            ..Default::default()
        }
    }

    /// Whether a device has been registered.
    pub fn is_registered(&self) -> bool {
        self.registered_jid.is_some()
    }

    /// Record a registration and seed a dummy encryption key.
    pub fn register(&mut self, jid: impl Into<String>) {
        let jid = jid.into();
        self.registered_jid = Some(jid.clone());
        if self.encryption_keys.is_empty() {
            self.encryption_keys.push(format!("derived-key-for-{jid}"));
        }
    }
}
