use crate::config::AuthMethod;
use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde_json::Value;

/// Jira client for connecting to a Jira instance.
///
/// Supports two authentication methods:
/// 1. Basic auth with username/password
/// 2. Bearer token auth with API key
///
/// Returns raw JSON responses without transformation.
#[derive(Clone)]
pub struct JiraClient {
    base_url: String,
    username: String,
    credential: String,
    auth_method: AuthMethod,
    client: Client,
}

impl JiraClient {
    /// Creates a new Jira client with the specified authentication method.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Jira instance (e.g., "https://your-domain.atlassian.net")
    /// * `username` - Jira username or email (for API key auth, this is the email)
    /// * `credential` - Password (for basic auth) or API key (for bearer auth)
    /// * `auth_method` - The authentication method to use
    pub fn new(
        base_url: String,
        username: String,
        credential: String,
        auth_method: AuthMethod,
    ) -> Self {
        Self {
            base_url,
            username,
            credential,
            auth_method,
            client: Client::new(),
        }
    }

    /// Creates a new Jira client with username/password authentication.
    #[allow(dead_code)]
    pub fn with_user_password(base_url: String, username: String, password: String) -> Self {
        Self::new(base_url, username, password, AuthMethod::UserPassword)
    }

    /// Creates a new Jira client with API key authentication.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Jira instance
    /// * `email` - Email address associated with the API key
    /// * `api_key` - The Jira API key
    #[allow(dead_code)]
    pub fn with_api_key(base_url: String, email: String, api_key: String) -> Self {
        Self::new(base_url, email, api_key, AuthMethod::ApiKey)
    }

    /// Creates a new Jira client with empty credentials.
    /// Used for unauthenticated access.
    #[allow(dead_code)]
    pub fn new_anonymous(base_url: String) -> Self {
        Self {
            base_url,
            username: String::new(),
            credential: String::new(),
            auth_method: AuthMethod::UserPassword,
            client: Client::new(),
        }
    }

    /// Fetches a Jira issue by its key/ID from the Jira server.
    ///
    /// Returns the raw JSON response from Jira without transformation.
    /// This allows access to all fields returned by the Jira API.
    ///
    /// # Arguments
    ///
    /// * `issue_key` - The Jira issue key (e.g., "PROJ-123")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The issue cannot be found (404)
    /// - Authentication fails (401)
    /// - Access is forbidden (403)
    /// - Other API errors occur
    pub async fn get_issue(&self, issue_key: &str) -> Result<Value> {
        // Build the API URL
        let url = format!(
            "{}/rest/api/3/issue/{}",
            self.base_url.trim_end_matches('/'),
            issue_key
        );

        // Create the request
        let mut request = self.client.get(&url);

        // Add authentication header based on auth method
        if !self.username.is_empty() && !self.credential.is_empty() {
            match self.auth_method {
                AuthMethod::UserPassword => {
                    // Use Basic auth with base64(username:password)
                    let credentials = format!("{}:{}", self.username, self.credential);
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&credentials);
                    request = request.header("Authorization", format!("Basic {}", encoded));
                }
                AuthMethod::ApiKey => {
                    // Use Bearer token with the API key
                    request =
                        request.header("Authorization", format!("Bearer {}", self.credential));
                }
            }
        }

        // Send the request
        let response = request.header("Accept", "application/json").send().await?;

        // Handle different status codes
        match response.status() {
            reqwest::StatusCode::OK => {
                let json_response: Value = response.json().await?;
                Ok(json_response)
            }
            reqwest::StatusCode::NOT_FOUND => Err(anyhow::anyhow!(
                "Jira issue '{}' not found (404)",
                issue_key
            )),
            reqwest::StatusCode::UNAUTHORIZED => Err(anyhow::anyhow!(
                "Unauthorized: Check your Jira credentials (401)"
            )),
            reqwest::StatusCode::FORBIDDEN => Err(anyhow::anyhow!(
                "Access forbidden: You do not have permission to view this issue (403)"
            )),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!("Jira API error ({}): {}", status, body))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jira_client_new_user_password() {
        let client = JiraClient::with_user_password(
            "http://localhost:8080".to_string(),
            "user@example.com".to_string(),
            "mypassword".to_string(),
        );
        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.username, "user@example.com");
        assert_eq!(client.credential, "mypassword");
        assert_eq!(client.auth_method, AuthMethod::UserPassword);
    }

    #[tokio::test]
    async fn test_jira_client_new_api_key() {
        let client = JiraClient::with_api_key(
            "http://localhost:8080".to_string(),
            "user@example.com".to_string(),
            "atatt1234567890".to_string(),
        );
        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.username, "user@example.com");
        assert_eq!(client.credential, "atatt1234567890");
        assert_eq!(client.auth_method, AuthMethod::ApiKey);
    }

    #[tokio::test]
    async fn test_jira_client_new_anonymous() {
        let client = JiraClient::new_anonymous("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.username.is_empty());
        assert!(client.credential.is_empty());
    }

    // Note: Real API tests would require a running Jira instance
    // For now, these tests verify the client can be instantiated correctly
}
