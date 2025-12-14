use chrono::{DateTime, Utc};
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
    /// Known contacts to make it easier to initiate chats.
    pub contacts: Vec<Contact>,
    /// Outgoing messages recorded for auditability.
    pub outgoing_messages: Vec<OutgoingMessage>,
    /// Whether the simulated connection is active.
    pub connected: bool,
    /// Timestamp of the last successful connection.
    pub last_connected: Option<DateTime<Utc>>,
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

    /// Record a successful connection handshake.
    pub fn mark_connected(&mut self) {
        self.connected = true;
        self.last_connected = Some(Utc::now());
    }

    /// Mark the session as disconnected.
    pub fn mark_disconnected(&mut self) {
        self.connected = false;
    }

    /// Whether the client is marked as connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Add or update a known contact.
    pub fn upsert_contact(
        &mut self,
        jid: impl Into<String>,
        display_name: impl Into<String>,
    ) -> &Contact {
        let jid = jid.into();
        let display_name = display_name.into();

        if let Some(pos) = self.contacts.iter().position(|c| c.jid == jid) {
            self.contacts[pos].display_name = display_name.clone();
            return &self.contacts[pos];
        }

        self.contacts.push(Contact { jid, display_name });
        let idx = self.contacts.len() - 1;
        &self.contacts[idx]
    }

    /// Store an outgoing message for traceability and return it.
    pub fn record_message(
        &mut self,
        to: impl Into<String>,
        body: impl Into<String>,
    ) -> OutgoingMessage {
        let message = OutgoingMessage {
            to: to.into(),
            body: body.into(),
            sent_at: Utc::now(),
        };
        self.outgoing_messages.push(message.clone());
        message
    }
}

/// Basic representation of a WhatsApp contact.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contact {
    pub jid: String,
    pub display_name: String,
}

/// Outgoing message record that includes a timestamp for auditing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutgoingMessage {
    pub to: String,
    pub body: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub sent_at: DateTime<Utc>,
}
