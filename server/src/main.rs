use clipboard_core::config::Config;
use server::run;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let config = Config::new().expect("Failed to load config");
    tracing::info!("Config loaded. Port: {}", config.server.port);

    if let Err(e) = run(config).await {
        tracing::error!("Server error: {}", e);
    }
}
