//! WebSocket transport for WhatsApp protocol.
//!
//! Provides connection management to WhatsApp servers using WebSocket + Noise Protocol.

pub mod handshake;

use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
use futures::{SinkExt, StreamExt};

use crate::crypto::{Cipher, NoiseHandshake, KeyPair};

pub use handshake::{do_handshake, WhatsAppConnection, HandshakeError};

/// WhatsApp WebSocket endpoints.
pub mod endpoints {
    pub const MAIN: &str = "wss://web.whatsapp.com/ws/chat";
    pub const FALLBACK: &str = "wss://w1.web.whatsapp.com/ws/chat";
}

/// WebSocket connection to WhatsApp servers.
pub struct NoiseSocket {
    /// The underlying WebSocket stream
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// Send cipher (after handshake)
    send_cipher: Option<Cipher>,
    /// Receive cipher (after handshake)
    recv_cipher: Option<Cipher>,
    /// Whether handshake is complete
    handshake_complete: bool,
}

impl NoiseSocket {
    /// Connect to WhatsApp servers.
    pub async fn connect(url: &str) -> Result<Self, SocketError> {
        let (ws, _response) = connect_async(url)
            .await
            .map_err(|e| SocketError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            ws,
            send_cipher: None,
            recv_cipher: None,
            handshake_complete: false,
        })
    }

    /// Connect to the main WhatsApp endpoint.
    pub async fn connect_main() -> Result<Self, SocketError> {
        Self::connect(endpoints::MAIN).await
    }

    /// Perform Noise Protocol handshake.
    pub async fn handshake(&mut self, static_key: KeyPair) -> Result<[u8; 32], SocketError> {
        let mut noise = NoiseHandshake::new_initiator(static_key);

        // Send message 1 (-> e)
        let msg1 = noise.write_message_1();
        let frame1 = self.build_handshake_frame(&msg1);
        self.send_raw(&frame1).await?;

        // Receive message 2 (<- e, ee, s, es)
        let response = self.recv_raw().await?;
        let msg2 = self.parse_handshake_frame(&response)?;
        let _payload = noise.read_message_2(&msg2)
            .map_err(|e| SocketError::HandshakeFailed(e.to_string()))?;

        // Send message 3 (-> s, se, payload)
        let client_payload = self.build_client_payload();
        let msg3 = noise.write_message_3(&client_payload)
            .map_err(|e| SocketError::HandshakeFailed(e.to_string()))?;
        let frame3 = self.build_handshake_frame(&msg3);
        self.send_raw(&frame3).await?;

        // Get remote static key
        let remote_static = noise.remote_static_key()
            .ok_or(SocketError::HandshakeFailed("no remote static key".to_string()))?
            .clone();

        // Split into transport ciphers
        let (send_cipher, recv_cipher) = noise.split();
        self.send_cipher = Some(send_cipher);
        self.recv_cipher = Some(recv_cipher);
        self.handshake_complete = true;

        Ok(remote_static)
    }

    /// Build handshake frame header.
    fn build_handshake_frame(&self, data: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(data.len() + 6);
        // WhatsApp-specific header
        frame.extend_from_slice(b"WA");
        frame.push(6); // Major version
        frame.push(0); // Minor version
        // Length prefix
        let len = data.len() as u32;
        frame.extend_from_slice(&len.to_be_bytes()[1..]); // 3 bytes
        frame.extend_from_slice(data);
        frame
    }

    /// Parse handshake frame.
    fn parse_handshake_frame(&self, data: &[u8]) -> Result<Vec<u8>, SocketError> {
        if data.len() < 4 {
            return Err(SocketError::InvalidFrame);
        }
        // Skip header and extract payload
        Ok(data[4..].to_vec())
    }

    /// Build client payload for handshake.
    fn build_client_payload(&self) -> Vec<u8> {
        // Minimal client payload - real implementation needs protobuf
        vec![0u8; 16]
    }

    /// Send raw bytes (before encryption).
    async fn send_raw(&mut self, data: &[u8]) -> Result<(), SocketError> {
        self.ws
            .send(Message::Binary(data.to_vec().into()))
            .await
            .map_err(|e| SocketError::SendFailed(e.to_string()))
    }

    /// Receive raw bytes.
    async fn recv_raw(&mut self) -> Result<Vec<u8>, SocketError> {
        match self.ws.next().await {
            Some(Ok(Message::Binary(data))) => Ok(data.to_vec()),
            Some(Err(e)) => Err(SocketError::ReceiveFailed(e.to_string())),
            _ => Err(SocketError::ConnectionClosed),
        }
    }

    /// Send an encrypted frame.
    pub async fn send(&mut self, data: &[u8]) -> Result<(), SocketError> {
        if !self.handshake_complete {
            return Err(SocketError::NotConnected);
        }

        let cipher = self.send_cipher.as_mut().ok_or(SocketError::NotConnected)?;
        
        // Encrypt the data
        let encrypted = cipher.encrypt(data, &[])
            .map_err(|_| SocketError::EncryptionFailed)?;

        // Build frame with length prefix
        let mut frame = Vec::with_capacity(encrypted.len() + 3);
        let len = encrypted.len() as u32;
        frame.extend_from_slice(&len.to_be_bytes()[1..]); // 3 bytes
        frame.extend_from_slice(&encrypted);

        self.send_raw(&frame).await
    }

    /// Receive and decrypt a frame.
    pub async fn recv(&mut self) -> Result<Vec<u8>, SocketError> {
        if !self.handshake_complete {
            return Err(SocketError::NotConnected);
        }

        let data = self.recv_raw().await?;
        
        if data.len() < 3 {
            return Err(SocketError::InvalidFrame);
        }

        let cipher = self.recv_cipher.as_mut().ok_or(SocketError::NotConnected)?;
        
        // Skip length prefix and decrypt
        let encrypted = &data[3..];
        cipher.decrypt(encrypted, &[])
            .map_err(|_| SocketError::DecryptionFailed)
    }

    /// Check if the socket is connected and handshake is complete.
    pub fn is_connected(&self) -> bool {
        self.handshake_complete
    }

    /// Close the connection.
    pub async fn close(&mut self) -> Result<(), SocketError> {
        self.ws
            .close(None)
            .await
            .map_err(|e| SocketError::SendFailed(e.to_string()))
    }
}

/// Socket errors.
#[derive(Debug, Clone)]
pub enum SocketError {
    ConnectionFailed(String),
    HandshakeFailed(String),
    SendFailed(String),
    ReceiveFailed(String),
    EncryptionFailed,
    DecryptionFailed,
    InvalidFrame,
    NotConnected,
    ConnectionClosed,
}

impl std::fmt::Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketError::ConnectionFailed(e) => write!(f, "connection failed: {}", e),
            SocketError::HandshakeFailed(e) => write!(f, "handshake failed: {}", e),
            SocketError::SendFailed(e) => write!(f, "send failed: {}", e),
            SocketError::ReceiveFailed(e) => write!(f, "receive failed: {}", e),
            SocketError::EncryptionFailed => write!(f, "encryption failed"),
            SocketError::DecryptionFailed => write!(f, "decryption failed"),
            SocketError::InvalidFrame => write!(f, "invalid frame"),
            SocketError::NotConnected => write!(f, "not connected"),
            SocketError::ConnectionClosed => write!(f, "connection closed"),
        }
    }
}

impl std::error::Error for SocketError {}
