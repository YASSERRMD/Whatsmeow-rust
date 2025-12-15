//! Complete WhatsApp handshake implementation.
//!
//! Implements the Noise_XX_25519_AESGCM_SHA256 handshake for WhatsApp Web.

use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::http::Request;
use tokio::net::TcpStream;
use futures::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead, Nonce};

use crate::crypto::{KeyPair, Hkdf};
use crate::store::Device;
use crate::proto::{
    HandshakeMessage, ClientHello, ClientFinish,
    ClientPayload, make_web_client_payload, make_device_pairing_data,
};

/// WhatsApp WebSocket endpoints
pub const WA_ENDPOINT: &str = "wss://web.whatsapp.com/ws/chat";
pub const WA_ORIGIN: &str = "https://web.whatsapp.com";

/// Noise protocol pattern name (exactly 32 bytes)
const NOISE_PATTERN: &[u8; 32] = b"Noise_XX_25519_AESGCM_SHA256\x00\x00\x00\x00";

/// WhatsApp connection header: 'W', 'A', MagicValue(6), DictVersion(3)
const WA_HEADER: [u8; 4] = [b'W', b'A', 6, 3];

/// Handshake errors
#[derive(Debug)]
pub enum HandshakeError {
    ConnectionFailed(String),
    Timeout,
    InvalidResponse(String),
    CryptoError(String),
    ProtocolError(String),
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeError::ConnectionFailed(e) => write!(f, "connection failed: {}", e),
            HandshakeError::Timeout => write!(f, "timeout"),
            HandshakeError::InvalidResponse(e) => write!(f, "invalid response: {}", e),
            HandshakeError::CryptoError(e) => write!(f, "crypto error: {}", e),
            HandshakeError::ProtocolError(e) => write!(f, "protocol error: {}", e),
        }
    }
}

impl std::error::Error for HandshakeError {}

/// Noise handshake state matching whatsmeow's implementation
pub struct NoiseHandshake {
    /// Hash state (h)
    hash: [u8; 32],
    /// Salt/chaining key
    salt: [u8; 32],
    /// Current cipher key
    key: [u8; 32],
    /// Counter for GCM nonces
    counter: u32,
}

impl NoiseHandshake {
    /// Start the handshake with pattern and header
    pub fn new(header: &[u8]) -> Self {
        // Pattern is exactly 32 bytes so use directly
        let hash: [u8; 32] = *NOISE_PATTERN;
        let salt = hash;
        let key = hash;
        
        let mut state = Self { hash, salt, key, counter: 0 };
        
        // Authenticate the header (prologue)
        state.authenticate(header);
        
        state
    }

    /// Mix data into the hash (authenticate)
    fn authenticate(&mut self, data: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(&self.hash);
        hasher.update(data);
        self.hash = hasher.finalize().into();
    }

    /// Generate IV for AES-GCM from counter
    fn generate_iv(&self) -> [u8; 12] {
        let mut iv = [0u8; 12];
        iv[8..12].copy_from_slice(&self.counter.to_be_bytes());
        iv
    }

    /// Encrypt using current key
    fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| HandshakeError::CryptoError("invalid key".to_string()))?;
        let iv = self.generate_iv();
        let nonce = Nonce::from_slice(&iv);
        
        // GCM with AAD = hash
        let ciphertext = cipher.encrypt(nonce, aes_gcm::aead::Payload {
            msg: plaintext,
            aad: &self.hash,
        }).map_err(|_| HandshakeError::CryptoError("encryption failed".to_string()))?;
        
        self.counter += 1;
        self.authenticate(&ciphertext);
        
        Ok(ciphertext)
    }

    /// Decrypt using current key
    fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| HandshakeError::CryptoError("invalid key".to_string()))?;
        let iv = self.generate_iv();
        let nonce = Nonce::from_slice(&iv);
        
        let plaintext = cipher.decrypt(nonce, aes_gcm::aead::Payload {
            msg: ciphertext,
            aad: &self.hash,
        }).map_err(|_| HandshakeError::CryptoError("decryption failed".to_string()))?;
        
        self.counter += 1;
        self.authenticate(ciphertext);
        
        Ok(plaintext)
    }

    /// Mix shared secret into key (after DH)
    fn mix_into_key(&mut self, shared_secret: &[u8]) -> Result<(), HandshakeError> {
        self.counter = 0;
        
        // HKDF extract and expand
        let derived = Hkdf::derive(Some(&self.salt), shared_secret, b"", 64);
        
        self.salt.copy_from_slice(&derived[0..32]);
        self.key.copy_from_slice(&derived[32..64]);
        
        Ok(())
    }

    /// Mix shared secret from DH
    fn mix_shared_secret(&mut self, priv_key: &[u8; 32], pub_key: &[u8; 32]) -> Result<(), HandshakeError> {
        // X25519 DH
        let shared = x25519_dalek::x25519(*priv_key, *pub_key);
        self.mix_into_key(&shared)
    }

    /// Extract final keys for transport
    fn finish(&self) -> Result<([u8; 32], [u8; 32]), HandshakeError> {
        let derived = Hkdf::derive(Some(&self.salt), &[], b"", 64);
        
        let mut write_key = [0u8; 32];
        let mut read_key = [0u8; 32];
        write_key.copy_from_slice(&derived[0..32]);
        read_key.copy_from_slice(&derived[32..64]);
        
        Ok((write_key, read_key))
    }
}

