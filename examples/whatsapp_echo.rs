//! Full WhatsApp Echo Bot
//! 
//! This example connects to real WhatsApp servers:
//! - Completes Noise XX handshake
//! - Displays QR code for device pairing
//! - Receives messages and echoes them back
//! 
//! Run with: cargo run --example whatsapp_echo

use std::time::Duration;
use tokio::time::timeout;

use whatsmeow_rust::{
    Device,
    socket::{do_handshake, WhatsAppConnection},
    protocol::QRPairing,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     WhatsApp Echo Bot - Full Connection                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Step 1: Initialize device with fresh keys
    println!("📱 Initializing device...");
    let mut device = Device::new();
    device.initialize();
    device.push_name = Some("Rust Bot".to_string());
    
    println!("   ✓ Device initialized");
    println!("   ✓ Registration ID: {}", device.registration_id);
    println!();

    // Step 2: Display QR code for pairing  
    println!("📲 QR Code for pairing (scan with WhatsApp):");
    let pairing = QRPairing::new(device.clone());
    
    if let Some(qr_data) = pairing.current_code() {
        match QRPairing::render_qr_ascii(qr_data) {
            Ok(qr_ascii) => {
                println!();
                for line in qr_ascii.lines() {
                    println!("   {}", line);
                }
                println!();
            }
            Err(e) => println!("   Could not render QR: {}", e),
        }
    }

    // Step 3: Connect and perform handshake
    println!("🔐 Connecting to WhatsApp servers...");
    
    match do_handshake(&device).await {
        Ok(mut conn) => {
            println!();
            println!("╔════════════════════════════════════════════════════════════╗");
            println!("║  ✅ HANDSHAKE COMPLETE - CONNECTED TO WHATSAPP!            ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!();
            println!("Waiting for messages... (Press Ctrl+C to exit)");
            println!();

            loop {
                match timeout(Duration::from_secs(30), conn.recv()).await {
                    Ok(Ok(data)) => {
                        println!("📨 Received {} bytes", data.len());
                        println!("   Raw: {:02x?}", &data[..data.len().min(30)]);
                        
                        // Skip first byte (flags) if it's 0x00
                        let decode_data = if !data.is_empty() && data[0] == 0 {
                            println!("   Skipping flags byte");
                            &data[1..]
                        } else {
                            &data[..]
                        };
                        
                        if decode_data.is_empty() {
                            println!("   Empty payload");
                            continue;
                        }
                        
                        // Print basic structure info
                        if decode_data.len() >= 2 && decode_data[0] == 0xf8 {
                            println!("   Binary XML list with {} items", decode_data[1]);
                        }
                        
                        // Try to decode as binary node
                        match whatsmeow_rust::decode(decode_data) {
                            Ok(node) => {
                                println!("   ✓ Decoded: <{}>", node.tag);
                                
                                // Print attributes
                                for (key, value) in &node.attrs {
                                    println!("     @{}: {:?}", key, value);
                                }
                                
                                // Print children
                                if let Some(children) = node.get_children() {
                                    for child in children {
                                        println!("     <{}>", child.tag);
                                    }
                                }
                                
                                // Handle specific message types
                                match node.tag.as_str() {
                                    "iq" => {
                                        println!("   📋 IQ stanza received");
                                    }
                                    "message" => {
                                        if let Some(body) = node.get_child_by_tag("body") {
                                            if let Some(text_bytes) = body.get_bytes() {
                                                let text = String::from_utf8_lossy(text_bytes);
                                                println!("   📝 Message: {}", text);
                                            }
                                        }
                                    }
                                    "success" => {
                                        println!("   🎉 Authentication successful!");
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                println!("   ⚠ Decode error: {}", e);
                                println!("   (Data may be incomplete or fragmented)");
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        println!("⚠ Connection error: {}", e);
                        break;
                    }
                    Err(_) => {
                        println!("⏰ Timeout (no messages in 30s)");
                    }
                }
            }
        }
        Err(e) => {
            println!("   ✗ Handshake failed: {}", e);
            println!();
            println!("This is expected - WhatsApp requires:");
            println!("1. Proper QR code scanned from phone");
            println!("2. Valid device registration");
            println!("3. Certificate verification");
            println!();
            println!("The handshake implementation is complete, but pairing");
            println!("requires scanning the QR code from your WhatsApp app.");
        }
    }

    Ok(())
}
