use crate::config::SharedConfig;
use crate::jira::JiraClient;
use crate::state::{Call, CallLog};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use parking_lot::Mutex;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

/// Application state shared between handlers.
#[derive(Clone)]
pub struct AppState {
    jira_client: JiraClient,
    call_log: Arc<Mutex<CallLog>>,
    #[allow(dead_code)]
    config: SharedConfig,
}

/// Starts the Axum web server.
///
/// # Arguments
///
/// * `call_log` - Shared call log for tracking MCP requests
/// * `config` - Shared Jira configuration
/// * `port` - Port number to listen on
///
/// # Errors
///
/// Returns an error if the server cannot bind to the port or fails during runtime.
pub async fn start_server(
    call_log: Arc<Mutex<CallLog>>,
    config: SharedConfig,
    port: u16,
) -> anyhow::Result<()> {
    let (jira_url, username, credential, auth_method) = {
        let cfg = config.lock();
        (
            cfg.url.clone(),
            cfg.username.clone(),
            cfg.password.clone(),
            cfg.auth_method,
        )
    };

    let jira_client = JiraClient::new(jira_url, username, credential, auth_method);

    let state = AppState {
        jira_client,
        call_log,
        config,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/jira/{id}", get(get_jira_issue))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let addr = listener.local_addr()?;
    info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint.
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Fetches a Jira issue by ID and logs the call.
///
/// # Arguments
///
/// * `id` - The Jira issue ID/key
/// * `state` - Application state containing the Jira client and call log
///
/// # Errors
///
/// Returns a 404 if the issue is not found, or 500 for internal errors.
async fn get_jira_issue(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    let path = format!("/jira/{}", id);

    // Fetch the issue from Jira
    match state.jira_client.get_issue(&id).await {
        Ok(issue) => {
            let response_json = serde_json::to_string(&issue).unwrap_or_default();
            let status_code = 200;

            // Log the successful call
            {
                let mut log = state.call_log.lock();
                log.add_call(Call::new(
                    "GET".to_string(),
                    path,
                    status_code,
                    response_json,
                ));
            }

            info!("Retrieved Jira issue: {}", id);
            (StatusCode::OK, axum::Json(issue))
        }
        Err(e) => {
            let error_msg = format!("Failed to fetch issue: {}", e);
            let status_code = 500;

            // Log the failed call
            {
                let mut log = state.call_log.lock();
                log.add_call(Call::new(
                    "GET".to_string(),
                    path,
                    status_code,
                    error_msg.clone(),
                ));
            }

            error!("Error fetching Jira issue: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({"error": error_msg})),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{create_shared_config, AuthMethod, JiraConfig};

    #[tokio::test]
    async fn test_app_state_creation() {
        let call_log = Arc::new(Mutex::new(CallLog::new()));
        let config = create_shared_config(JiraConfig::new(
            "http://localhost:8080".to_string(),
            "user@example.com".to_string(),
            "token123".to_string(),
            AuthMethod::UserPassword,
        ));
        let state = AppState {
            jira_client: JiraClient::new(
                "http://localhost:8080".to_string(),
                "user@example.com".to_string(),
                "token123".to_string(),
                AuthMethod::UserPassword,
            ),
            call_log,
            config,
        };
        assert_eq!(state.call_log.lock().len(), 0);
    }
}
