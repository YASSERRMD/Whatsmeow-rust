//! Basic Flow Example - Demonstrates the core workflow of the Whatsmeow-rust library.
//!
//! This example shows:
//! - Creating a client with custom configuration
//! - Registering a device (JID)
//! - Connecting to the session
//! - Sending and receiving messages
//! - Marking messages as delivered and read
//! - Disconnecting and persisting state
//!
//! Run with: `cargo run --example basic_flow`

use std::fs;
use whatsmeow_rust::{MessageStatus, SessionState, WhatsmeowClient, WhatsmeowConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Whatsmeow-rust Basic Flow Demo ===\n");

    // Step 1: Create configuration
    println!("📝 Step 1: Creating configuration...");
    let config = WhatsmeowConfig::default()
        .with_database_path("./examples_data/basic.db")
        .with_media_path("./examples_data/media")
        .with_user_agent("whatsmeow-example/1.0");
    println!("   Config: {:?}\n", config);

    // Step 2: Create or load session state
    println!("📂 Step 2: Loading session state...");
    let state_path = "./examples_data/basic_session.json";
    fs::create_dir_all("./examples_data")?;
    let state = match fs::read_to_string(state_path) {
        Ok(contents) => {
            println!("   Found existing session file");
            serde_json::from_str(&contents).unwrap_or_else(|_| {
                println!("   Could not parse, creating new session");
                SessionState::with_device_name("basic-flow-demo")
            })
        }
        Err(_) => {
            println!("   No existing session, creating new one");
            SessionState::with_device_name("basic-flow-demo")
        }
    };

    // Step 3: Create client
    println!("\n🔧 Step 3: Creating client...");
    let mut client = WhatsmeowClient::new(config.clone(), state);
    println!("   Client created successfully");

    // Step 4: Register device if not already registered
    println!("\n📱 Step 4: Registering device...");
    if client.state.is_registered() {
        println!("   Device already registered as: {:?}", client.state.registered_jid);
    } else {
        let jid = "demo-user-12345@s.whatsapp.net";
        client.register_device(jid);
        println!("   Registered with JID: {}", jid);
    }

    // Step 5: Connect
    println!("\n🔌 Step 5: Connecting...");
    match client.connect() {
        Ok(summary) => println!("   {}", summary),
        Err(e) => println!("   Error: {:?}", e),
    }

    // Step 6: Send a message
    println!("\n📤 Step 6: Sending a message...");
    let recipient = "friend-98765@s.whatsapp.net";
    match client.send_message(recipient, "Hello from Whatsmeow-rust! 🦀") {
        Ok(msg) => {
            println!("   Message sent!");
            println!("   ID: {}", msg.id);
            println!("   To: {}", msg.to);
            println!("   Body: {}", msg.body);
            println!("   Status: {:?}", msg.status);
            println!("   Sent at: {}", msg.sent_at);

            // Step 7: Mark as delivered
            println!("\n📬 Step 7: Marking message as delivered...");
            match client.mark_message_status(msg.id, MessageStatus::Delivered) {
                Ok(updated) => println!("   Status updated to: {:?}", updated.status),
                Err(e) => println!("   Error: {:?}", e),
            }

            // Step 8: Mark as read
            println!("\n👁️ Step 8: Marking message as read...");
            match client.mark_message_status(msg.id, MessageStatus::Read) {
                Ok(updated) => println!("   Status updated to: {:?}", updated.status),
                Err(e) => println!("   Error: {:?}", e),
            }
        }
        Err(e) => println!("   Error sending message: {:?}", e),
    }

    // Step 9: Simulate receiving a message
    println!("\n📥 Step 9: Simulating incoming message...");
    match client.simulate_incoming_message(recipient, "Hi there! Got your message 👋") {
        Ok(msg) => {
            println!("   Message received!");
            println!("   ID: {}", msg.id);
            println!("   From: {}", msg.from);
            println!("   Body: {}", msg.body);
            println!("   Received at: {}", msg.received_at);
        }
        Err(e) => println!("   Error: {:?}", e),
    }

    // Step 10: List contacts
    println!("\n👥 Step 10: Listing contacts...");
    if client.state.contacts.is_empty() {
        println!("   No contacts found");
    } else {
        for contact in &client.state.contacts {
            println!("   - {} ({})", contact.display_name, contact.jid);
        }
    }

    // Step 11: List all messages
    println!("\n💬 Step 11: Listing all messages...");
    println!("   Outgoing messages: {}", client.state.outgoing_messages.len());
    for msg in &client.state.outgoing_messages {
        println!("     [{}] → {}: {} ({:?})", msg.sent_at, msg.to, msg.body, msg.status);
    }
    println!("   Incoming messages: {}", client.state.incoming_messages.len());
    for msg in &client.state.incoming_messages {
        println!("     [{}] ← {}: {}", msg.received_at, msg.from, msg.body);
    }

    // Step 12: Disconnect
    println!("\n🔌 Step 12: Disconnecting...");
    match client.disconnect() {
        Ok(_) => println!("   Disconnected successfully"),
        Err(e) => println!("   Error: {:?}", e),
    }

    // Step 13: Persist state
    println!("\n💾 Step 13: Persisting state to {}...", state_path);
    client.store_state(state_path)?;
    println!("   State saved successfully");

    // Step 14: Show events timeline
    println!("\n📊 Step 14: Events timeline:");
    for event in &client.state.events {
        println!("   [{}] {:?}", event.at, event.kind);
    }

    println!("\n=== Demo Complete! ===");
    println!("Session state saved to: {}", state_path);
    println!("Run this example again to see state persistence in action.\n");

    Ok(())
}
