//! Complete WhatsApp handshake implementation.
//!
//! Implements the Noise_XX_25519_AESGCM_SHA256 handshake for WhatsApp Web.

use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;
use futures::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use sha2::{Sha256, Digest};

use crate::crypto::{KeyPair, Cipher, Hkdf};
use crate::store::Device;
use crate::proto::{
    HandshakeMessage, ClientHello, ServerHello, ClientFinish,
    ClientPayload, make_web_client_payload, make_device_pairing_data,
};

/// WhatsApp WebSocket endpoints
pub const WA_ENDPOINT: &str = "wss://web.whatsapp.com/ws/chat";

/// Noise protocol pattern name
const NOISE_PATTERN: &[u8] = b"Noise_XX_25519_AESGCM_SHA256\0\0\0\0";

/// WhatsApp header
const WA_HEADER: &[u8] = b"WA\x06\x00";

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

/// Noise handshake state
pub struct NoiseHandshakeState {
    /// Current hash value
    hash: [u8; 32],
    /// Chaining key
    ck: [u8; 32],
    /// Current cipher (after first key exchange)
    cipher: Option<Cipher>,
}

impl NoiseHandshakeState {
    /// Initialize the handshake with the pattern name
    pub fn new(header: &[u8]) -> Self {
        // Initialize h with SHA256(pattern_name)
        let mut hasher = Sha256::new();
        hasher.update(NOISE_PATTERN);
        let hash: [u8; 32] = hasher.finalize().into();
        
        // ck = h
        let ck = hash;
        
        // Mix in the header (prologue)
        let mut state = Self { hash, ck, cipher: None };
        state.mix_hash(header);
        
        state
    }

    /// Mix data into the hash
    fn mix_hash(&mut self, data: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(&self.hash);
        hasher.update(data);
        self.hash = hasher.finalize().into();
    }

    /// Mix a shared secret into the chaining key
    fn mix_shared_secret(&mut self, shared_secret: &[u8]) {
        let derived = Hkdf::derive(Some(&self.ck), shared_secret, b"", 64);
        self.ck.copy_from_slice(&derived[0..32]);
        let mut cipher_key = [0u8; 32];
        cipher_key.copy_from_slice(&derived[32..64]);
        self.cipher = Some(Cipher::new(cipher_key));
    }

    /// Encrypt data
    fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        if let Some(ref mut cipher) = self.cipher {
            let ciphertext = cipher.encrypt(plaintext, &self.hash).unwrap();
            self.mix_hash(&ciphertext);
            ciphertext
        } else {
            self.mix_hash(plaintext);
            plaintext.to_vec()
        }
    }

    /// Decrypt data
    fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        if let Some(ref mut cipher) = self.cipher {
            let plaintext = cipher.decrypt(ciphertext, &self.hash)
                .map_err(|_| HandshakeError::CryptoError("decryption failed".to_string()))?;
            self.mix_hash(ciphertext);
            Ok(plaintext)
        } else {
            self.mix_hash(ciphertext);
            Ok(ciphertext.to_vec())
        }
    }

    /// Get final send/receive ciphers
    fn finish(&self) -> (Cipher, Cipher) {
        let derived = Hkdf::derive(Some(&self.ck), &[], b"", 64);
        let mut send_key = [0u8; 32];
        let mut recv_key = [0u8; 32];
        send_key.copy_from_slice(&derived[0..32]);
        recv_key.copy_from_slice(&derived[32..64]);
        (Cipher::new(send_key), Cipher::new(recv_key))
    }
}

/// WhatsApp connection with completed handshake
pub struct WhatsAppConnection {
    pub ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub send_cipher: Cipher,
    pub recv_cipher: Cipher,
    pub device: Device,
}

