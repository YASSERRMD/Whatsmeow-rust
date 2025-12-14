use serde::{Deserialize, Serialize};

/// Base configuration used by the Whatsmeow client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WhatsmeowConfig {
    /// Path to the persistent database used for contacts and message state.
    pub database_path: String,
    /// Directory containing media downloads and uploads.
    pub media_path: String,
    /// Identifier sent in the client user agent string.
    pub user_agent: String,
}

impl Default for WhatsmeowConfig {
    fn default() -> Self {
        Self {
            database_path: "./data/whatsmeow.db".into(),
            media_path: "./data/media".into(),
            user_agent: "whatsmeow-rust/0.1".into(),
        }
    }
}

impl WhatsmeowConfig {
    /// Override the database path.
    pub fn with_database_path(mut self, path: impl Into<String>) -> Self {
        self.database_path = path.into();
        self
    }

    /// Override the media directory path.
    pub fn with_media_path(mut self, path: impl Into<String>) -> Self {
        self.media_path = path.into();
        self
    }

    /// Override the user agent string.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }
}
