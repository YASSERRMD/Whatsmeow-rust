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
    /// Incoming messages recorded for testing inbound flows.
    pub incoming_messages: Vec<IncomingMessage>,
    /// Whether the simulated connection is active.
    pub connected: bool,
    /// Timestamp of the last successful connection.
    pub last_connected: Option<DateTime<Utc>>,
    /// Optional pairing code for QR/pairing flows.
    pub pairing_code: Option<PairingCode>,
    /// Historical timeline of notable session events.
    pub events: Vec<SessionEvent>,
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
        self.push_event(EventKind::Registered);
    }

    /// Record a successful connection handshake.
    pub fn mark_connected(&mut self) {
        self.connected = true;
        self.last_connected = Some(Utc::now());
        self.push_event(EventKind::Connected);
    }

    /// Mark the session as disconnected.
    pub fn mark_disconnected(&mut self) {
        self.connected = false;
        self.push_event(EventKind::Disconnected);
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
        self.events
            .push(SessionEvent::new(EventKind::MessageSent(message.clone())));
        message
    }

    /// Store an incoming message to mirror the outbound log.
    pub fn record_incoming_message(
        &mut self,
        from: impl Into<String>,
        body: impl Into<String>,
    ) -> IncomingMessage {
        let message = IncomingMessage {
            from: from.into(),
            body: body.into(),
            received_at: Utc::now(),
        };
        self.incoming_messages.push(message.clone());
        self.events
            .push(SessionEvent::new(EventKind::MessageReceived(
                message.clone(),
            )));
        message
    }

    /// Save a new pairing code with expiry time and emit an event.
    pub fn set_pairing_code(&mut self, code: impl Into<String>, expires_at: DateTime<Utc>) {
        self.pairing_code = Some(PairingCode {
            code: code.into(),
            issued_at: Utc::now(),
            expires_at,
        });
        self.events
            .push(SessionEvent::new(EventKind::PairingCodeIssued));
    }

    /// Add an event to the session timeline.
    pub fn push_event(&mut self, kind: EventKind) {
        self.events.push(SessionEvent::new(kind));
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

/// Incoming message record to mirror real-world delivery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncomingMessage {
    pub from: String,
    pub body: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub received_at: DateTime<Utc>,
}

/// Representation of a pairing code used for device linking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingCode {
    pub code: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub issued_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

/// Event history for the session lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionEvent {
    pub kind: EventKind,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub at: DateTime<Utc>,
}

impl SessionEvent {
    pub fn new(kind: EventKind) -> Self {
        Self {
            kind,
            at: Utc::now(),
        }
    }
}

/// Different types of notable events that can be recorded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventKind {
    Registered,
    Connected,
    Disconnected,
    PairingCodeIssued,
    MessageSent(OutgoingMessage),
    MessageReceived(IncomingMessage),
}
