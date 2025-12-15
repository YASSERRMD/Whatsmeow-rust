//! Cryptographic key pair utilities for WhatsApp protocol.
//!
//! Provides Curve25519 key pair generation and management for Signal Protocol.

use rand::RngCore;
use x25519_dalek::{PublicKey, StaticSecret};

/// A Curve25519 key pair for Signal Protocol operations.
#[derive(Clone)]
pub struct KeyPair {
    /// Public key (32 bytes)
    pub public: [u8; 32],
    /// Private key (32 bytes) 
    pub private: [u8; 32],
}

impl KeyPair {
    /// Generate a new random key pair.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let mut private = [0u8; 32];
        rng.fill_bytes(&mut private);
        
        // Apply clamping as per Curve25519 spec
        private[0] &= 248;
        private[31] &= 127;
        private[31] |= 64;
        
        Self::from_private_key(private)
    }

    /// Create a key pair from an existing private key.
    pub fn from_private_key(private: [u8; 32]) -> Self {
        let secret = StaticSecret::from(private);
        let public = PublicKey::from(&secret);
        
        Self {
            public: *public.as_bytes(),
            private,
        }
    }

    /// Get the public key as bytes.
    pub fn public_key(&self) -> &[u8; 32] {
        &self.public
    }

    /// Get the private key as bytes.
    pub fn private_key(&self) -> &[u8; 32] {
        &self.private
    }

    /// Perform X25519 Diffie-Hellman key agreement.
    pub fn dh(&self, their_public: &[u8; 32]) -> [u8; 32] {
        let secret = StaticSecret::from(self.private);
        let their_key = x25519_dalek::PublicKey::from(*their_public);
        let shared = secret.diffie_hellman(&their_key);
        *shared.as_bytes()
    }
}

impl std::fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyPair")
            .field("public", &hex::encode(self.public))
            .field("private", &"[REDACTED]")
            .finish()
    }
}

/// A pre-key for Signal Protocol.
#[derive(Clone)]
pub struct PreKey {
    /// The key pair
    pub key_pair: KeyPair,
    /// Key ID
    pub key_id: u32,
    /// Signature (if signed pre-key)
    pub signature: Option<[u8; 64]>,
}

impl PreKey {
    /// Generate a new pre-key with the given ID.
    pub fn new(key_id: u32) -> Self {
        Self {
            key_pair: KeyPair::generate(),
            key_id,
            signature: None,
        }
    }

    /// Generate a signed pre-key.
    pub fn new_signed(key_id: u32, identity_key: &KeyPair) -> Self {
        let mut pre_key = Self::new(key_id);
        pre_key.signature = Some(identity_key.sign(&pre_key.key_pair));
        pre_key
    }
}

impl KeyPair {
    /// Sign another key pair's public key.
    pub fn sign(&self, key_to_sign: &KeyPair) -> [u8; 64] {
        use ed25519_dalek::{SigningKey, Signer};
        
        // Create message to sign: 0x05 || public_key
        let mut message = [0u8; 33];
        message[0] = 0x05; // DJB type
        message[1..].copy_from_slice(&key_to_sign.public);
        
        // Convert Curve25519 private key to Ed25519 signing key
        // Note: This is a simplified approach; real implementation needs proper conversion
        let signing_key = SigningKey::from_bytes(&self.private);
        let signature = signing_key.sign(&message);
        
        signature.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_pair_generation() {
        let kp = KeyPair::generate();
        assert_ne!(kp.public, [0u8; 32]);
        assert_ne!(kp.private, [0u8; 32]);
    }

    #[test]
    fn test_dh_agreement() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();
        
        let alice_shared = alice.dh(&bob.public);
        let bob_shared = bob.dh(&alice.public);
        
        assert_eq!(alice_shared, bob_shared);
    }

    #[test]
    fn test_pre_key_generation() {
        let pk = PreKey::new(1);
        assert_eq!(pk.key_id, 1);
        assert!(pk.signature.is_none());
    }
}
