use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    /// Networking metadata for simulated upstream connectivity.
    pub network: NetworkState,
    /// State of the most recent QR login flow.
    pub qr_login: Option<QrLogin>,
    /// Media downloads recorded for inspection.
    pub media: Vec<MediaItem>,
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

    /// Record a network handshake against the configured endpoint.
    pub fn mark_network_handshake(
        &mut self,
        endpoint: impl Into<String>,
        latency_ms: Option<u128>,
        status_code: Option<u16>,
        error: Option<String>,
    ) {
        self.network = NetworkState {
            endpoint: endpoint.into(),
            last_handshake: Some(Utc::now()),
            latency_ms,
            status_code,
            error,
        };
        self.push_event(EventKind::NetworkHandshaked(self.network.clone()));
    }

    /// Store a QR login token and expiry timestamp.
    pub fn set_qr_login(&mut self, token: impl Into<String>, expires_at: DateTime<Utc>) {
        let login = QrLogin {
            token: token.into(),
            issued_at: Utc::now(),
            expires_at,
            verified: false,
        };
        self.qr_login = Some(login.clone());
        self.push_event(EventKind::QrCodeGenerated(login));
    }

    /// Mark the QR login as verified once the token is used.
    pub fn verify_qr_login(&mut self) -> Option<QrLogin> {
        let cloned = {
            let login = self.qr_login.as_mut()?;
            login.verified = true;
            login.clone()
        };
        self.push_event(EventKind::QrCodeVerified);
        Some(cloned)
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
            id: Uuid::new_v4(),
            to: to.into(),
            body: body.into(),
            sent_at: Utc::now(),
            status: MessageStatus::Queued,
            ciphertext: None,
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
            id: Uuid::new_v4(),
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

    /// Record a media download and emit an event.
    pub fn record_media(
        &mut self,
        source: impl Into<String>,
        file_path: impl Into<String>,
        bytes: u64,
    ) -> MediaItem {
        let item = MediaItem {
            id: Uuid::new_v4(),
            source: source.into(),
            file_path: file_path.into(),
            bytes,
            downloaded_at: Utc::now(),
        };
        self.media.push(item.clone());
        self.events
            .push(SessionEvent::new(EventKind::MediaDownloaded(item.clone())));
        item
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

    /// Update the status of an outgoing message and return the updated record.
    pub fn mark_outgoing_status(
        &mut self,
        id: Uuid,
        status: MessageStatus,
    ) -> Option<&OutgoingMessage> {
        let maybe_message = self.outgoing_messages.iter_mut().find(|msg| msg.id == id)?;
        maybe_message.status = status.clone();
        self.events
            .push(SessionEvent::new(EventKind::MessageStatusChanged {
                id,
                status,
            }));
        Some(maybe_message)
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
    #[serde(with = "uuid::serde::compact")]
    pub id: Uuid,
    pub to: String,
    pub body: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub sent_at: DateTime<Utc>,
    pub status: MessageStatus,
    /// Base64-encoded encrypted payload for the message body.
    pub ciphertext: Option<String>,
}

/// Incoming message record to mirror real-world delivery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncomingMessage {
    #[serde(with = "uuid::serde::compact")]
    pub id: Uuid,
    pub from: String,
    pub body: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub received_at: DateTime<Utc>,
}

/// Simplified message states to mimic delivery receipts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageStatus {
    Queued,
    Delivered,
    Read,
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
    NetworkHandshaked(NetworkState),
    QrCodeGenerated(QrLogin),
    QrCodeVerified,
    MessageSent(OutgoingMessage),
    MessageReceived(IncomingMessage),
    MessageStatusChanged { id: Uuid, status: MessageStatus },
    MessageEncrypted(Uuid),
    MediaDownloaded(MediaItem),
}

/// Network connection metadata recorded per session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkState {
    pub endpoint: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub last_handshake: Option<DateTime<Utc>>,
    pub latency_ms: Option<u128>,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            endpoint: "https://chat.whatsmeow.test".into(),
            last_handshake: None,
            latency_ms: None,
            status_code: None,
            error: None,
        }
    }
}

/// QR login token tracked for local verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QrLogin {
    pub token: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub issued_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
}

/// Media record stored after a simulated download.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaItem {
    #[serde(with = "uuid::serde::compact")]
    pub id: Uuid,
    pub source: String,
    pub file_path: String,
    pub bytes: u64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub downloaded_at: DateTime<Utc>,
}
