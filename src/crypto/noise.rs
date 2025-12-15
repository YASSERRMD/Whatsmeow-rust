//! Noise Protocol implementation for WhatsApp handshake.
//!
//! WhatsApp uses Noise_XX_25519_AESGCM_SHA256 for the initial handshake.

use crate::crypto::{Cipher, CipherError, Hkdf, KeyPair};
use sha2::{Sha256, Digest};

/// Noise Protocol pattern identifier.
pub const NOISE_PROTOCOL_NAME: &[u8] = b"Noise_XX_25519_AESGCM_SHA256\0\0\0\0";

/// Noise handshake state.
pub struct NoiseHandshake {
    /// Local static key pair
    local_static: KeyPair,
    /// Local ephemeral key pair
    local_ephemeral: Option<KeyPair>,
    /// Remote static public key
    remote_static: Option<[u8; 32]>,
    /// Remote ephemeral public key
    remote_ephemeral: Option<[u8; 32]>,
    /// Chaining key (ck)
    chaining_key: [u8; 32],
    /// Hash state (h)
    hash: [u8; 32],
    /// Cipher for encryption
    cipher: Option<Cipher>,
}

impl NoiseHandshake {
    /// Initialize a new Noise handshake as initiator.
    pub fn new_initiator(local_static: KeyPair) -> Self {
        let mut hs = Self {
            local_static,
            local_ephemeral: None,
            remote_static: None,
            remote_ephemeral: None,
            chaining_key: [0u8; 32],
            hash: [0u8; 32],
            cipher: None,
        };
        hs.initialize();
        hs
    }

    /// Initialize a new Noise handshake as responder.
    pub fn new_responder(local_static: KeyPair) -> Self {
        Self::new_initiator(local_static)
    }

    /// Initialize the handshake state.
    fn initialize(&mut self) {
        // h = SHA256(protocol_name)
        let mut hasher = Sha256::new();
        hasher.update(NOISE_PROTOCOL_NAME);
        self.hash = hasher.finalize().into();
        
        // ck = h
        self.chaining_key = self.hash;
        
        // Generate ephemeral key
        self.local_ephemeral = Some(KeyPair::generate());
    }

    /// Mix a value into the hash.
    fn mix_hash(&mut self, data: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(&self.hash);
        hasher.update(data);
        self.hash = hasher.finalize().into();
    }

    /// Mix a key into the chaining key and return new cipher key.
    fn mix_key(&mut self, input: &[u8]) -> [u8; 32] {
        let derived = Hkdf::derive(Some(&self.chaining_key), input, b"", 64);
        self.chaining_key.copy_from_slice(&derived[0..32]);
        let mut key = [0u8; 32];
        key.copy_from_slice(&derived[32..64]);
        key
    }

