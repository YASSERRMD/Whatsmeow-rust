# whatsmeow-rust

A Rust implementation of the WhatsApp Web protocol, inspired by and ported from [tulir/whatsmeow](https://github.com/tulir/whatsmeow).

## Overview

This library provides the building blocks for connecting to WhatsApp servers using the real protocol:

- **Binary Protocol**: Node-based binary XML encoding/decoding with 236-token dictionary compression
- **Cryptography**: Curve25519 key pairs, AES-256-GCM encryption, HKDF key derivation, Noise Protocol XX handshake
- **Transport**: WebSocket connection with Noise Protocol encryption
- **Storage**: Device state, session management, contact storage with pluggable backends
- **Features**: QR code pairing, message building/parsing, presence, typing indicators

## Getting Started

### Prerequisites
- Rust 1.70+ (2021 edition)
- Tokio async runtime

### Build and Test
```bash
cargo build
cargo test
```

### Run the Echo Bot Example
```bash
cargo run --example echo_bot
```

## Module Structure

```
src/
├── types/       # JID, MessageID, events
├── binary/      # Node, token dictionary, encoder/decoder
├── crypto/      # KeyPair, HKDF, Cipher, NoiseHandshake
├── socket/      # NoiseSocket WebSocket transport
├── store/       # Device, store traits, MemoryStore
└── protocol/    # Client, QRPairing, message builders
```

## Usage

```rust
use whatsmeow_rust::{
    Client, Device, JID,
    protocol::{build_text_message, QRPairing},
    types::servers,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize device with fresh keys
    let mut device = Device::new();
    device.initialize();

    // Generate QR code for pairing
    let pairing = QRPairing::new(device.clone());
    if let Some(qr_data) = pairing.current_code() {
        let qr_ascii = QRPairing::render_qr_ascii(qr_data)?;
        println!("{}", qr_ascii);
    }

    // Build a message
    let to = JID::new("1234567890", servers::DEFAULT_USER);
    let msg = build_text_message(&to, "Hello from Rust!", None);

    // Create client and connect
    let mut client = Client::new();
    client.connect().await?;
    
    // Send message
    client.send_message(to, "Hello!").await?;

    Ok(())
}
```

## Features

| Feature | Description |
|---------|-------------|
| QR Pairing | Generate QR codes for device linking |
| Text Messages | Build and parse text messages |
| Media Messages | Image, video, audio, document support |
| Presence | Online/offline status |
| Typing Indicators | Composing/paused states |
| Read Receipts | Delivery and read confirmations |
| Binary Encoding | Efficient WhatsApp binary XML format |
| Noise Protocol | Secure handshake with WhatsApp servers |

## Architecture

The library follows the same architecture as whatsmeow:

1. **Types** (`types/`): Core data types like JID, events
2. **Binary** (`binary/`): WhatsApp's binary XML protocol
3. **Crypto** (`crypto/`): Signal Protocol primitives
4. **Socket** (`socket/`): Encrypted WebSocket transport
5. **Store** (`store/`): Persistent storage for keys and sessions
6. **Protocol** (`protocol/`): High-level client API

## Status

This is a work-in-progress port of whatsmeow. Current capabilities:
- ✅ Binary protocol encoding/decoding
- ✅ Curve25519/AES-GCM/HKDF crypto
- ✅ Noise Protocol handshake
- ✅ Device initialization and key generation
- ✅ QR code generation
- ✅ Message building and parsing
- 🚧 Full WebSocket connection (needs real server testing)
- 🚧 Signal Protocol sessions
- 🚧 Group messaging

## License

MPL-2.0 (same as whatsmeow)

## Credits

Ported from [tulir/whatsmeow](https://github.com/tulir/whatsmeow) - A Go library for the WhatsApp Web multidevice API.