/// WhatsApp connection with completed handshake
pub struct WhatsAppConnection {
    pub ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub write_key: [u8; 32],
    pub read_key: [u8; 32],
    pub write_counter: u32,
    pub read_counter: u32,
    pub device: Device,
}

impl WhatsAppConnection {
    /// Send an encrypted frame
    pub async fn send(&mut self, data: &[u8]) -> Result<(), HandshakeError> {
        let cipher = Aes256Gcm::new_from_slice(&self.write_key)
            .map_err(|_| HandshakeError::CryptoError("invalid key".to_string()))?;
        
        let mut iv = [0u8; 12];
        iv[8..12].copy_from_slice(&self.write_counter.to_be_bytes());
        let nonce = Nonce::from_slice(&iv);
        
        let encrypted = cipher.encrypt(nonce, data)
            .map_err(|_| HandshakeError::CryptoError("encryption failed".to_string()))?;
        
        self.write_counter += 1;
        
        // Frame format: length (3 bytes) + encrypted data
        let len = encrypted.len();
        let mut frame = Vec::with_capacity(len + 3);
        frame.push(((len >> 16) & 0xFF) as u8);
        frame.push(((len >> 8) & 0xFF) as u8);
        frame.push((len & 0xFF) as u8);
        frame.extend_from_slice(&encrypted);
        
        self.ws.send(Message::Binary(frame.into())).await
            .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))
    }

    /// Receive and decrypt a frame
    pub async fn recv(&mut self) -> Result<Vec<u8>, HandshakeError> {
        loop {
            let msg = timeout(Duration::from_secs(30), self.ws.next()).await
                .map_err(|_| HandshakeError::Timeout)?
                .ok_or(HandshakeError::ConnectionFailed("connection closed".to_string()))?
                .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;

            match msg {
                Message::Binary(data) => {
                    // Skip very short frames (likely keep-alive or error)
                    if data.len() < 4 {
                        println!("   [recv] Skipping short frame ({} bytes)", data.len());
                        continue;
                    }
                    
                    // Parse length header
                    let frame_len = ((data[0] as usize) << 16) 
                                  | ((data[1] as usize) << 8) 
                                  | (data[2] as usize);
                    
                    // Sanity check - if frame_len is way off, the data might be malformed
                    if frame_len > 100000 || frame_len + 3 > data.len() + 100 {
                        println!("   [recv] Skipping malformed frame (len={}, data={})", frame_len, data.len());
                        continue;
                    }
                    
                    println!("   [recv] Frame: {} total, header says {}, counter={}", 
                             data.len(), frame_len, self.read_counter);
                    
                    // The encrypted data is after the 3-byte length prefix
                    let encrypted = &data[3..];
                    
                    if encrypted.is_empty() {
                        println!("   [recv] Empty encrypted data, skipping");
                        continue;
                    }
                    
                    let cipher = Aes256Gcm::new_from_slice(&self.read_key)
                        .map_err(|_| HandshakeError::CryptoError("invalid key".to_string()))?;
                    
                    let mut iv = [0u8; 12];
                    iv[8..12].copy_from_slice(&self.read_counter.to_be_bytes());
                    let nonce = Nonce::from_slice(&iv);
                    
                    match cipher.decrypt(nonce, encrypted) {
                        Ok(decrypted) => {
                            self.read_counter += 1;
                            return Ok(decrypted);
                        }
                        Err(_) => {
                            println!("   [recv] Decryption failed, trying next counter...");
                            // Try incrementing counter in case we missed a message
                            self.read_counter += 1;
                            let mut iv2 = [0u8; 12];
                            iv2[8..12].copy_from_slice(&self.read_counter.to_be_bytes());
                            let nonce2 = Nonce::from_slice(&iv2);
                            
                            if let Ok(decrypted) = cipher.decrypt(nonce2, encrypted) {
                                self.read_counter += 1;
                                return Ok(decrypted);
                            }
                            
                            // Give up on this frame
                            println!("   [recv] Still failed, skipping frame");
                            continue;
                        }
                    }
                }
                Message::Close(frame) => {
                    let reason = frame.map(|f| format!("{}: {}", f.code, f.reason)).unwrap_or_default();
                    return Err(HandshakeError::ConnectionFailed(format!("connection closed: {}", reason)));
                }
                Message::Ping(_) | Message::Pong(_) => {
                    println!("   [recv] Ping/Pong");
                    continue;
                }
                _ => {
                    println!("   [recv] Unexpected message type, skipping");
                    continue;
                }
            }
        }
    }
}

