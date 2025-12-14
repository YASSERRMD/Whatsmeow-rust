use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{config::WhatsmeowConfig, state::SessionState};

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
    pub fn connect(&self) -> Result<String, ClientError> {
        if !self.state.is_registered() {
            return Err(ClientError::NotRegistered);
        }

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

    /// Persist session state to disk in JSON format.
    pub fn store_state(&self, path: impl AsRef<Path>) -> Result<(), ClientError> {
        let serialized = serde_json::to_string_pretty(&self.state)?;
        fs::write(path, serialized)?;
        Ok(())
    }
}
