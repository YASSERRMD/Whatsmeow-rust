//! QR code pairing for WhatsApp authentication.
//!
//! Handles QR code generation and pairing flow for linking devices.

use tokio::sync::mpsc;
use std::time::Duration;
use qrcode::{QrCode, render::unicode};

use crate::crypto::KeyPair;
use crate::store::Device;

/// QR channel event types.
#[derive(Debug, Clone)]
pub enum QREvent {
    /// New QR code to display
    Code {
        /// The QR code data string
        data: String,
        /// Timeout before next code
        timeout: Duration,
    },
    /// Pairing successful
    Success,
    /// Pairing timed out
    Timeout,
    /// Error during pairing
    Error(String),
    /// Client outdated
    ClientOutdated,
}

/// QR channel for receiving pairing events.
pub type QRChannel = mpsc::Receiver<QREvent>;

/// QR pairing state.
pub struct QRPairing {
    /// Device being paired
    device: Device,
    /// QR codes to emit
    codes: Vec<String>,
    /// Current code index
    current_index: usize,
    /// Whether pairing is complete
    complete: bool,
}

impl QRPairing {
    /// Create a new QR pairing session.
    pub fn new(device: Device) -> Self {
        // Generate QR code data
        // Format: ref,publicKey,advSecretKey,serverRef
        let codes = Self::generate_codes(&device);
        
        Self {
            device,
            codes,
            current_index: 0,
            complete: false,
        }
    }

    /// Generate QR codes for pairing.
    /// Format: ref,noisePublicKey,identityPublicKey,advSecretKey
    fn generate_codes(device: &Device) -> Vec<String> {
        let noise_pub = device.noise_key.as_ref()
            .map(|k| base64::encode(&k.public))
            .unwrap_or_default();
        
        let identity_pub = device.identity_key.as_ref()
            .map(|k| base64::encode(&k.public))
            .unwrap_or_default();

        let adv_secret = device.adv_secret_key.as_ref()
            .map(|k| base64::encode(k))
            .unwrap_or_default();

        // Generate multiple refs for timeout rotation (6 codes with 20s timeout each)
        (0..6).map(|_| {
            let ref_id = format!("{:X}", rand::random::<u64>());
            format!("{},{},{},{}", ref_id, noise_pub, identity_pub, adv_secret)
        }).collect()
    }

    /// Get the current QR code data.
    pub fn current_code(&self) -> Option<&str> {
        self.codes.get(self.current_index).map(|s| s.as_str())
    }

    /// Advance to the next QR code.
    pub fn next_code(&mut self) -> Option<&str> {
        if self.current_index + 1 < self.codes.len() {
            self.current_index += 1;
            self.codes.get(self.current_index).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Get timeout for current code.
    pub fn current_timeout(&self) -> Duration {
        if self.current_index == 0 {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(20)
        }
    }

    /// Mark pairing as complete.
    pub fn mark_complete(&mut self) {
        self.complete = true;
    }

    /// Check if pairing is complete.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Render QR code as ASCII for terminal display.
    pub fn render_qr_ascii(data: &str) -> Result<String, QRError> {
        let code = QrCode::new(data.as_bytes())
            .map_err(|e| QRError::GenerationFailed(e.to_string()))?;
        
        let image = code.render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        
        Ok(image)
    }
}

/// QR code errors.
#[derive(Debug, Clone)]
pub enum QRError {
    GenerationFailed(String),
    PairingFailed(String),
    Timeout,
}

impl std::fmt::Display for QRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QRError::GenerationFailed(e) => write!(f, "QR generation failed: {}", e),
            QRError::PairingFailed(e) => write!(f, "pairing failed: {}", e),
            QRError::Timeout => write!(f, "pairing timed out"),
        }
    }
}

impl std::error::Error for QRError {}

/// Start QR pairing and return a channel for events.
pub fn start_qr_pairing(device: Device) -> (QRPairing, mpsc::Sender<QREvent>) {
    let (tx, _rx) = mpsc::channel(16);
    let pairing = QRPairing::new(device);
    (pairing, tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_pairing_creation() {
        let mut device = Device::new();
        device.initialize();
        
        let pairing = QRPairing::new(device);
        assert!(!pairing.is_complete());
        assert!(pairing.current_code().is_some());
    }

    #[test]
    fn test_qr_code_rotation() {
        let mut device = Device::new();
        device.initialize();
        
        let mut pairing = QRPairing::new(device);
        let first = pairing.current_code().unwrap().to_string();
        
        let second = pairing.next_code();
        assert!(second.is_some());
        assert_ne!(first, second.unwrap());
    }

    #[test]
    fn test_qr_ascii_render() {
        let result = QRPairing::render_qr_ascii("test data");
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