impl WhatsAppConnection {
    /// Send an encrypted frame
    pub async fn send(&mut self, data: &[u8]) -> Result<(), HandshakeError> {
        let encrypted = self.send_cipher.encrypt(data, &[])
            .map_err(|_| HandshakeError::CryptoError("encryption failed".to_string()))?;
        
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
        let msg = timeout(Duration::from_secs(30), self.ws.next()).await
            .map_err(|_| HandshakeError::Timeout)?
            .ok_or(HandshakeError::ConnectionFailed("connection closed".to_string()))?
            .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;

        match msg {
            Message::Binary(data) => {
                if data.len() < 3 {
                    return Err(HandshakeError::InvalidResponse("frame too short".to_string()));
                }
                // Skip length prefix and decrypt
                let encrypted = &data[3..];
                self.recv_cipher.decrypt(encrypted, &[])
                    .map_err(|_| HandshakeError::CryptoError("decryption failed".to_string()))
            }
            Message::Close(_) => Err(HandshakeError::ConnectionFailed("connection closed".to_string())),
            _ => Err(HandshakeError::InvalidResponse("unexpected message type".to_string())),
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

    // Generate ephemeral key pair
    let ephemeral = KeyPair::generate();

    // Connect to WhatsApp
    println!("   Connecting to {}...", WA_ENDPOINT);
    let (mut ws, _) = timeout(Duration::from_secs(10), connect_async(WA_ENDPOINT)).await
        .map_err(|_| HandshakeError::Timeout)?
        .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;
    println!("   ✓ Connected");

    // Initialize Noise state
    let mut noise = NoiseHandshakeState::new(WA_HEADER);

    // === Message 1: -> e ===
    println!("   Sending handshake message 1 (-> e)...");
    noise.mix_hash(&ephemeral.public);

    let client_hello = HandshakeMessage {
        client_hello: Some(ClientHello {
            ephemeral: Some(ephemeral.public.to_vec()),
        }),
        server_hello: None,
        client_finish: None,
    };
    
    let mut msg1_data = Vec::new();
    client_hello.encode(&mut msg1_data)
        .map_err(|e| HandshakeError::ProtocolError(e.to_string()))?;

    // First frame: WA header (intro) + 3-byte length + protobuf
    // The intro header WA\x06\x00 is sent only once at connection start
    let mut frame = Vec::new();
    frame.extend_from_slice(WA_HEADER); // WA\x06\x00 - connection intro
    // Then length prefix (3 bytes, big-endian)
    let len = msg1_data.len();
    frame.push(((len >> 16) & 0xFF) as u8);
    frame.push(((len >> 8) & 0xFF) as u8);
    frame.push((len & 0xFF) as u8);
    frame.extend_from_slice(&msg1_data);
    
    println!("   Frame: {} bytes (header: {:02x?})", frame.len(), &frame[..7.min(frame.len())]);
    
    ws.send(Message::Binary(frame.into())).await
        .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;
    println!("   ✓ Message 1 sent ({} bytes protobuf)", len);

    // === Message 2: <- e, ee, s, es ===
    println!("   Waiting for handshake message 2...");
    
    // Accumulate response data
    let mut response_data = Vec::new();
    
    // Give server time to respond with full frame
    for attempt in 0..5 {
        let response = timeout(Duration::from_secs(5), ws.next()).await
            .map_err(|_| HandshakeError::Timeout)?
            .ok_or(HandshakeError::ConnectionFailed("no response".to_string()))?
            .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;

        match response {
            Message::Binary(data) => {
                println!("   ✓ Received {} bytes: {:02x?}", data.len(), &data[..data.len().min(16)]);
                response_data.extend_from_slice(&data);
                
                // If we got a reasonable amount of data, try to parse
                if response_data.len() >= 100 {
                    break;
                }
            }
            Message::Close(frame) => {
                let reason = frame.map(|f| format!("{}: {}", f.code, f.reason)).unwrap_or_default();
                return Err(HandshakeError::ConnectionFailed(format!("server closed: {}", reason)));
            }
            _ => {
                println!("   Got other message type on attempt {}", attempt);
            }
        }
    }
    
    if response_data.is_empty() {
        return Err(HandshakeError::InvalidResponse("no data received".to_string()));
    }

    // Try to find where the protobuf starts - skip potential length header
    let proto_start = if response_data.len() >= 3 {
        let potential_len = ((response_data[0] as usize) << 16) 
                          | ((response_data[1] as usize) << 8) 
                          | (response_data[2] as usize);
        if potential_len > 0 && potential_len <= response_data.len() - 3 && potential_len < 10000 {
            println!("   Detected length prefix: {} bytes", potential_len);
            3
        } else {
            0
        }
    } else {
        0
    };
    
    let proto_data = &response_data[proto_start..];
    println!("   Decoding {} bytes of protobuf...", proto_data.len());

    let server_hello_msg = HandshakeMessage::decode(proto_data)
        .map_err(|e| HandshakeError::ProtocolError(format!("failed to decode: {}", e)))?;

    let server_hello = server_hello_msg.server_hello
        .ok_or(HandshakeError::InvalidResponse("missing server_hello".to_string()))?;

    let server_ephemeral = server_hello.ephemeral
        .ok_or(HandshakeError::InvalidResponse("missing server ephemeral".to_string()))?;
    let server_static_ciphertext = server_hello.r#static
        .ok_or(HandshakeError::InvalidResponse("missing server static".to_string()))?;
    let cert_ciphertext = server_hello.payload
        .ok_or(HandshakeError::InvalidResponse("missing server payload".to_string()))?;

    if server_ephemeral.len() != 32 {
        return Err(HandshakeError::InvalidResponse("invalid server ephemeral length".to_string()));
    }

    let mut server_ephemeral_arr = [0u8; 32];
    server_ephemeral_arr.copy_from_slice(&server_ephemeral);

    // Mix server ephemeral
    noise.mix_hash(&server_ephemeral);

    // ee: DH(ephemeral, server_ephemeral)
    let shared_ee = ephemeral.dh(&server_ephemeral_arr);
    noise.mix_shared_secret(&shared_ee);

    // Decrypt server static
    let server_static = noise.decrypt(&server_static_ciphertext)?;
    if server_static.len() != 32 {
        return Err(HandshakeError::InvalidResponse("invalid server static length".to_string()));
    }
    let mut server_static_arr = [0u8; 32];
    server_static_arr.copy_from_slice(&server_static);

    // es: DH(ephemeral, server_static)
    let shared_es = ephemeral.dh(&server_static_arr);
    noise.mix_shared_secret(&shared_es);

    // Decrypt and verify certificate (simplified - just decrypt)
    let _cert = noise.decrypt(&cert_ciphertext)?;
    println!("   ✓ Server certificate decrypted");

    // === Message 3: -> s, se ===
    println!("   Sending handshake message 3 (-> s, se)...");

    // Encrypt our static key
    let static_encrypted = noise.encrypt(&noise_key.public);

    // se: DH(noise_key, server_ephemeral)
    let shared_se = noise_key.dh(&server_ephemeral_arr);
    noise.mix_shared_secret(&shared_se);

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

    let payload_encrypted = noise.encrypt(&payload_bytes);

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

    ws.send(Message::Binary(msg3_data.into())).await
        .map_err(|e| HandshakeError::ConnectionFailed(e.to_string()))?;
    println!("   ✓ Message 3 sent");

    // Get final ciphers
    let (send_cipher, recv_cipher) = noise.finish();
    println!("   ✓ Handshake complete!");

    Ok(WhatsAppConnection {
        ws,
        send_cipher,
        recv_cipher,
        device: device.clone(),
    })
}
