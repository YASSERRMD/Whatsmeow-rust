use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use uuid::Uuid;
use whatsmeow_rust::{ScaffoldClientError as ClientError, MessageStatus, SessionState, WhatsmeowClient, WhatsmeowConfig};

/// Reference CLI demonstrating the Whatsmeow Rust scaffolding.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to the JSON session file.
    #[arg(long, default_value = "./data/session.json")]
    state_file: PathBuf,

    /// Override the user agent advertised by the client.
    #[arg(long)]
    user_agent: Option<String>,

    /// Choose a command to run.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Register a device identifier (JID).
    Register { jid: String },
    /// Attempt a connection using the configured session.
    Connect,
    /// Perform a simulated network handshake with the upstream endpoint.
    BootstrapNetwork { endpoint: Option<String> },
    /// Disconnect from the simulated session while keeping local state.
    Disconnect,
    /// Send a message to a known contact while connected.
    SendMessage { to: String, message: String },
    /// Generate a mock pairing code for linking a device.
    RequestPairingCode,
    /// Generate a QR login token to mirror native pairing.
    GenerateQr,
    /// Verify a previously generated QR token.
    VerifyQr { token: String },
    /// Simulate receipt of a message while connected.
    ReceiveMessage { from: String, message: String },
    /// Mark an outgoing message as delivered.
    MarkDelivered { id: String },
    /// Mark an outgoing message as read.
    MarkRead { id: String },
    /// List known contacts.
    ListContacts,
    /// List stored message history.
    ListMessages,
    /// List the recorded lifecycle events.
    ListEvents,
    /// Decrypt an outgoing message by id.
    DecryptMessage { id: String },
    /// Print the current configuration.
    ShowConfig,
    /// Download media to the configured media directory.
    DownloadMedia { url: String, output: Option<String> },
    /// List recorded media downloads.
    ListMedia,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut config = WhatsmeowConfig::default();

    if let Some(agent) = cli.user_agent {
        config = config.with_user_agent(agent);
    }

    let state_dir = cli
        .state_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(state_dir)?;

    let mut client = WhatsmeowClient::new(config, load_state(&cli.state_file));

    match cli.command {
        Commands::Register { jid } => {
            client.register_device(&jid);
            persist_state(&client, &cli.state_file)?;
            println!("Registered device: {jid}");
        }
        Commands::Connect => match client.connect() {
            Ok(summary) => {
                println!("{summary}");
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(err) => return Err(err.into()),
        },
        Commands::BootstrapNetwork { endpoint } => match client.bootstrap_network(endpoint) {
            Ok(network) => {
                println!(
                    "Handshaked with {} (latency {:?} ms, status {:?}, error {:?})",
                    network.endpoint, network.latency_ms, network.status_code, network.error
                );
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(err) => return Err(err.into()),
        },
        Commands::Disconnect => match client.disconnect() {
            Ok(_) => {
                println!("Disconnected.");
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(err) => return Err(err.into()),
        },
        Commands::SendMessage { to, message } => match client.send_message(&to, &message) {
            Ok(record) => {
                println!(
                    "Sent to {} at {} (id {}, status {:?}): {}",
                    record.to, record.sent_at, record.id, record.status, record.body
                );
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(ClientError::NotConnected) => {
                eprintln!("Device not connected. Run the connect command first.");
            }
            Err(err) => return Err(err.into()),
        },
        Commands::RequestPairingCode => match client.request_pairing_code() {
            Ok(code) => {
                println!("Pairing code (valid 5m): {code}");
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(ClientError::PairingCodeExists) => {
                eprintln!(
                    "Pairing code already issued. Clear state or reuse it before requesting a new one."
                );
            }
            Err(err) => return Err(err.into()),
        },
        Commands::GenerateQr => match client.generate_qr_login() {
            Ok(login) => {
                println!("QR token {} (expires {})", login.token, login.expires_at);
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(err) => return Err(err.into()),
        },
        Commands::VerifyQr { token } => match client.verify_qr_login(&token) {
            Ok(login) => {
                println!("Verified QR token {} at {}", login.token, login.issued_at);
                persist_state(&client, &cli.state_file)?;
            }
            Err(ClientError::QrLoginMissing) => {
                eprintln!("Generate a QR token first using generate-qr.");
            }
            Err(ClientError::QrLoginExpired) => {
                eprintln!("QR token expired. Generate a fresh code with generate-qr.");
            }
            Err(ClientError::QrLoginMismatch) => {
                eprintln!("Provided token does not match the active QR login.");
            }
            Err(ClientError::NotRegistered) => {
                eprintln!("Device not registered. Run the register command first.");
            }
            Err(err) => return Err(err.into()),
        },
        Commands::ReceiveMessage { from, message } => {
            match client.simulate_incoming_message(&from, &message) {
                Ok(record) => {
                    println!(
                        "Received from {} at {} (id {}): {}",
                        record.from, record.received_at, record.id, record.body
                    );
                    persist_state(&client, &cli.state_file)?;
                }
                Err(ClientError::NotRegistered) => {
                    eprintln!("Device not registered. Run the register command first.");
                }
                Err(ClientError::NotConnected) => {
                    eprintln!("Device not connected. Run the connect command first.");
                }
                Err(err) => return Err(err.into()),
            }
        }
        Commands::MarkDelivered { id } => match parse_uuid(&id) {
            Ok(uuid) => match client.mark_message_status(uuid, MessageStatus::Delivered) {
                Ok(record) => {
                    println!(
                        "Marked message {} as {:?} for {}",
                        record.id, record.status, record.to
                    );
                    persist_state(&client, &cli.state_file)?;
                }
                Err(ClientError::NotRegistered) => {
                    eprintln!("Device not registered. Run the register command first.");
                }
                Err(ClientError::NotConnected) => {
                    eprintln!("Device not connected. Run the connect command first.");
                }
                Err(ClientError::MessageNotFound(_)) => {
                    eprintln!("No outgoing message found for id {id}.");
                }
                Err(err) => return Err(err.into()),
            },
            Err(err) => eprintln!("Invalid message id: {err}"),
        },
        Commands::MarkRead { id } => match parse_uuid(&id) {
            Ok(uuid) => match client.mark_message_status(uuid, MessageStatus::Read) {
                Ok(record) => {
                    println!(
                        "Marked message {} as {:?} for {}",
                        record.id, record.status, record.to
                    );
                    persist_state(&client, &cli.state_file)?;
                }
                Err(ClientError::NotRegistered) => {
                    eprintln!("Device not registered. Run the register command first.");
                }
                Err(ClientError::NotConnected) => {
                    eprintln!("Device not connected. Run the connect command first.");
                }
                Err(ClientError::MessageNotFound(_)) => {
                    eprintln!("No outgoing message found for id {id}.");
                }
                Err(err) => return Err(err.into()),
            },
            Err(err) => eprintln!("Invalid message id: {err}"),
        },
        Commands::ListContacts => {
            if client.state.contacts.is_empty() {
                println!("No contacts stored.");
            } else {
                for contact in &client.state.contacts {
                    println!("{} ({})", contact.display_name, contact.jid);
                }
            }
        }
        Commands::ListMessages => {
            if client.state.outgoing_messages.is_empty()
                && client.state.incoming_messages.is_empty()
            {
                println!("No messages have been recorded.");
            } else {
                for msg in &client.state.outgoing_messages {
                    println!(
                        "[sent {}] to {} (id {}, status {:?}): {}",
                        msg.sent_at, msg.to, msg.id, msg.status, msg.body
                    );
                }
                for msg in &client.state.incoming_messages {
                    println!(
                        "[recv {}] from {} (id {}): {}",
                        msg.received_at, msg.from, msg.id, msg.body
                    );
                }
            }
        }
        Commands::ListEvents => {
            if client.state.events.is_empty() {
                println!("No events recorded.");
            } else {
                for event in &client.state.events {
                    println!("[{0}] {1:?}", event.at, event.kind);
                }
            }
        }
        Commands::DecryptMessage { id } => match parse_uuid(&id) {
            Ok(uuid) => match client.decrypt_message_body(uuid) {
                Ok(plaintext) => println!("Plaintext: {plaintext}"),
                Err(ClientError::MessageNotFound(_)) => {
                    eprintln!("No outgoing message found for id {id}.");
                }
                Err(err) => return Err(err.into()),
            },
            Err(err) => eprintln!("Invalid message id: {err}"),
        },
        Commands::ShowConfig => {
            println!("Config: {:?}", client.config);
            println!("Session: {:?}", client.state);
        }
        Commands::DownloadMedia { url, output } => {
            match client.download_media(&url, output.as_deref()) {
                Ok(item) => {
                    println!(
                        "Downloaded {} bytes from {} to {} (id {})",
                        item.bytes, item.source, item.file_path, item.id
                    );
                    persist_state(&client, &cli.state_file)?;
                }
                Err(ClientError::NotRegistered) => {
                    eprintln!("Device not registered. Run the register command first.");
                }
                Err(ClientError::NotConnected) => {
                    eprintln!("Device not connected. Run the connect command first.");
                }
                Err(err) => return Err(err.into()),
            }
        }
        Commands::ListMedia => {
            if client.state.media.is_empty() {
                println!("No media downloaded yet.");
            } else {
                for item in &client.state.media {
                    println!(
                        "[{0}] {1} bytes from {2} -> {3}",
                        item.downloaded_at, item.bytes, item.source, item.file_path
                    );
                }
            }
        }
    }

    Ok(())
}

fn load_state(path: &PathBuf) -> SessionState {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => SessionState::with_device_name("whatsmeow-rust"),
    }
}

fn persist_state(client: &WhatsmeowClient, path: &PathBuf) -> Result<(), ClientError> {
    client.store_state(path)
}

fn parse_uuid(id: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(id)
}