/// Perform complete WhatsApp handshake
pub async fn do_handshake(device: &Device) -> Result<WhatsAppConnection, HandshakeError> {
    // Get device keys
    let noise_key = device.noise_key.as_ref()
        .ok_or(HandshakeError::ProtocolError("no noise key".to_string()))?;
    let identity_key = device.identity_key.as_ref()
        .ok_or(HandshakeError::ProtocolError("no identity key".to_string()))?;
    let signed_prekey = device.signed_pre_key.as_ref()
        .ok_or(HandshakeError::ProtocolError("no signed prekey".to_string()))?;

    // Generate ephemeral key pair for this session
    let ephemeral_priv: [u8; 32] = rand::random();
    let ephemeral_pub = x25519_dalek::x25519(ephemeral_priv, x25519_dalek::X25519_BASEPOINT_BYTES);

    // Connect to WhatsApp
    println!("   Connecting to {}...", WA_ENDPOINT);

    let (mut ws, _) = timeout(Duration::from_secs(10), connect_async(WA_ENDPOINT)).await
        .map_err(|_| HandshakeError::Timeout)?
        .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;
    println!("   ✓ Connected");

    // Initialize Noise handshake state
    let mut noise = NoiseHandshake::new(&WA_HEADER);

    // === Message 1: -> e (send ephemeral public key) ===
    println!("   Sending handshake message 1 (-> e)...");
    
    // Authenticate ephemeral public (mix into hash)
    noise.authenticate(&ephemeral_pub);

    let client_hello = HandshakeMessage {
        client_hello: Some(ClientHello {
            ephemeral: Some(ephemeral_pub.to_vec()),
        }),
        server_hello: None,
        client_finish: None,
    };
    
    let mut msg1_proto = Vec::new();
    client_hello.encode(&mut msg1_proto)
        .map_err(|e| HandshakeError::ProtocolError(e.to_string()))?;

    // First frame: WA header + 3-byte length + protobuf
    let mut frame = Vec::new();
    frame.extend_from_slice(&WA_HEADER);
    let len = msg1_proto.len();
    frame.push(((len >> 16) & 0xFF) as u8);
    frame.push(((len >> 8) & 0xFF) as u8);
    frame.push((len & 0xFF) as u8);
    frame.extend_from_slice(&msg1_proto);
    
    println!("   Sending {} bytes: header={:02x?}, length={}", 
             frame.len(), &frame[..4], len);
    
    ws.send(Message::Binary(frame.into())).await
        .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;
    println!("   ✓ Message 1 sent");

    // === Message 2: <- e, ee, s, es ===
    println!("   Waiting for handshake message 2...");
    
    // Accumulate response data
    let mut response_data = Vec::new();
    let mut frame_len: Option<usize> = None;
    
    for attempt in 0..10 {
        let response = timeout(Duration::from_secs(5), ws.next()).await
            .map_err(|_| HandshakeError::Timeout)?
            .ok_or(HandshakeError::ConnectionFailed("no response".to_string()))?
            .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;

        match response {
            Message::Binary(data) => {
                response_data.extend_from_slice(&data);
                println!("   ✓ Received {} bytes (attempt {}): {:02x?}...", 
                         data.len(), attempt + 1, &data[..data.len().min(20)]);
                
                // Parse frame length once we have at least 3 bytes
                if frame_len.is_none() && response_data.len() >= 3 {
                    let len = ((response_data[0] as usize) << 16) 
                            | ((response_data[1] as usize) << 8) 
                            | (response_data[2] as usize);
                    frame_len = Some(len);
                    println!("   Frame declares {} bytes of content", len);
                }
                
                // Check if we have complete frame
                if let Some(len) = frame_len {
                    if response_data.len() >= len + 3 {
                        // Extract just the protobuf content
                        response_data = response_data[3..3+len].to_vec();
                        println!("   ✓ Complete frame received: {} bytes protobuf", response_data.len());
                        break;
                    }
                }
            }
            Message::Close(frame) => {
                let reason = frame.map(|f| format!("{}: {}", f.code, f.reason)).unwrap_or_default();
                return Err(HandshakeError::ConnectionFailed(format!("server closed: {}", reason)));
            }
            _ => {}
        }
    }
    
    if response_data.is_empty() {
        return Err(HandshakeError::InvalidResponse("no data received".to_string()));
    }

    // Decode server hello
    let server_hello_msg = HandshakeMessage::decode(&response_data[..])
        .map_err(|e| HandshakeError::ProtocolError(format!("failed to decode HandshakeMessage: {}", e)))?;

    let server_hello = server_hello_msg.server_hello
        .ok_or(HandshakeError::InvalidResponse("missing server_hello in response".to_string()))?;

    let server_ephemeral = server_hello.ephemeral
        .ok_or(HandshakeError::InvalidResponse("missing server ephemeral".to_string()))?;
    let server_static_ciphertext = server_hello.r#static
        .ok_or(HandshakeError::InvalidResponse("missing server static".to_string()))?;
    let cert_ciphertext = server_hello.payload
        .ok_or(HandshakeError::InvalidResponse("missing server payload".to_string()))?;

    if server_ephemeral.len() != 32 {
        return Err(HandshakeError::InvalidResponse(
            format!("invalid server ephemeral length: {} (expected 32)", server_ephemeral.len())
        ));
    }

    let mut server_eph_arr = [0u8; 32];
    server_eph_arr.copy_from_slice(&server_ephemeral);

    println!("   Server ephemeral: {:02x?}...", &server_ephemeral[..8]);

    // Authenticate server ephemeral
    noise.authenticate(&server_ephemeral);

    // ee: DH(ephemeral_priv, server_ephemeral)
    noise.mix_shared_secret(&ephemeral_priv, &server_eph_arr)?;

    // Decrypt server static public key
    let server_static = noise.decrypt(&server_static_ciphertext)?;
    if server_static.len() != 32 {
        return Err(HandshakeError::InvalidResponse(
            format!("invalid server static length: {} (expected 32)", server_static.len())
        ));
    }
    let mut server_static_arr = [0u8; 32];
    server_static_arr.copy_from_slice(&server_static);

    println!("   Server static: {:02x?}...", &server_static[..8]);

    // es: DH(ephemeral_priv, server_static)
    noise.mix_shared_secret(&ephemeral_priv, &server_static_arr)?;

    // Decrypt certificate (we don't verify it for now, just decrypt)
    let _cert = noise.decrypt(&cert_ciphertext)?;
    println!("   ✓ Server certificate decrypted ({} bytes)", _cert.len());

    // === Message 3: -> s, se ===
    println!("   Sending handshake message 3 (-> s, se)...");

    // Encrypt our static public key
    let static_encrypted = noise.encrypt(&noise_key.public)?;

    // se: DH(noise_priv, server_ephemeral)
    let noise_priv: [u8; 32] = noise_key.private;
    noise.mix_shared_secret(&noise_priv, &server_eph_arr)?;

    // Build client payload with device pairing data
    let signature = signed_prekey.signature.unwrap_or([0u8; 64]);
    let pairing_data = make_device_pairing_data(
        device.registration_id,
        &identity_key.public,
        signed_prekey.key_id,
        &signed_prekey.key_pair.public,
        &signature,
    );

    let mut client_payload = make_web_client_payload(device.push_name.as_deref());
    client_payload.device_pairing_data = Some(pairing_data);

    let mut payload_bytes = Vec::new();
    client_payload.encode(&mut payload_bytes)
        .map_err(|e| HandshakeError::ProtocolError(e.to_string()))?;

    let payload_encrypted = noise.encrypt(&payload_bytes)?;

    let client_finish = HandshakeMessage {
        client_hello: None,
        server_hello: None,
        client_finish: Some(ClientFinish {
            r#static: Some(static_encrypted),
            payload: Some(payload_encrypted),
        }),
    };

    let mut msg3_data = Vec::new();
    client_finish.encode(&mut msg3_data)
        .map_err(|e| HandshakeError::ProtocolError(e.to_string()))?;

    // Frame: 3-byte length + protobuf (no header on subsequent frames)
    let len3 = msg3_data.len();
    let mut frame3 = Vec::with_capacity(len3 + 3);
    frame3.push(((len3 >> 16) & 0xFF) as u8);
    frame3.push(((len3 >> 8) & 0xFF) as u8);
    frame3.push((len3 & 0xFF) as u8);
    frame3.extend_from_slice(&msg3_data);

    ws.send(Message::Binary(frame3.into())).await
        .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;
    println!("   ✓ Message 3 sent ({} bytes)", len3);

    // Get final ciphers
    let (write_key, read_key) = noise.finish()?;
    println!("   ✓ Handshake complete!");

    Ok(WhatsAppConnection {
        ws,
        write_key,
        read_key,
        write_counter: 0,
        read_counter: 0,
        device: device.clone(),
    })
}
