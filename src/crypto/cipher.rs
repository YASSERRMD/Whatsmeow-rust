//! AES-GCM cipher for WhatsApp protocol encryption.
//!
//! Used in Noise Protocol for symmetric encryption of frames.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};

/// AES-256-GCM cipher for encrypting/decrypting messages.
pub struct Cipher {
    key: [u8; 32],
    nonce_counter: u64,
}

impl Cipher {
    /// Create a new cipher with the given key.
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            nonce_counter: 0,
        }
    }

    /// Encrypt data with optional associated data.
    pub fn encrypt(&mut self, plaintext: &[u8], ad: &[u8]) -> Result<Vec<u8>, CipherError> {
        let nonce = self.next_nonce();
        self.encrypt_with_nonce(plaintext, &nonce, ad)
    }

    /// Encrypt with a specific nonce.
    pub fn encrypt_with_nonce(
        &self,
        plaintext: &[u8],
        nonce: &[u8; 12],
        ad: &[u8],
    ) -> Result<Vec<u8>, CipherError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| CipherError::InvalidKey)?;
        
        let nonce = Nonce::from_slice(nonce);
        
        // AES-GCM with AD
        cipher
            .encrypt(nonce, aes_gcm::aead::Payload { msg: plaintext, aad: ad })
            .map_err(|_| CipherError::EncryptionFailed)
    }

    /// Decrypt data with optional associated data.
    pub fn decrypt(&mut self, ciphertext: &[u8], ad: &[u8]) -> Result<Vec<u8>, CipherError> {
        let nonce = self.next_nonce();
        self.decrypt_with_nonce(ciphertext, &nonce, ad)
    }

    /// Decrypt with a specific nonce.
    pub fn decrypt_with_nonce(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; 12],
        ad: &[u8],
    ) -> Result<Vec<u8>, CipherError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|_| CipherError::InvalidKey)?;
        
        let nonce = Nonce::from_slice(nonce);
        
        cipher
            .decrypt(nonce, aes_gcm::aead::Payload { msg: ciphertext, aad: ad })
            .map_err(|_| CipherError::DecryptionFailed)
    }

    /// Generate the next nonce (counter-based).
    fn next_nonce(&mut self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        // Put counter in last 8 bytes (big-endian)
        nonce[4..12].copy_from_slice(&self.nonce_counter.to_be_bytes());
        self.nonce_counter += 1;
        nonce
    }

    /// Reset the nonce counter.
    pub fn reset_nonce(&mut self) {
        self.nonce_counter = 0;
    }

    /// Set the nonce counter to a specific value.
    pub fn set_nonce(&mut self, counter: u64) {
        self.nonce_counter = counter;
    }
}

/// Cipher errors.
#[derive(Debug, Clone, PartialEq)]
pub enum CipherError {
    InvalidKey,
    EncryptionFailed,
    DecryptionFailed,
}

impl std::fmt::Display for CipherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherError::InvalidKey => write!(f, "invalid key"),
            CipherError::EncryptionFailed => write!(f, "encryption failed"),
            CipherError::DecryptionFailed => write!(f, "decryption failed"),
        }
    }
}

impl std::error::Error for CipherError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0xab; 32];
        let mut cipher = Cipher::new(key);
        
        let plaintext = b"Hello, WhatsApp!";
        let ad = b"additional data";
        
        let ciphertext = cipher.encrypt(plaintext, ad).unwrap();
        cipher.reset_nonce();
        let decrypted = cipher.decrypt(&ciphertext, ad).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_ad_fails() {
        let key = [0xab; 32];
        let mut cipher = Cipher::new(key);
        
        let plaintext = b"Hello, WhatsApp!";
        let ciphertext = cipher.encrypt(plaintext, b"correct ad").unwrap();
        
        cipher.reset_nonce();
        let result = cipher.decrypt(&ciphertext, b"wrong ad");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_increments() {
        let key = [0xab; 32];
        let mut cipher = Cipher::new(key);
        
        let nonce1 = cipher.next_nonce();
        let nonce2 = cipher.next_nonce();
        
        assert_ne!(nonce1, nonce2);
    }
}
