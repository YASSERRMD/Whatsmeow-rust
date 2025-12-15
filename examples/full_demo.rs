//! Full Demo - Complete demonstration of all Whatsmeow-rust library features.
//!
//! This example provides a comprehensive walkthrough of:
//! - Configuration and initialization
//! - Device registration
//! - Connection lifecycle
//! - Messaging (send/receive/status)
//! - Encryption/decryption
//! - QR and pairing flows
//! - Media downloads
//! - Network bootstrap
//! - Event tracking
//! - State persistence
//!
//! Run with: `cargo run --example full_demo`

use std::fs;
use whatsmeow_rust::{MessageStatus, SessionState, WhatsmeowClient, WhatsmeowConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        Whatsmeow-rust Complete Library Demonstration         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create output directory
    let data_dir = "./examples_data/full_demo";
    fs::create_dir_all(format!("{}/media", data_dir))?;

    // ═══════════════════════════════════════════════════════════════════════
    // PART 1: CONFIGURATION
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 1: CONFIGURATION\n");
    
    let config = WhatsmeowConfig::default()
        .with_database_path(format!("{}/whatsmeow.db", data_dir))
        .with_media_path(format!("{}/media", data_dir))
        .with_user_agent("whatsmeow-full-demo/1.0")
        .with_network_endpoint("https://httpbin.org/get")
        .with_encryption_secret("demo-encryption-secret-32bytes!!");

    println!("   📝 Configuration created:");
    println!("      Database: {}", config.database_path);
    println!("      Media: {}", config.media_path);
    println!("      User Agent: {}", config.user_agent);
    println!("      Network Endpoint: {}", config.network_endpoint);
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 2: CLIENT INITIALIZATION
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 2: CLIENT INITIALIZATION\n");

    let state_path = format!("{}/session.json", data_dir);
    let state = match fs::read_to_string(&state_path) {
        Ok(contents) => {
            println!("   📂 Loading existing session...");
            serde_json::from_str(&contents).unwrap_or_else(|_| {
                SessionState::with_device_name("full-demo-device")
            })
        }
        Err(_) => {
            println!("   📂 Creating new session...");
            SessionState::with_device_name("full-demo-device")
        }
    };

    let mut client = WhatsmeowClient::new(config, state);
    println!("   ✅ Client initialized\n");

    // ═══════════════════════════════════════════════════════════════════════
    // PART 3: REGISTRATION & CONNECTION
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 3: REGISTRATION & CONNECTION\n");

    // Register if needed
    if !client.state.is_registered() {
        let jid = "full-demo-user@s.whatsapp.net";
        client.register_device(jid);
        println!("   📱 Device registered: {}", jid);
    } else {
        println!("   📱 Already registered as: {:?}", client.state.registered_jid);
    }

    // Connect
    match client.connect() {
        Ok(summary) => println!("   🔌 {}", summary),
        Err(e) => println!("   ❌ Connection error: {:?}", e),
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 4: NETWORK BOOTSTRAP
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 4: NETWORK BOOTSTRAP\n");

    match client.bootstrap_network(None::<String>) {
        Ok(network) => {
            println!("   🌐 Network handshake complete:");
            println!("      Endpoint: {}", network.endpoint);
            println!("      Latency: {:?} ms", network.latency_ms);
            println!("      Status: {:?}", network.status_code);
            if let Some(ref err) = network.error {
                println!("      Error: {}", err);
            }
        }
        Err(e) => println!("   ⚠️ Network error (expected in demo): {:?}", e),
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 5: MESSAGING
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 5: MESSAGING\n");

    // Send messages
    let recipients = [
        ("alice@demo.com", "Hello Alice! 👋"),
        ("bob@demo.com", "Hey Bob, how's it going? 🚀"),
        ("charlie@demo.com", "Charlie, check this out! 🎉"),
    ];

    for (recipient, body) in recipients {
        match client.send_message(recipient, body) {
            Ok(msg) => {
                println!("   📤 Sent to {} (id: {})", recipient, &msg.id.to_string()[..8]);
                
                // Update status
                let _ = client.mark_message_status(msg.id, MessageStatus::Delivered);
                let _ = client.mark_message_status(msg.id, MessageStatus::Read);
            }
            Err(e) => println!("   ❌ Error: {:?}", e),
        }
    }

    // Receive messages
    println!();
    for (sender, body) in [
        ("alice@demo.com", "Hi there! Got your message 😊"),
        ("bob@demo.com", "All good here! 👍"),
    ] {
        match client.simulate_incoming_message(sender, body) {
            Ok(msg) => println!("   📥 Received from {} (id: {})", sender, &msg.id.to_string()[..8]),
            Err(e) => println!("   ❌ Error: {:?}", e),
        }
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 6: ENCRYPTION
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 6: ENCRYPTION\n");

    // Send an encrypted message and then decrypt it
    match client.send_message("security-test@demo.com", "This is a top-secret message! 🔐") {
        Ok(msg) => {
            println!("   🔒 Sent encrypted message:");
            println!("      Plaintext: \"{}\"", msg.body);
            if let Some(ref cipher) = msg.ciphertext {
                let preview = if cipher.len() > 50 { &cipher[..50] } else { cipher };
                println!("      Ciphertext: {}...", preview);
            }
            
            // Now decrypt it
            match client.decrypt_message_body(msg.id) {
                Ok(decrypted) => {
                    println!("   🔓 Decrypted: \"{}\"", decrypted);
                    println!("   ✅ Encryption roundtrip successful!");
                }
                Err(e) => println!("   ❌ Decryption error: {:?}", e),
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 7: QR & PAIRING
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 7: QR & PAIRING AUTHENTICATION\n");

    // Pairing code
    client.state.pairing_code = None; // Reset for demo
    match client.request_pairing_code() {
        Ok(code) => {
            println!("   🔢 Pairing code: {}", code);
            if let Some(ref pairing) = client.state.pairing_code {
                println!("      Expires: {}", pairing.expires_at.format("%H:%M:%S"));
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // QR login
    client.state.qr_login = None; // Reset for demo
    match client.generate_qr_login() {
        Ok(qr) => {
            println!("   📱 QR Token: {}", qr.token);
            println!("      Expires: {}", qr.expires_at.format("%H:%M:%S"));
            
            // Verify it
            match client.verify_qr_login(&qr.token) {
                Ok(_) => println!("   ✅ QR verified successfully!"),
                Err(e) => println!("   ❌ Verification error: {:?}", e),
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 8: MEDIA DOWNLOAD
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 8: MEDIA DOWNLOAD\n");

    match client.download_media("https://httpbin.org/image/png", Some("demo_image.png")) {
        Ok(item) => {
            println!("   📥 Downloaded media:");
            println!("      File: {}", item.file_path);
            println!("      Size: {} bytes", item.bytes);
        }
        Err(e) => println!("   ⚠️ Media download error: {:?}", e),
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 9: STATE SUMMARY
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 9: STATE SUMMARY\n");

    println!("   📊 Session Statistics:");
    println!("      Contacts: {}", client.state.contacts.len());
    println!("      Outgoing messages: {}", client.state.outgoing_messages.len());
    println!("      Incoming messages: {}", client.state.incoming_messages.len());
    println!("      Media items: {}", client.state.media.len());
    println!("      Events logged: {}", client.state.events.len());
    println!("      Connected: {}", client.state.connected);
    println!();

    // List contacts
    if !client.state.contacts.is_empty() {
        println!("   👥 Contacts:");
        for contact in &client.state.contacts {
            println!("      - {} ({})", contact.display_name, contact.jid);
        }
        println!();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PART 10: DISCONNECT & PERSIST
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 10: DISCONNECT & PERSIST\n");

    match client.disconnect() {
        Ok(_) => println!("   🔌 Disconnected successfully"),
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    client.store_state(&state_path)?;
    println!("   💾 State saved to: {}", state_path);
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PART 11: EVENTS TIMELINE
    // ═══════════════════════════════════════════════════════════════════════
    println!("▶ PART 11: EVENTS TIMELINE\n");

    println!("   📋 Complete event history:");
    for (i, event) in client.state.events.iter().enumerate() {
        println!("      {:>2}. [{}] {:?}", i + 1, event.at.format("%H:%M:%S"), event.kind);
    }
    println!();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Demo Complete! 🎉                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Session saved to: {:<40} ║", &state_path[..40.min(state_path.len())]);
    println!("║  Run again to see state persistence in action!               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
