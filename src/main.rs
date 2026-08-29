mod config;
mod jira;
mod logs;
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
    // Create shared log collector
    let log_collector = Arc::new(Mutex::new(logs::LogCollector::new()));

    // Initialize logging to write to the collector instead of stdout
    logs::init_tracing(log_collector.clone());

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
            // Log server errors through tracing instead of stderr
            tracing::error!("Server error: {}", e);
        }
    });

    // Run the TUI in the main task
    // If TUI fails (e.g., in headless environment), keep the server running
    match ui::run_tui(call_log, shared_config, log_collector).await {
        Ok(_) => {
            // Normal TUI exit
            server_handle.abort();
        }
        Err(e) => {
            // Log TUI errors through tracing instead of stderr
            tracing::error!("TUI error: {}", e);
            tracing::info!("Running server in headless mode...");
            // Keep the server running indefinitely if TUI fails
            let _ = server_handle.await;
        }
    }

    Ok(())
}
