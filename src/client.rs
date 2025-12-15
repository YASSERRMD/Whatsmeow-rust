use std::{fs, path::Path};

use aes_gcm::{Aes256Gcm, Nonce, aead::Aead, aead::KeyInit};
use base64::{Engine as _, engine::general_purpose};
use chrono::{Duration, Utc};
use rand::{Rng, RngCore, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    config::WhatsmeowConfig,
    state::{
        EventKind, IncomingMessage, MessageStatus, NetworkState, OutgoingMessage, QrLogin,
        SessionEvent, SessionState,
    },
};

/// High-level facade that mimics a Whatsmeow client lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsmeowClient {
    pub config: WhatsmeowConfig,
    pub state: SessionState,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("device is not registered; call `register_device` first")]
    NotRegistered,
    #[error("device is not connected; call `connect` first")]
    NotConnected,
    #[error("pairing code already exists; reuse or clear it before requesting a new one")]
    PairingCodeExists,
    #[error("no outgoing message found for id {0}")]
    MessageNotFound(Uuid),
    #[error("qr login not generated yet")]
    QrLoginMissing,
    #[error("qr login token mismatch")]
    QrLoginMismatch,
    #[error("qr login token expired")]
    QrLoginExpired,
    #[error("encryption failed: {0}")]
    EncryptionFailure(String),
    #[error("failed to serialize session: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("failed to persist session: {0}")]
    Io(#[from] std::io::Error),
}

impl WhatsmeowClient {
    /// Instantiate a client with custom configuration and state.
    pub fn new(config: WhatsmeowConfig, state: SessionState) -> Self {
        Self { config, state }
    }

    /// Produce a human-readable handshake summary.
    pub fn connect(&mut self) -> Result<String, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        self.state.mark_connected();

