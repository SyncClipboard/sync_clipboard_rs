pub mod config;
pub mod clipboard;
pub mod clipboard_handler;
pub mod sync;
pub mod mobile_api;
pub mod discovery;

uniffi::setup_scaffolding!();
pub mod crypto;

pub fn hello() -> String {
    "Hello from core".to_string()
}
