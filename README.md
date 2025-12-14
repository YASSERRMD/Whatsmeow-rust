# whatsmeow-rust

This repository provides a small, documented Rust scaffold inspired by the
[Whatsmeow](https://github.com/tulir/whatsmeow) Go client. It is **not** a
feature-complete port; instead, it offers a starting point for experimenting
with configuration, session management, and a basic CLI workflow.

## Features
- Serializable `WhatsmeowConfig` and `SessionState` structures.
- A `WhatsmeowClient` façade with registration, connection/disconnection
  simulation, message logging, inbound message recording, pairing code
  issuance, event tracking, and session persistence to JSON.
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
cargo run -- receive-message --from 12345@s.whatsapp.net --message "Hi back!"
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
- `outgoing_messages`: log of sent messages, timestamped for traceability.
- `incoming_messages`: log of received messages recorded via the CLI.
- `connected` and `last_connected`: simple connection status markers.
- `pairing_code`: placeholder for QR/pairing-based login flows.
- `events`: ordered log of lifecycle events.

## Notes
- The upstream library includes networking, QR login, message encryption, and
  persistence layers that are **not** reproduced here.
- This scaffold focuses on clear types and extension points so you can iterate
  toward a fuller implementation.
