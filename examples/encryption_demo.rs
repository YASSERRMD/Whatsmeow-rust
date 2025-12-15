//! Encryption Demo - Demonstrates message encryption and decryption.
//!
//! This example shows:
//! - Configuring custom encryption secrets
//! - Sending encrypted messages
//! - Viewing stored ciphertext
//! - Decrypting message bodies using the public API
//!
//! Run with: `cargo run --example encryption_demo`

use std::fs;
use whatsmeow_rust::{SessionState, WhatsmeowClient, WhatsmeowConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Whatsmeow-rust Encryption Demo ===\n");

    // Setup with custom encryption secret
    fs::create_dir_all("./examples_data")?;
    
    println!("🔑 Step 1: Configuring encryption...");
    let secret = "my-super-secret-key-for-demo";
    let config = WhatsmeowConfig::default()
        .with_database_path("./examples_data/encryption.db")
        .with_media_path("./examples_data/media")
        .with_encryption_secret(secret);
    
    println!("   Encryption secret: {}...", &secret[..15]);
    println!("   (In production, use a strong, random key!)\n");

    let state = SessionState::with_device_name("encryption-demo");
    let mut client = WhatsmeowClient::new(config, state);

    // Register and connect
    println!("📱 Step 2: Setting up client...");
    client.register_device("encrypt-demo@s.whatsapp.net");
    client.connect()?;
    println!("   Client ready\n");

    // Send multiple encrypted messages
    println!("🔒 Step 3: Sending encrypted messages...\n");
    
    let messages = [
        ("alice@example.com", "Hello Alice! This is a secret message 🤫"),
        ("bob@example.com", "Hey Bob, let's keep this between us! 🔐"),
        ("charlie@example.com", "Top secret information: The answer is 42! 🎲"),
    ];

    let mut message_ids = Vec::new();

    for (recipient, body) in messages {
        match client.send_message(recipient, body) {
            Ok(msg) => {
                println!("   📤 Sent to {}", recipient);
                println!("      Plaintext: \"{}\"", msg.body);
                if let Some(ref cipher) = msg.ciphertext {
                    let preview = if cipher.len() > 50 { &cipher[..50] } else { cipher };
                    println!("      Ciphertext: {}...", preview);
                }
                println!();
                message_ids.push(msg.id);
            }
            Err(e) => println!("   ❌ Error: {:?}", e),
        }
    }

    // Decrypt messages using the public decrypt_message_body API
    println!("🔓 Step 4: Decrypting stored messages...\n");
    
    for id in &message_ids {
        match client.decrypt_message_body(*id) {
            Ok(plaintext) => {
                println!("   Message ID: {}", id);
                println!("   Decrypted: \"{}\"\n", plaintext);
            }
            Err(e) => println!("   ❌ Error decrypting {}: {:?}", id, e),
        }
    }

    // Show how encryption works conceptually
    println!("🧪 Step 5: Understanding the encryption flow...\n");
    
    println!("   When you call send_message(), the library:");
    println!("   1. Creates a message record with the plaintext body");
    println!("   2. Encrypts the body using AES-256-GCM with your secret");
    println!("   3. Stores the ciphertext alongside the message");
    println!("   4. Logs an encryption event");
    println!();
    println!("   The decrypt_message_body() method:");
    println!("   1. Finds the message by ID");
    println!("   2. Retrieves the stored ciphertext");
    println!("   3. Decrypts it using the same secret");
    println!("   4. Returns the original plaintext");

    // Show stored messages with encryption details
    println!("\n📋 Step 6: All stored messages with encryption state:\n");
    println!("   {:<36} | {:<20} | Has Cipher | Body", "ID", "Recipient");
    println!("   {}", "-".repeat(90));
    
    for msg in &client.state.outgoing_messages {
        let has_cipher = if msg.ciphertext.is_some() { "Yes" } else { "No" };
        let body_preview = if msg.body.len() > 30 {
            format!("{}...", &msg.body[..30])
        } else {
            msg.body.clone()
        };
        println!("   {} | {:<20} | {:<10} | {}", msg.id, msg.to, has_cipher, body_preview);
    }

    // Show encryption events
    println!("\n📊 Step 7: Encryption-related events:");
    for event in &client.state.events {
        let event_str = format!("{:?}", event.kind);
        if event_str.contains("Encrypt") {
            println!("   [{}] {:?}", event.at.format("%H:%M:%S"), event.kind);
        }
    }

    // Save state
    let state_path = "./examples_data/encryption_session.json";
    println!("\n💾 Saving session to {}...", state_path);
    client.store_state(state_path)?;

    println!("\n=== Encryption Demo Complete! ===");
    println!("Check {} to see stored ciphertext\n", state_path);

    Ok(())
}