        Ok(format!(
            "Connecting as {} with user-agent {} (media at {})",
            self.state
                .registered_jid
                .as_deref()
                .unwrap_or("unregistered"),
            self.config.user_agent,
            self.config.media_path
        ))
    }

    /// Set the registered JID within the session.
    pub fn register_device(&mut self, jid: impl Into<String>) {
        self.state.register(jid);
    }

    /// Generate a mock pairing code to mimic QR/pairing flows.
    pub fn request_pairing_code(&mut self) -> Result<String, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        if let Some(existing) = &self.state.pairing_code {
            if existing.expires_at > Utc::now() {
                return Err(ClientError::PairingCodeExists);
            }

            self.state.pairing_code = None;
        }

        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let expires_at = Utc::now() + Duration::minutes(5);
        self.state.set_pairing_code(code.clone(), expires_at);
        Ok(code)
    }

    /// Perform a simulated network handshake and record metadata.
    pub fn bootstrap_network(
        &mut self,
        endpoint: Option<String>,
    ) -> Result<NetworkState, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        let endpoint_to_use = endpoint.unwrap_or_else(|| self.config.network_endpoint.clone());
        let start = std::time::Instant::now();
        let response = ureq::get(&endpoint_to_use).call();
        let elapsed_ms = start.elapsed().as_millis();

        let (status_code, error) = match response {
            Ok(resp) => (Some(resp.status()), None),
            Err(ureq::Error::Status(code, _)) => (Some(code), Some("unexpected status".into())),
            Err(err) => (None, Some(err.to_string())),
        };

        self.state.mark_network_handshake(
            endpoint_to_use.clone(),
            Some(elapsed_ms),
            status_code,
            error,
        );
        Ok(self.state.network.clone())
    }

    /// Create a QR login token and return a printable representation.
    pub fn generate_qr_login(&mut self) -> Result<QrLogin, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();
        let expires_at = Utc::now() + Duration::minutes(10);
        self.state.set_qr_login(token.clone(), expires_at);
        Ok(self.state.qr_login.clone().expect("qr login stored"))
    }

    /// Validate a previously generated QR token and mark it verified.
    pub fn verify_qr_login(&mut self, token: &str) -> Result<QrLogin, ClientError> {
        let login = self
            .state
            .qr_login
            .clone()
            .ok_or(ClientError::QrLoginMissing)?;

        if login.expires_at < Utc::now() {
            // Clear the expired token so callers can immediately generate a new one.
            self.state.qr_login = None;
            return Err(ClientError::QrLoginExpired);
        }

        if login.token != token {
            return Err(ClientError::QrLoginMismatch);
        }
        let verified = self
            .state
            .verify_qr_login()
            .expect("qr login should exist after check");
        Ok(verified)
    }

    /// Disconnect the client while keeping local state.
    pub fn disconnect(&mut self) -> Result<(), ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        self.state.mark_disconnected();
        Ok(())
    }

    /// Add a contact and record an outgoing message in the session log.
    pub fn send_message(
        &mut self,
        to: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<OutgoingMessage, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        if !self.state.is_connected() {
            return Err(ClientError::NotConnected);
        }

        let to = to.into();
        self.state.upsert_contact(&to, &to);
        let mut record = self.state.record_message(to, body);
        let ciphertext = self.encrypt_body(&record.body)?;
        record.ciphertext = Some(ciphertext.clone());
        self.state
            .events
            .push(SessionEvent::new(EventKind::MessageEncrypted(record.id)));
        if let Some(entry) = self
            .state
            .outgoing_messages
            .iter_mut()
            .find(|msg| msg.id == record.id)
        {
            entry.ciphertext = Some(ciphertext);
        }
        Ok(record)
    }

    /// Record an incoming message to demonstrate receive flows.
    pub fn simulate_incoming_message(
        &mut self,
        from: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<IncomingMessage, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        if !self.state.is_connected() {
            return Err(ClientError::NotConnected);
        }

        let from = from.into();
        self.state.upsert_contact(&from, &from);
        Ok(self.state.record_incoming_message(from, body))
    }

    /// Update delivery status for an outgoing message to mimic delivery receipts.
    pub fn mark_message_status(
        &mut self,
        id: Uuid,
        status: MessageStatus,
    ) -> Result<OutgoingMessage, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

        if !self.state.is_connected() {
            return Err(ClientError::NotConnected);
        }

        let updated = self
            .state
            .mark_outgoing_status(id, status)
            .cloned()
            .ok_or(ClientError::MessageNotFound(id))?;

        Ok(updated)
    }

    /// Persist session state to disk in JSON format.
    pub fn store_state(&self, path: impl AsRef<Path>) -> Result<(), ClientError> {
        let serialized = serde_json::to_string_pretty(&self.state)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Decrypt an outgoing message payload for inspection.
    pub fn decrypt_message_body(&self, id: Uuid) -> Result<String, ClientError> {
        let message = self
            .state
            .outgoing_messages
            .iter()
            .find(|msg| msg.id == id)
            .ok_or(ClientError::MessageNotFound(id))?;
        let ciphertext = message
            .ciphertext
            .as_ref()
            .ok_or_else(|| ClientError::EncryptionFailure("ciphertext missing".into()))?;
        self.decrypt_body(ciphertext)
    }

    fn encrypt_body(&self, body: &str) -> Result<String, ClientError> {
        let cipher = self.cipher()?;
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, body.as_bytes())
            .map_err(|err| ClientError::EncryptionFailure(err.to_string()))?;

        let mut payload = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
        Ok(general_purpose::STANDARD.encode(payload))
    }

    fn decrypt_body(&self, ciphertext: &str) -> Result<String, ClientError> {
        let cipher = self.cipher()?;
        let decoded = general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(|err| ClientError::EncryptionFailure(err.to_string()))?;

        if decoded.len() < 12 {
            return Err(ClientError::EncryptionFailure(
                "ciphertext missing nonce".into(),
            ));
        }

        let (nonce_bytes, body) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, body)
            .map_err(|err| ClientError::EncryptionFailure(err.to_string()))?;

        String::from_utf8(plaintext).map_err(|err| ClientError::EncryptionFailure(err.to_string()))
    }

    fn cipher(&self) -> Result<Aes256Gcm, ClientError> {
        if self.config.encryption_secret.is_empty() {
            return Err(ClientError::EncryptionFailure("empty secret".into()));
        }

        let mut hasher = Sha256::new();
        hasher.update(self.config.encryption_secret.as_bytes());
        let digest = hasher.finalize();
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&digest);
        Ok(Aes256Gcm::new(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expired_pairing_code_allows_regeneration() {
        let mut client = WhatsmeowClient::new(WhatsmeowConfig::default(), SessionState::default());
        client.register_device("123@s.whatsapp.net");
        client.state.set_pairing_code("EXPIRED", Utc::now() - Duration::minutes(1));

        let code = client.request_pairing_code().expect("should re-issue code");

        assert_ne!(code, "EXPIRED");
        assert!(client.state.pairing_code.as_ref().is_some());
    }

    #[test]
    fn expired_qr_tokens_are_rejected() {
        let mut client = WhatsmeowClient::new(WhatsmeowConfig::default(), SessionState::default());
        client.register_device("123@s.whatsapp.net");
        client
            .state
            .set_qr_login("expired", Utc::now() - Duration::minutes(5));

        let result = client.verify_qr_login("expired");

        assert!(matches!(result, Err(ClientError::QrLoginExpired)));
        assert!(client.state.qr_login.is_none());
    }
}
