# whatsmeow-rust

A small, documented Rust scaffold inspired by the [Whatsmeow](https://github.com/tulir/whatsmeow) Go client. It mirrors the shape of a messaging client—configuration, session persistence, QR flows, encryption helpers, networking probes, and media downloads—while keeping the implementation deliberately lightweight for local experimentation. This project is **not** a production-ready WhatsApp client.

## Getting started

### Prerequisites
- Rust 1.76+ (stable toolchain recommended)
- Network access for `ureq` when invoking the networking or media commands
- A writable working directory (default state is written under `./data/`)

### Build and test
```bash
cargo test
```

### Run the CLI
The CLI ships with a set of subcommands that exercise the client. By default, state is read from and written to `./data/session.json`. You can point to a different file with `--state-file` and override the advertised user agent with `--user-agent`.

```bash
cargo run -- --help
cargo run -- register --jid 12345@s.whatsapp.net
cargo run -- connect
cargo run -- bootstrap-network --endpoint https://chat.whatsmeow.test
cargo run -- send-message --to 12345@s.whatsapp.net --message "Hello from Rust"
cargo run -- receive-message --from 12345@s.whatsapp.net --message "Hi back!"
cargo run -- mark-delivered --id <message-id>
cargo run -- mark-read --id <message-id>
cargo run -- decrypt-message --id <message-id>
cargo run -- request-pairing-code
cargo run -- generate-qr
cargo run -- verify-qr --token <token>
cargo run -- download-media --url https://example.com/media.jpg
cargo run -- download-media --url https://example.com/media.jpg --output hero.jpg
cargo run -- list-media
cargo run -- list-contacts
cargo run -- list-messages
cargo run -- list-events
cargo run -- show-config
cargo run -- disconnect
```

## Capabilities
The scaffold keeps the domain surface area of the upstream project while remaining intentionally simplified:
- **Configuration**: `WhatsmeowConfig` captures database, media, user agent, network endpoint, and encryption secret settings with builder-style overrides.
- **Registration and session persistence**: Device JID registration seeds encryption key placeholders and persists state to JSON alongside a device name and contact list.
- **Connection lifecycle**: `connect` / `disconnect` toggle connection flags and emit ordered session events.
- **Messaging simulation**: Outgoing and incoming messages are timestamped, tracked with UUIDs, and logged to the event timeline. Delivery/read receipts update stored status.
- **QR and pairing flows**: Request expiring pairing codes, generate 10-minute QR login tokens, and verify them with mismatch/expiry handling.
- **Networking probe**: `bootstrap-network` performs a real HTTP GET to the configured endpoint, storing latency, status code, and any error string for inspection.
- **Encryption helpers**: Outbound messages are encrypted with AES-256-GCM using a key derived from the configured `encryption_secret`; ciphertext and nonce are stored to allow local decryption.
- **Media downloads**: Media files are fetched over HTTP when connected and registered, saved under the configured media directory, and tracked in session state with byte counts and event entries.
- **Events**: Every significant transition (registration, network handshake, QR generation/verification, message sends/receives, media downloads) is appended to a chronological event log.

## Session layout
The session JSON (default `./data/session.json`) mirrors the structures in `src/state.rs`:
- `registered_jid`, `encryption_keys`, and `device_name`
- `contacts`: Known JIDs with display names, updated when messages are sent
- `outgoing_messages` / `incoming_messages`: Message bodies, timestamps, statuses, UUIDs, and (for outgoing) optional ciphertext
- `connected` and `last_connected`: Simple connection markers
- `network`: Last handshake endpoint, latency, status code, error text, and timestamp
- `pairing_code` and `qr_login`: Expiring codes for pairing and QR-based login flows
- `media`: Downloaded file metadata (source URL, local path, byte size, timestamp)
- `events`: Ordered lifecycle history referencing the items above

## Configuration notes
`WhatsmeowConfig::default()` writes artifacts under `./data/` (`whatsmeow.db` and `media/`) and advertises the `whatsmeow-rust/0.1` user agent. Override fields with the provided `with_*` helpers when embedding the crate, or use `--user-agent` in the CLI for quick testing.

## Limitations
This repository is a teaching scaffold. Networking, QR, encryption, and media handling are best-effort demonstrations for local use—they do not implement the WhatsApp protocol and should not be treated as secure or production-ready.
