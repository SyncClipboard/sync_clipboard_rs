#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build_info;

fn main() {
    println!("Starting SyncClipboard...");
    println!("Build Hash: {}", build_info::BUILD_HASH);
    println!("Build Time: {}", build_info::BUILD_TIME);
    syncclipboard_rs::run();
}
