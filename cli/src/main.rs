use clap::{Parser, Subcommand};
use clipboard_core::clipboard::ClipboardData;
use reqwest::Client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Server URL
    #[arg(short, long, default_value = "http://localhost:5033")]
    url: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get clipboard content from server
    Get,
    /// Set clipboard content to server
    Set {
        /// Content to set
        content: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Client::new();
    let api_url = format!("{}/SyncClipboard.json", args.url);

    match args.command {
        Commands::Get => {
            let resp = client.get(&api_url).send().await?;
            if resp.status().is_success() {
                let data: ClipboardData = resp.json().await?;
                match data {
                    ClipboardData::Text { content, .. } => println!("{}", content),
                    _ => println!("Non-text content"),
                }
            } else {
                eprintln!("Failed to get clipboard: {}", resp.status());
            }
        }
        Commands::Set { content } => {
            let data = ClipboardData::new_text(content);
            let resp = client.put(&api_url).json(&data).send().await?;
            if resp.status().is_success() {
                println!("Clipboard set successfully");
            } else {
                eprintln!("Failed to set clipboard: {}", resp.status());
            }
        }
    }
    Ok(())
}
