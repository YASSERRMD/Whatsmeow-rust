use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use whatsmeow_rust::{ClientError, SessionState, WhatsmeowClient, WhatsmeowConfig};

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
    /// Disconnect from the simulated session while keeping local state.
    Disconnect,
    /// Send a message to a known contact while connected.
    SendMessage { to: String, message: String },
    /// Print the current configuration.
    ShowConfig,
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
                    "Sent to {} at {}: {}",
                    record.to, record.sent_at, record.body
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
        Commands::ShowConfig => {
            println!("Config: {:?}", client.config);
            println!("Session: {:?}", client.state);
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
