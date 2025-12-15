//! Media Download Demo - Demonstrates media handling capabilities.
//!
//! This example shows:
//! - Configuring media download paths
//! - Downloading media from URLs
//! - Tracking downloaded media in session
//! - Listing media history
//!
//! Run with: `cargo run --example media_demo`
//!
//! Note: This example requires network access to download files.

use std::fs;
use whatsmeow_rust::{SessionState, WhatsmeowClient, WhatsmeowConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Whatsmeow-rust Media Download Demo ===\n");

    // Setup with custom media path
    let media_dir = "./examples_data/downloads";
    fs::create_dir_all(media_dir)?;
    
    println!("📂 Step 1: Configuring media settings...");
    let config = WhatsmeowConfig::default()
        .with_database_path("./examples_data/media.db")
        .with_media_path(media_dir);
    
    println!("   Media directory: {}\n", media_dir);

    let state = SessionState::with_device_name("media-demo");
    let mut client = WhatsmeowClient::new(config, state);

    // Register and connect (required for media downloads)
    println!("📱 Step 2: Setting up client...");
    client.register_device("media-demo@s.whatsapp.net");
    match client.connect() {
        Ok(summary) => println!("   {}\n", summary),
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
            return Ok(());
        }
    }

    // Download media from various public URLs
    println!("📥 Step 3: Downloading media files...\n");
    
    // Using reliable public test URLs
    let media_urls = [
        ("https://via.placeholder.com/150.png", Some("placeholder_150.png")),
        ("https://via.placeholder.com/300x200.jpg", Some("placeholder_300x200.jpg")),
        ("https://raw.githubusercontent.com/YASSERRMD/Whatsmeow-rust/main/README.md", Some("readme.md")),
    ];

    for (url, filename) in media_urls {
        println!("   🔗 URL: {}", url);
        print!("   ⏳ Downloading... ");
        
        match client.download_media(url, filename) {
            Ok(item) => {
                println!("✅ Done!");
                println!("      ID: {}", item.id);
                println!("      Saved to: {}", item.file_path);
                println!("      Size: {} bytes", item.bytes);
                println!("      Downloaded at: {}", item.downloaded_at);
            }
            Err(e) => {
                println!("❌ Failed");
                println!("      Error: {:?}", e);
            }
        }
        println!();
    }

    // Download with auto-generated filename
    println!("📥 Step 4: Download with auto-generated filename...\n");
    let auto_url = "https://via.placeholder.com/100x100.png";
    println!("   URL: {}", auto_url);
    print!("   Downloading... ");
    
    match client.download_media(auto_url, None::<&str>) {
        Ok(item) => {
            println!("✅ Done!");
            println!("   Auto-generated path: {}", item.file_path);
            println!("   Size: {} bytes", item.bytes);
        }
        Err(e) => {
            println!("❌ Failed");
            println!("   Error: {:?}", e);
        }
    }

    // List all downloaded media
    println!("\n📋 Step 5: Media inventory:\n");
    
    if client.state.media.is_empty() {
        println!("   No media downloaded yet.");
    } else {
        println!("   {:<36} | {:<12} | {:<30} | Source", "ID", "Size", "Local Path");
        println!("   {}", "-".repeat(100));
        
        for item in &client.state.media {
            let source_preview = if item.source.len() > 30 {
                format!("...{}", &item.source[item.source.len()-27..])
            } else {
                item.source.clone()
            };
            println!(
                "   {} | {:>10} B | {:<30} | {}",
                item.id,
                item.bytes,
                item.file_path,
                source_preview
            );
        }
        
        let total_bytes: u64 = client.state.media.iter().map(|m| m.bytes).sum();
        println!("\n   📊 Total: {} files, {} bytes", client.state.media.len(), total_bytes);
    }

    // Show media events
    println!("\n📊 Step 6: Media-related events:");
    for event in &client.state.events {
        let event_str = format!("{:?}", event.kind);
        if event_str.contains("Media") {
            println!("   [{}] {:?}", event.at.format("%H:%M:%S"), event.kind);
        }
    }

    // Check actual files on disk
    println!("\n📁 Step 7: Files in media directory:");
    match fs::read_dir(media_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    println!(
                        "   {} ({} bytes)",
                        entry.file_name().to_string_lossy(),
                        metadata.len()
                    );
                }
            }
        }
        Err(e) => println!("   Could not read directory: {:?}", e),
    }

    // Save state
    let state_path = "./examples_data/media_session.json";
    println!("\n💾 Saving session to {}...", state_path);
    client.store_state(state_path)?;

    println!("\n=== Media Demo Complete! ===");
    println!("Downloaded files are in: {}", media_dir);
    println!("Session state saved to: {}\n", state_path);

    Ok(())
}