    /// Encrypt and authenticate data.
    fn encrypt_and_hash(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, CipherError> {
        let cipher = self.cipher.as_mut().ok_or(CipherError::InvalidKey)?;
        let ciphertext = cipher.encrypt(plaintext, &self.hash)?;
        self.mix_hash(&ciphertext);
        Ok(ciphertext)
    }

    /// Decrypt and verify data.
    fn decrypt_and_hash(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
        let cipher = self.cipher.as_mut().ok_or(CipherError::InvalidKey)?;
        let plaintext = cipher.decrypt(ciphertext, &self.hash)?;
        self.mix_hash(ciphertext);
        Ok(plaintext)
    }

    /// Write the first handshake message (-> e).
    pub fn write_message_1(&mut self) -> Vec<u8> {
        let ephemeral_public = self.local_ephemeral.as_ref().expect("ephemeral key not set").public;
        
        // Mix ephemeral public key into hash
        self.mix_hash(&ephemeral_public);
        
        // Return ephemeral public key
        ephemeral_public.to_vec()
    }

    /// Read the second handshake message (<- e, ee, s, es).
    pub fn read_message_2(&mut self, message: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        if message.len() < 32 {
            return Err(HandshakeError::MessageTooShort);
        }

        // Extract remote ephemeral (e)
        let mut remote_e = [0u8; 32];
        remote_e.copy_from_slice(&message[0..32]);
        self.remote_ephemeral = Some(remote_e);
        self.mix_hash(&remote_e);

        // Perform ee DH - clone ephemeral to avoid borrow issues
        let ephemeral = self.local_ephemeral.clone().expect("ephemeral key not set");
        let shared_ee = ephemeral.dh(&remote_e);
        let key = self.mix_key(&shared_ee);
        self.cipher = Some(Cipher::new(key));

        // Decrypt remote static (s)
        let encrypted_s = &message[32..32 + 48]; // 32 bytes + 16 tag
        let remote_s = self.decrypt_and_hash(encrypted_s)
            .map_err(|_| HandshakeError::DecryptionFailed)?;
        
        if remote_s.len() != 32 {
            return Err(HandshakeError::InvalidKeySize);
        }
        let mut remote_static = [0u8; 32];
        remote_static.copy_from_slice(&remote_s);
        self.remote_static = Some(remote_static);

        // Perform es DH
        let shared_es = ephemeral.dh(&remote_static);
        let key = self.mix_key(&shared_es);
        self.cipher = Some(Cipher::new(key));

        // Decrypt payload
        let encrypted_payload = &message[80..];
        let payload = self.decrypt_and_hash(encrypted_payload)
            .map_err(|_| HandshakeError::DecryptionFailed)?;

        Ok(payload)
    }

    /// Write the third handshake message (-> s, se).
    pub fn write_message_3(&mut self, payload: &[u8]) -> Result<Vec<u8>, HandshakeError> {
        let mut message = Vec::new();

        // Encrypt local static (s) - clone to avoid borrow conflict
        let local_static_public = self.local_static.public;
        let encrypted_s = self.encrypt_and_hash(&local_static_public)
            .map_err(|_| HandshakeError::EncryptionFailed)?;
        message.extend_from_slice(&encrypted_s);

        // Perform se DH
        let remote_e = self.remote_ephemeral.ok_or(HandshakeError::MissingRemoteKey)?;
        let shared_se = self.local_static.dh(&remote_e);
        let key = self.mix_key(&shared_se);
        self.cipher = Some(Cipher::new(key));

        // Encrypt payload
        let encrypted_payload = self.encrypt_and_hash(payload)
            .map_err(|_| HandshakeError::EncryptionFailed)?;
        message.extend_from_slice(&encrypted_payload);

        Ok(message)
    }

    /// Split into transport ciphers after handshake completes.
    pub fn split(self) -> (Cipher, Cipher) {
        let derived = Hkdf::derive(Some(&self.chaining_key), &[], b"", 64);
        
        let mut send_key = [0u8; 32];
        let mut recv_key = [0u8; 32];
        send_key.copy_from_slice(&derived[0..32]);
        recv_key.copy_from_slice(&derived[32..64]);
        
        (Cipher::new(send_key), Cipher::new(recv_key))
    }

    /// Get the remote static public key after handshake.
    pub fn remote_static_key(&self) -> Option<&[u8; 32]> {
        self.remote_static.as_ref()
    }
}

/// Handshake errors.
#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeError {
    MessageTooShort,
    DecryptionFailed,
    EncryptionFailed,
    InvalidKeySize,
    MissingRemoteKey,
}

impl std::fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeError::MessageTooShort => write!(f, "message too short"),
            HandshakeError::DecryptionFailed => write!(f, "decryption failed"),
            HandshakeError::EncryptionFailed => write!(f, "encryption failed"),
            HandshakeError::InvalidKeySize => write!(f, "invalid key size"),
            HandshakeError::MissingRemoteKey => write!(f, "missing remote key"),
        }
    }
}

impl std::error::Error for HandshakeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_initialization() {
        let kp = KeyPair::generate();
        let hs = NoiseHandshake::new_initiator(kp);
        
        assert!(hs.local_ephemeral.is_some());
        assert_ne!(hs.hash, [0u8; 32]);
    }

    #[test]
    fn test_write_message_1() {
        let kp = KeyPair::generate();
        let mut hs = NoiseHandshake::new_initiator(kp);
        
        let msg = hs.write_message_1();
        assert_eq!(msg.len(), 32);
    }
}
