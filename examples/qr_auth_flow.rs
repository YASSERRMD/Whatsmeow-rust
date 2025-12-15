//! QR Authentication Flow Example - Demonstrates QR and pairing code flows.
//!
//! This example shows:
//! - Requesting pairing codes with expiration
//! - Generating QR login tokens
//! - Verifying QR tokens
//! - Handling expired tokens
//!
//! Run with: `cargo run --example qr_auth_flow`

use std::fs;
use whatsmeow_rust::{SessionState, WhatsmeowClient, WhatsmeowConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Whatsmeow-rust QR Authentication Flow Demo ===\n");

    // Setup
    fs::create_dir_all("./examples_data")?;
    let config = WhatsmeowConfig::default()
        .with_database_path("./examples_data/qr_auth.db")
        .with_media_path("./examples_data/media");

    let state = SessionState::with_device_name("qr-auth-demo");
    let mut client = WhatsmeowClient::new(config, state);

    // Step 1: Register device (required for auth flows)
    println!("📱 Step 1: Registering device...");
    let jid = "qr-demo-user@s.whatsapp.net";
    client.register_device(jid);
    println!("   Registered with JID: {}\n", jid);

    // Step 2: Request pairing code
    println!("🔐 Step 2: Requesting pairing code...");
    match client.request_pairing_code() {
        Ok(code) => {
            println!("   ✅ Pairing code generated: {}", code);
            if let Some(ref pairing) = client.state.pairing_code {
                println!("   Expires at: {}", pairing.expires_at);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Step 3: Try requesting another pairing code (should fail if one exists and not expired)
    println!("\n🔐 Step 3: Trying to request another pairing code...");
    match client.request_pairing_code() {
        Ok(code) => println!("   ✅ New pairing code: {}", code),
        Err(e) => println!("   ⚠️ Expected behavior: {:?}", e),
    }

    // Step 4: Generate QR login token
    println!("\n📷 Step 4: Generating QR login token...");
    match client.generate_qr_login() {
        Ok(qr) => {
            println!("   ╔════════════════════════════════════════╗");
            println!("   ║             QR LOGIN TOKEN             ║");
            println!("   ╠════════════════════════════════════════╣");
            println!("   ║  Token: {}  ║", qr.token);
            println!("   ║  Issued: {}  ║", qr.issued_at.format("%H:%M:%S"));
            println!("   ║  Expires: {}  ║", qr.expires_at.format("%H:%M:%S"));
            println!("   ╚════════════════════════════════════════╝");
            println!();
            println!("   In a real app, this would be rendered as a QR code image");
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Step 5: Verify QR token with wrong token
    println!("\n🔍 Step 5: Testing verification with wrong token...");
    match client.verify_qr_login("WRONG-TOKEN") {
        Ok(_) => println!("   ✅ Verified (unexpected)"),
        Err(e) => println!("   ⚠️ Expected rejection: {:?}", e),
    }

    // Step 6: Verify with correct token
    println!("\n🔍 Step 6: Verifying with correct token...");
    if let Some(ref qr) = client.state.qr_login {
        let token = qr.token.clone();
        match client.verify_qr_login(&token) {
            Ok(verified) => {
                println!("   ✅ QR login verified successfully!");
                println!("   Token: {}", verified.token);
                println!("   Verified: {}", verified.verified);
            }
            Err(e) => println!("   ❌ Error: {:?}", e),
        }
    } else {
        println!("   No QR login token available");
    }

    // Step 7: Generate a fresh QR token
    println!("\n📷 Step 7: Generating fresh QR token for demo...");
    // Force clean state for new QR
    client.state.qr_login = None;
    match client.generate_qr_login() {
        Ok(qr) => {
            println!("   New QR token: {}", qr.token);
            println!("   Expires: {}", qr.expires_at);
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Step 8: Show events timeline
    println!("\n📊 Step 8: Authentication events timeline:");
    for event in &client.state.events {
        println!("   [{}] {:?}", event.at.format("%H:%M:%S"), event.kind);
    }

    // Save state
    let state_path = "./examples_data/qr_auth_session.json";
    println!("\n💾 Saving session to {}...", state_path);
    client.store_state(state_path)?;

    println!("\n=== QR Auth Flow Demo Complete! ===\n");

    Ok(())
}
