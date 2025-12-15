//! Event types for WhatsApp events.
//!
//! These events are emitted when various things happen on the WhatsApp connection.

use crate::types::JID;

/// Connected event is emitted when the client connects to WhatsApp servers.
#[derive(Debug, Clone)]
pub struct Connected {
    /// Whether this is an initial connection or a reconnection
    pub is_reconnect: bool,
}

/// Disconnected event is emitted when the client disconnects.
#[derive(Debug, Clone)]
pub struct Disconnected {
    /// The reason for disconnection
    pub reason: DisconnectReason,
}

/// Reason for disconnection
#[derive(Debug, Clone, PartialEq)]
pub enum DisconnectReason {
    /// Normal logout by user
    LoggedOut,
    /// Connection replaced by another device
    Replaced,
    /// Server requested disconnect
    ServerRequested,
    /// Network error
    NetworkError(String),
    /// Unknown reason
    Unknown,
}

/// LoggedOut event is emitted when the user is logged out.
#[derive(Debug, Clone)]
pub struct LoggedOut {
    /// Whether the logout was initiated by the user
    pub by_user: bool,
    /// Reason for logout if available
    pub reason: Option<String>,
}

/// QR code event for pairing
#[derive(Debug, Clone)]
pub struct QRCode {
    /// The QR code data to display
    pub code: String,
    /// How many seconds the code is valid
    pub timeout_seconds: u64,
}

/// Pairing code event (alternative to QR)
#[derive(Debug, Clone)]
pub struct PairingCode {
    /// The pairing code to enter on phone
    pub code: String,
}

/// Message event containing a received message
#[derive(Debug, Clone)]
pub struct Message {
    /// The message info
    pub info: MessageInfo,
    /// The message content
    pub content: MessageContent,
}

/// Information about a message
#[derive(Debug, Clone)]
pub struct MessageInfo {
    /// Unique message ID
    pub id: String,
    /// Sender JID
    pub sender: JID,
    /// Chat JID (same as sender for 1:1, group JID for groups)
    pub chat: JID,
    /// Whether this message was sent by us
    pub is_from_me: bool,
    /// Whether this is a group message
    pub is_group: bool,
    /// Timestamp of the message
    pub timestamp: i64,
    /// Push name of sender
    pub push_name: Option<String>,
}

/// Content of a message
#[derive(Debug, Clone)]
pub enum MessageContent {
    /// Text message
    Text(String),
    /// Image message
    Image {
        url: String,
        caption: Option<String>,
        mimetype: String,
    },
    /// Video message
    Video {
        url: String,
        caption: Option<String>,
        mimetype: String,
    },
    /// Audio message
    Audio {
        url: String,
        mimetype: String,
        ptt: bool, // Voice note
    },
    /// Document message
    Document {
        url: String,
        filename: String,
        mimetype: String,
    },
    /// Sticker message
    Sticker {
        url: String,
    },
    /// Location message
    Location {
        latitude: f64,
        longitude: f64,
        name: Option<String>,
    },
    /// Contact message
    Contact {
        display_name: String,
        vcard: String,
    },
    /// Reaction to a message
    Reaction {
        target_id: String,
        emoji: String,
    },
    /// Unknown/unsupported message type
    Unknown,
}

/// Receipt event for message delivery/read status
#[derive(Debug, Clone)]
pub struct Receipt {
    /// Message IDs this receipt is for
    pub message_ids: Vec<String>,
    /// The chat JID
    pub chat: JID,
    /// The sender of the receipt
    pub sender: JID,
    /// Type of receipt
    pub receipt_type: ReceiptType,
    /// Timestamp of the receipt
    pub timestamp: i64,
}

/// Type of receipt
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiptType {
    /// Message was delivered
    Delivered,
    /// Message was read
    Read,
    /// Media was played (for audio/video)
    Played,
    /// Server received the message
    Server,
}

/// Presence event
#[derive(Debug, Clone)]
pub struct Presence {
    /// JID of the user
    pub from: JID,
    /// Whether the user is available
    pub available: bool,
    /// Last seen timestamp if available
    pub last_seen: Option<i64>,
}

/// Typing indicator event
#[derive(Debug, Clone)]
pub struct ChatState {
    /// JID of the chat
    pub chat: JID,
    /// JID of the user (for groups)
    pub sender: JID,
    /// The chat state
    pub state: ChatStateType,
}

/// Chat state type
#[derive(Debug, Clone, PartialEq)]
pub enum ChatStateType {
    /// User is composing a message
    Composing,
    /// User stopped typing
    Paused,
    /// User is recording audio
    Recording,
}

/// History sync notification
#[derive(Debug, Clone)]
pub struct HistorySync {
    /// Type of history sync
    pub sync_type: HistorySyncType,
    /// Data (for download)
    pub data: Vec<u8>,
}

/// History sync type
#[derive(Debug, Clone, PartialEq)]
pub enum HistorySyncType {
    Initial,
    Recent,
    Push,
    Full,
}

/// All possible events that can be received
#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connected),
    Disconnected(Disconnected),
    LoggedOut(LoggedOut),
    QRCode(QRCode),
    PairingCode(PairingCode),
    Message(Message),
    Receipt(Receipt),
    Presence(Presence),
    ChatState(ChatState),
    HistorySync(HistorySync),
}
