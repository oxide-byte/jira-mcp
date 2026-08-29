mod config;
mod jira;
mod modal;
mod server;
mod state;
mod ui;

use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    // Load configuration from environment
    let jira_config = config::JiraConfig::from_env()?;
    let shared_config = config::create_shared_config(jira_config);

    // Create shared state for tracking calls
    let call_log = Arc::new(Mutex::new(state::CallLog::new()));
    let call_log_server = Arc::clone(&call_log);
    let config_server = Arc::clone(&shared_config);

    // Determine server port from environment or use default
    let server_port: u16 = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    // Spawn the web server in a background task
    let server_handle = task::spawn(async move {
        if let Err(e) = server::start_server(call_log_server, config_server, server_port).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Run the TUI in the main task
    // If TUI fails (e.g., in headless environment), keep the server running
    match ui::run_tui(call_log, shared_config).await {
        Ok(_) => {
            // Normal TUI exit
            server_handle.abort();
        }
        Err(e) => {
            eprintln!("TUI error: {}", e);
            eprintln!("Running server in headless mode...");
            // Keep the server running indefinitely if TUI fails
            let _ = server_handle.await;
        }
    }

    Ok(())
}
