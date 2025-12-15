//! Real WhatsApp Connection Example
//! 
//! This example demonstrates connecting to real WhatsApp servers:
//! - WebSocket connection to wss://web.whatsapp.com/ws/chat
//! - Noise Protocol handshake
//! - QR code generation for device pairing
//! - Echo bot functionality
//! 
//! Run with: cargo run --example whatsapp_connect

use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};

use whatsmeow_rust::{
    Device, JID,
    protocol::{QRPairing, build_text_message},
    types::servers,
    crypto::{KeyPair, NoiseHandshake, Cipher, Hkdf},
    binary::{Node, encode, decode},
};

/// WhatsApp WebSocket endpoint
const WA_ENDPOINT: &str = "wss://web.whatsapp.com/ws/chat";

/// WhatsApp protocol version header
const WA_HEADER: &[u8] = b"WA\x06\x00";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     WhatsApp Real Connection - Whatsmeow Rust              ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Step 1: Initialize device with fresh keys
    println!("📱 Step 1: Initializing device...");
    let mut device = Device::new();
    device.initialize();
    
    let noise_key = device.noise_key.clone().expect("noise key");
    let identity_key = device.identity_key.clone().expect("identity key");
    
    println!("   ✓ Noise key: {}...", &hex::encode(&noise_key.public)[..16]);
    println!("   ✓ Identity key: {}...", &hex::encode(&identity_key.public)[..16]);
    println!();

    // Step 2: Connect to WhatsApp WebSocket
    println!("🌐 Step 2: Connecting to WhatsApp servers...");
    println!("   Endpoint: {}", WA_ENDPOINT);
    
    let connect_result = timeout(
        Duration::from_secs(10),
        connect_async(WA_ENDPOINT)
    ).await;

    let (mut ws_stream, response) = match connect_result {
        Ok(Ok((stream, resp))) => {
            println!("   ✓ Connected! Status: {}", resp.status());
            (stream, resp)
        }
        Ok(Err(e)) => {
            println!("   ✗ Connection failed: {}", e);
            println!();
            println!("Note: WhatsApp may block connections without proper headers.");
            println!("The Noise handshake requires specific protocol implementation.");
            return Ok(());
        }
        Err(_) => {
            println!("   ✗ Connection timeout");
            return Ok(());
        }
    };
    println!();

    // Step 3: Initialize Noise handshake
    println!("🔐 Step 3: Starting Noise Protocol handshake...");
    let mut noise = NoiseHandshake::new_initiator(noise_key.clone());
    
    // Build handshake message 1 (-> e)
    let ephemeral_pub = noise.write_message_1();
    println!("   ✓ Generated ephemeral key: {}...", &hex::encode(&ephemeral_pub)[..16]);
    
    // Build the handshake frame
    // Format: WA header + prologue + ephemeral public key
    let mut handshake_frame = Vec::new();
    handshake_frame.extend_from_slice(WA_HEADER); // WA\x06\x00
    handshake_frame.extend_from_slice(&ephemeral_pub);
    
    println!("   Sending handshake message 1 ({} bytes)...", handshake_frame.len());
    
    // Send handshake
    ws_stream.send(Message::Binary(handshake_frame.into())).await?;
    println!("   ✓ Handshake message 1 sent");
    
    // Wait for response
    println!("   Waiting for server response...");
    let response_result = timeout(
        Duration::from_secs(10),
        ws_stream.next()
    ).await;

    match response_result {
        Ok(Some(Ok(Message::Binary(data)))) => {
            println!("   ✓ Received response: {} bytes", data.len());
            println!("   First bytes: {:02x?}", &data[..data.len().min(20)]);
            
            // Try to parse the response
            if data.len() >= 32 {
                println!();
                println!("📨 Server responded with handshake data!");
                
                // The server sends back: ephemeral + encrypted static + encrypted payload
                // We would need to complete the Noise XX pattern here
                println!("   Server ephemeral: {}...", &hex::encode(&data[..32.min(data.len())])[..32.min(data.len() * 2)]);
            }
        }
        Ok(Some(Ok(Message::Close(frame)))) => {
            println!("   ⚠ Server closed connection");
            if let Some(f) = frame {
                println!("   Close code: {}, reason: {}", f.code, f.reason);
            }
        }
        Ok(Some(Ok(msg))) => {
            println!("   Received other message type: {:?}", msg);
        }
        Ok(Some(Err(e))) => {
            println!("   ✗ Error receiving: {}", e);
        }
        Ok(None) => {
            println!("   ✗ Connection closed unexpectedly");
        }
        Err(_) => {
            println!("   ✗ Timeout waiting for response");
        }
    }
    println!();

    // Step 4: Generate QR code for pairing (would be sent after handshake)
    println!("📲 Step 4: QR Code for pairing...");
    let pairing = QRPairing::new(device.clone());
    
    if let Some(qr_data) = pairing.current_code() {
        println!("   QR Data: {}", &qr_data[..qr_data.len().min(60)]);
        
        match QRPairing::render_qr_ascii(qr_data) {
            Ok(qr_ascii) => {
                println!();
                println!("   Scan with WhatsApp -> Linked Devices -> Link a Device:");
                println!();
                for line in qr_ascii.lines() {
                    println!("   {}", line);
                }
            }
            Err(e) => println!("   Could not render QR: {}", e),
        }
    }
    println!();

    // Step 5: Show what echo bot would do
    println!("🔁 Step 5: Echo Bot Logic (demonstration)...");
    let test_jid = JID::new("1234567890", servers::DEFAULT_USER);
    let incoming_text = "Hello from WhatsApp!";
    let echo_text = format!("Echo: {}", incoming_text);
    
    let msg = build_text_message(&test_jid, &echo_text, None);
    let encoded = encode(&msg);
    
    println!("   When a message is received:");
    println!("   - Parse the incoming node");
    println!("   - Extract sender JID and text");
    println!("   - Build echo response: \"{}\"", echo_text);
    println!("   - Encode: {} bytes", encoded.len());
    println!("   - Send back through encrypted channel");
    println!();

    // Summary
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                   Connection Summary                        ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  ✅ Device initialized with Curve25519 keys                 ║");
    println!("║  ✅ WebSocket connection attempted                          ║");
    println!("║  ✅ Noise handshake message 1 sent                          ║");
    println!("║  📋 QR code generated for device linking                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("To complete the full WhatsApp connection:");
    println!("1. Complete Noise XX handshake (message 2 & 3)");
    println!("2. Send client payload with device info");
    println!("3. Handle QR scan from WhatsApp app");
    println!("4. Process pair-success IQ from server");
    println!("5. Start encrypted message loop");

    // Close connection
    let _ = ws_stream.close(None).await;

    Ok(())
}
