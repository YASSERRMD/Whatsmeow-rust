//! WhatsApp Client implementation.
//!
//! High-level client for connecting to and interacting with WhatsApp.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{JID, Event, Message, MessageInfo, MessageContent};
use crate::binary::{Node, encode, decode};
use crate::crypto::KeyPair;
use crate::socket::{NoiseSocket, SocketError, endpoints};
use crate::store::{Device, MemoryStore, Store, DeviceStore};

/// Client configuration.
#[derive(Clone)]
pub struct ClientConfig {
    /// WebSocket endpoint URL
    pub endpoint: String,
    /// User agent string
    pub user_agent: String,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            endpoint: endpoints::MAIN.to_string(),
            user_agent: "WhatsApp/2.24.0".to_string(),
            auto_reconnect: true,
        }
    }
}

/// Event handler type.
pub type EventHandler = Box<dyn Fn(Event) + Send + Sync>;

/// WhatsApp client for connecting and messaging.
pub struct Client {
    /// Client configuration
    config: ClientConfig,
    /// Device information and keys
    device: Arc<RwLock<Device>>,
    /// Data store
    store: Arc<dyn Store>,
    /// Socket connection (when connected)
    socket: Option<NoiseSocket>,
    /// Whether currently connected
    connected: bool,
    /// Event handlers
    event_handlers: Vec<EventHandler>,
}

/// Client errors.
#[derive(Debug, Clone)]
pub enum ClientError {
    NotConnected,
    NotLoggedIn,
    AlreadyConnected,
    ConnectionFailed(String),
    HandshakeFailed(String),
    SendFailed(String),
    ReceiveFailed(String),
    StoreError(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::NotConnected => write!(f, "not connected"),
            ClientError::NotLoggedIn => write!(f, "not logged in"),
            ClientError::AlreadyConnected => write!(f, "already connected"),
            ClientError::ConnectionFailed(e) => write!(f, "connection failed: {}", e),
            ClientError::HandshakeFailed(e) => write!(f, "handshake failed: {}", e),
            ClientError::SendFailed(e) => write!(f, "send failed: {}", e),
            ClientError::ReceiveFailed(e) => write!(f, "receive failed: {}", e),
            ClientError::StoreError(e) => write!(f, "store error: {}", e),
        }
    }
}

impl std::error::Error for ClientError {}

impl Client {
    /// Create a new client with default configuration.
    pub fn new() -> Self {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new client with custom configuration.
    pub fn with_config(config: ClientConfig) -> Self {
        let mut device = Device::new();
        device.initialize();

        Self {
            config,
            device: Arc::new(RwLock::new(device)),
            store: Arc::new(MemoryStore::new()),
            socket: None,
            connected: false,
            event_handlers: Vec::new(),
        }
    }

    /// Create a new client with a custom store.
    pub fn with_store<S: Store + 'static>(config: ClientConfig, store: S) -> Self {
        let mut device = Device::new();
        device.initialize();

        Self {
            config,
            device: Arc::new(RwLock::new(device)),
            store: Arc::new(store),
            socket: None,
            connected: false,
            event_handlers: Vec::new(),
        }
    }

    /// Add an event handler.
    pub fn add_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    /// Connect to WhatsApp servers.
    pub async fn connect(&mut self) -> Result<(), ClientError> {
        if self.connected {
            return Err(ClientError::AlreadyConnected);
        }

        // Connect WebSocket
        let mut socket = NoiseSocket::connect(&self.config.endpoint)
            .await
            .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

        // Perform Noise handshake
        let device = self.device.read().await;
        let noise_key = device.noise_key.clone()
            .ok_or(ClientError::HandshakeFailed("no noise key".to_string()))?;
        drop(device);

        let _remote_static = socket.handshake(noise_key)
            .await
            .map_err(|e| ClientError::HandshakeFailed(e.to_string()))?;

        self.socket = Some(socket);
        self.connected = true;

        // Emit connected event
        self.emit_event(Event::Connected(crate::types::Connected {
            is_reconnect: false,
        }));

        Ok(())
    }

