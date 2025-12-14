# whatsmeow-rust

This repository provides a small, documented Rust scaffold inspired by the
[Whatsmeow](https://github.com/tulir/whatsmeow) Go client. It is **not** a
feature-complete port; instead, it offers a starting point for experimenting
with configuration, session management, and a basic CLI workflow.

## Features
- Serializable `WhatsmeowConfig` and `SessionState` structures.
- A `WhatsmeowClient` façade with registration, connection/disconnection
  simulation, message logging, inbound message recording, pairing code
  issuance, delivery/read receipt simulation, event tracking, session
  persistence to JSON, a lightweight networking handshake, QR login token
  issuance/verification, and symmetric message encryption helpers.
- Command-line interface built with `clap` for registering a device, printing
  configuration, connecting/disconnecting, generating pairing codes, sending
  mock messages, recording received messages, and inspecting stored contacts or
  history.

## Usage

```bash
cargo run -- --help
cargo run -- register --jid 12345@s.whatsapp.net
cargo run -- connect
cargo run -- send-message --to 12345@s.whatsapp.net --message "Hello from Rust"
cargo run -- request-pairing-code
cargo run -- generate-qr
cargo run -- verify-qr --token <token>
cargo run -- bootstrap-network --endpoint https://chat.whatsmeow.test
cargo run -- receive-message --from 12345@s.whatsapp.net --message "Hi back!"
cargo run -- mark-delivered --id <message-id>
cargo run -- mark-read --id <message-id>
cargo run -- decrypt-message --id <message-id>
cargo run -- list-contacts
cargo run -- list-messages
cargo run -- list-events
cargo run -- disconnect
cargo run -- show-config
```

The commands store state in `./data/session.json` by default. Use
`--state-file` to change the location and `--user-agent` to override the client
identifier.

### Session contents

- `registered_jid` and `encryption_keys`: capture registration output.
- `contacts`: populated as messages are sent to JIDs.
- `outgoing_messages`: log of sent messages, each with a UUID and delivery
  status.
- `incoming_messages`: log of received messages recorded via the CLI, with
  unique IDs for debugging.
- `connected` and `last_connected`: simple connection status markers.
- `pairing_code`: placeholder for QR/pairing-based login flows.
- `events`: ordered log of lifecycle events including networking, QR, and
  encryption milestones.

### Networking, QR, and encryption

- Call `bootstrap-network` after registration to store endpoint and latency
  measurements in the session, mirroring a network handshake.
- Call `generate-qr` followed by `verify-qr --token <token>` to simulate QR
  login token issuance and verification.
- Messages are encrypted with a symmetric XOR+Base64 helper on send; use
  `decrypt-message --id <message-id>` to view the decrypted body locally.

## Notes
- Networking, QR login, encryption, and persistence are implemented as local
  simulations; they mirror the upstream concerns but are not production-grade
  protocol implementations.
- This scaffold focuses on clear types and extension points so you can iterate
  toward a fuller implementation.
