//! HKDF (HMAC-based Key Derivation Function) for WhatsApp protocol.
//!
//! Used in Noise Protocol and Signal Protocol for key derivation.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// HKDF-SHA256 key derivation.
pub struct Hkdf {
    prk: [u8; 32],
}

impl Hkdf {
    /// Create a new HKDF instance with the given input key material and salt.
    pub fn new(salt: Option<&[u8]>, ikm: &[u8]) -> Self {
        // HKDF-Extract
        let salt = salt.unwrap_or(&[0u8; 32]);
        let mut mac = HmacSha256::new_from_slice(salt)
            .expect("HMAC can take key of any size");
        mac.update(ikm);
        let prk: [u8; 32] = mac.finalize().into_bytes().into();
        
        Self { prk }
    }

    /// Expand the key to the desired length with optional info.
    pub fn expand(&self, info: &[u8], length: usize) -> Vec<u8> {
        let mut output = Vec::with_capacity(length);
        let mut t = Vec::new();
        let mut counter = 1u8;
        
        while output.len() < length {
            let mut mac = HmacSha256::new_from_slice(&self.prk)
                .expect("HMAC can take key of any size");
            mac.update(&t);
            mac.update(info);
            mac.update(&[counter]);
            t = mac.finalize().into_bytes().to_vec();
            
            let remaining = length - output.len();
            let to_copy = remaining.min(t.len());
            output.extend_from_slice(&t[..to_copy]);
            
            counter += 1;
        }
        
        output.truncate(length);
        output
    }

    /// Convenience function to extract and expand in one call.
    pub fn derive(salt: Option<&[u8]>, ikm: &[u8], info: &[u8], length: usize) -> Vec<u8> {
        let hkdf = Self::new(salt, ikm);
        hkdf.expand(info, length)
    }
}

/// Derive keys for Noise Protocol handshake.
pub fn derive_noise_keys(shared_secret: &[u8], salt: &[u8]) -> ([u8; 32], [u8; 32]) {
    let derived = Hkdf::derive(Some(salt), shared_secret, b"", 64);
    let mut key1 = [0u8; 32];
    let mut key2 = [0u8; 32];
    key1.copy_from_slice(&derived[0..32]);
    key2.copy_from_slice(&derived[32..64]);
    (key1, key2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_basic() {
        let ikm = [0x0b; 22];
        let salt = [0x00; 13];
        let info = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
        
        let output = Hkdf::derive(Some(&salt), &ikm, &info, 42);
        assert_eq!(output.len(), 42);
    }

    #[test]
    fn test_hkdf_no_salt() {
        let ikm = b"input key material";
        let output = Hkdf::derive(None, ikm, b"info", 32);
        assert_eq!(output.len(), 32);
    }

    #[test]
    fn test_derive_noise_keys() {
        let shared = [0xab; 32];
        let salt = [0xcd; 32];
        let (key1, key2) = derive_noise_keys(&shared, &salt);
        
        assert_ne!(key1, [0u8; 32]);
        assert_ne!(key2, [0u8; 32]);
        assert_ne!(key1, key2);
    }
}