    /// Disconnect from WhatsApp servers.
    pub async fn disconnect(&mut self) -> Result<(), ClientError> {
        if let Some(ref mut socket) = self.socket {
            socket.close().await
                .map_err(|e| ClientError::SendFailed(e.to_string()))?;
        }
        
        self.socket = None;
        self.connected = false;

        self.emit_event(Event::Disconnected(crate::types::Disconnected {
            reason: crate::types::DisconnectReason::LoggedOut,
        }));

        Ok(())
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Check if logged in (has JID).
    pub async fn is_logged_in(&self) -> bool {
        let device = self.device.read().await;
        device.is_registered()
    }

    /// Get the device JID.
    pub async fn get_jid(&self) -> Option<JID> {
        let device = self.device.read().await;
        device.jid.clone()
    }

    /// Send a text message.
    pub async fn send_message(&mut self, to: JID, text: &str) -> Result<String, ClientError> {
        if !self.connected {
            return Err(ClientError::NotConnected);
        }

        // Generate message ID
        let message_id = format!("{:X}", rand::random::<u64>());

        // Build message node
        let mut node = Node::new("message");
        node.set_attr("id", message_id.clone());
        node.set_attr("type", "text");
        node.set_attr("to", to.to_string());

        let mut body = Node::new("body");
        body.set_bytes(text.as_bytes().to_vec());
        node.add_child(body);

        // Encode and send
        let data = encode(&node);
        
        if let Some(ref mut socket) = self.socket {
            socket.send(&data)
                .await
                .map_err(|e| ClientError::SendFailed(e.to_string()))?;
        }

        Ok(message_id)
    }

    /// Receive and process incoming data.
    pub async fn receive(&mut self) -> Result<Option<Event>, ClientError> {
        if !self.connected {
            return Err(ClientError::NotConnected);
        }

        let socket = self.socket.as_mut().ok_or(ClientError::NotConnected)?;
        
        let data = socket.recv()
            .await
            .map_err(|e| ClientError::ReceiveFailed(e.to_string()))?;

        // Decode the node
        let node = decode(&data)
            .map_err(|e| ClientError::ReceiveFailed(e.to_string()))?;

        // Process node based on tag
        let event = self.process_node(&node)?;
        
        if let Some(ref evt) = event {
            self.emit_event(evt.clone());
        }

        Ok(event)
    }

    /// Process a received node.
    fn process_node(&self, node: &Node) -> Result<Option<Event>, ClientError> {
        match node.tag.as_str() {
            "message" => {
                // Parse message
                let id = node.get_attr_str("id").unwrap_or("").to_string();
                let from_str = node.get_attr_str("from").unwrap_or("");
                let from: JID = from_str.parse().unwrap_or_default();

                // Get body content
                let body = node.get_child_by_tag("body")
                    .and_then(|b| b.get_bytes())
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .unwrap_or_default();

                let msg = Message {
                    info: MessageInfo {
                        id,
                        sender: from.clone(),
                        chat: from,
                        is_from_me: false,
                        is_group: false,
                        timestamp: chrono::Utc::now().timestamp(),
                        push_name: None,
                    },
                    content: MessageContent::Text(body),
                };

                Ok(Some(Event::Message(msg)))
            }
            "receipt" => {
                // Parse receipt
                let receipt = crate::types::Receipt {
                    message_ids: vec![node.get_attr_str("id").unwrap_or("").to_string()],
                    chat: node.get_attr_str("from").unwrap_or("").parse().unwrap_or_default(),
                    sender: node.get_attr_str("participant").unwrap_or("").parse().unwrap_or_default(),
                    receipt_type: match node.get_attr_str("type") {
                        Some("read") => crate::types::ReceiptType::Read,
                        Some("played") => crate::types::ReceiptType::Played,
                        _ => crate::types::ReceiptType::Delivered,
                    },
                    timestamp: chrono::Utc::now().timestamp(),
                };

                Ok(Some(Event::Receipt(receipt)))
            }
            _ => Ok(None),
        }
    }

    /// Emit an event to all handlers.
    fn emit_event(&self, event: Event) {
        for handler in &self.event_handlers {
            handler(event.clone());
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_client_with_config() {
        let config = ClientConfig {
            endpoint: "wss://custom.endpoint".to_string(),
            ..Default::default()
        };
        let client = Client::with_config(config);
        assert!(!client.is_connected());
    }
}
