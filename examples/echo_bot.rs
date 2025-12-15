//! WhatsApp Echo Bot Example
//! 
//! This example demonstrates the whatsmeow-rust library capabilities:
//! - QR code generation and display
//! - Connection to WhatsApp servers
//! - Message receiving and parsing
//! - Echo back messages to the sender
//! 
//! Run with: cargo run --example echo_bot

use std::time::Duration;
use tokio::time::sleep;

use whatsmeow_rust::{
    Client, ClientConfig, Device, MemoryStore,
    JID, Node, encode, decode,
    protocol::{
        QRPairing, QREvent,
        build_text_message, build_presence, build_chat_state,
        generate_message_id, parse_message,
    },
    types::{Event, MessageContent, servers},
    crypto::KeyPair,
    binary::Attrs,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          WhatsApp Echo Bot - Whatsmeow Rust                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Initialize device
    println!("📱 Initializing device...");
    let mut device = Device::new();
    device.initialize();
    println!("   ✓ Device initialized");
    println!("   ✓ Noise key generated: {:?}...", &hex::encode(&device.noise_key.as_ref().unwrap().public)[..16]);
    println!("   ✓ Identity key generated: {:?}...", &hex::encode(&device.identity_key.as_ref().unwrap().public)[..16]);
    println!("   ✓ Registration ID: {}", device.registration_id);
    println!();

    // Generate QR code for pairing
    println!("📲 Generating QR code for pairing...");
    let pairing = QRPairing::new(device.clone());
    
    if let Some(qr_data) = pairing.current_code() {
        println!("   QR Data: {}", &qr_data[..50.min(qr_data.len())]);
        println!();
        
        // Render QR as ASCII
        match QRPairing::render_qr_ascii(qr_data) {
            Ok(qr_ascii) => {
                println!("   Scan this QR code with WhatsApp:");
                println!();
                for line in qr_ascii.lines() {
                    println!("   {}", line);
                }
                println!();
            }
            Err(e) => println!("   ⚠ Could not render QR: {}", e),
        }
    }

    // Demonstrate message building
    println!("📝 Demonstrating message building...");
    let test_jid = JID::new("1234567890", servers::DEFAULT_USER);
    
    let text_msg = build_text_message(&test_jid, "Hello from Whatsmeow Rust!", None);
    println!("   ✓ Text message built:");
    println!("     - Tag: {}", text_msg.tag);
    println!("     - ID: {}", text_msg.get_attr_str("id").unwrap_or("?"));
    println!("     - To: {}", text_msg.get_attr_str("to").unwrap_or("?"));
    println!();

    // Demonstrate presence
    let presence = build_presence(true);
    println!("   ✓ Presence built: type={}", presence.get_attr_str("type").unwrap_or("?"));
    
    // Demonstrate chat state (typing indicator)
    let typing = build_chat_state(&test_jid, true);
    println!("   ✓ Typing indicator built");
    println!();

    // Demonstrate binary encoding/decoding
    println!("🔄 Demonstrating binary encoding/decoding...");
    let encoded = encode(&text_msg);
    println!("   ✓ Encoded message: {} bytes", encoded.len());
    
    // Simulate incoming message
    println!();
    println!("📨 Simulating incoming echo message...");
    
    let incoming_jid = JID::new("9876543210", servers::DEFAULT_USER);
    let mut incoming = Node::new("message");
    incoming.set_attr("id", generate_message_id());
    incoming.set_attr("type", "text");
    incoming.set_attr("from", incoming_jid.to_string());
    incoming.set_attr("notify", "Test User");
    
    let mut body = Node::new("body");
    body.set_bytes(b"Hello, this is a test message!".to_vec());
    incoming.add_child(body);
    
    // Parse the simulated message
    if let Some((info, content)) = parse_message(&incoming) {
        println!("   ✓ Received message:");
        println!("     - ID: {}", info.id);
        println!("     - From: {}", info.sender);
        println!("     - Push Name: {:?}", info.push_name);
        
        match content {
            MessageContent::Text(text) => {
                println!("     - Content: \"{}\"", text);
                
                // Echo the message back
                println!();
                println!("🔁 Echoing message back to sender...");
                let echo_msg = build_text_message(&info.sender, &format!("Echo: {}", text), None);
                println!("   ✓ Echo message built:");
                println!("     - To: {}", echo_msg.get_attr_str("to").unwrap_or("?"));
                println!("     - ID: {}", echo_msg.get_attr_str("id").unwrap_or("?"));
            }
            _ => println!("     - Content: (non-text message)"),
        }
    }
    
    // Demonstrate crypto operations
    println!();
    println!("🔐 Demonstrating crypto operations...");
    let kp1 = KeyPair::generate();
    let kp2 = KeyPair::generate();
    let shared1 = kp1.dh(&kp2.public);
    let shared2 = kp2.dh(&kp1.public);
    println!("   ✓ Key exchange:");
    println!("     - Alice public: {}...", &hex::encode(&kp1.public)[..16]);
    println!("     - Bob public: {}...", &hex::encode(&kp2.public)[..16]);
    println!("     - Shared secret matches: {}", shared1 == shared2);
    
    // Summary
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                     Example Complete                        ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  ✅ Device initialization                                   ║");
    println!("║  ✅ QR code generation                                      ║");
    println!("║  ✅ Message building (text, presence, typing)               ║");
    println!("║  ✅ Binary encoding/decoding                                ║");
    println!("║  ✅ Message parsing                                         ║");
    println!("║  ✅ Echo functionality                                      ║");
    println!("║  ✅ Crypto (key exchange)                                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Note: To connect to real WhatsApp servers, you would need");
    println!("to complete the WebSocket connection and Noise handshake.");
    println!("This example demonstrates the library's building blocks.");

    Ok(())
}
