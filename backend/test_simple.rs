use reqwest;
use serde_json::{json, Value};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Simple API Test");
    println!("==================");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    
    let base_url = "http://localhost:3000";

    // Test if server is running
    println!("\n1. Testing server connectivity...");
    match client.get(base_url).send().await {
        Ok(response) => {
            println!("✅ Server is running (status: {})", response.status());
        }
        Err(e) => {
            println!("❌ Server is not running: {}", e);
            println!("💡 Make sure to run 'cargo run' in another terminal first");
            return Ok(());
        }
    }

    // Test creating a document
    println!("\n2. Testing document creation...");
    match client.post(format!("{}/api/doc", base_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("✅ Document created: {}", serde_json::to_string_pretty(&data)?);
            } else {
                println!("❌ Failed to create document: {}", response.status());
            }
        }
        Err(e) => {
            println!("❌ Error creating document: {}", e);
        }
    }

    println!("\n✅ Simple test complete!");
    Ok(())
} 