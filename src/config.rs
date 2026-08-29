use anyhow::Result;
use parking_lot::Mutex;
use std::sync::Arc;

/// Authentication method for Jira
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthMethod {
    /// Username and password (Basic auth)
    UserPassword,
    /// API key (Bearer token)
    ApiKey,
}

impl AuthMethod {
    pub fn label(&self) -> &'static str {
        match self {
            AuthMethod::UserPassword => "Username/Password",
            AuthMethod::ApiKey => "API Key",
        }
    }
}

/// Jira configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct JiraConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub auth_method: AuthMethod,
}

impl JiraConfig {
    /// Loads configuration from environment variables (.env file or system env)
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing
    pub fn from_env() -> Result<Self> {
        // Load from .env file if it exists
        let _ = dotenvy::dotenv();

        let url = std::env::var("JIRA_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Determine auth method from environment
        let auth_method = if std::env::var("JIRA_API_KEY").is_ok() {
            AuthMethod::ApiKey
        } else {
            AuthMethod::UserPassword
        };

        let (username, password) = match auth_method {
            AuthMethod::ApiKey => {
                // For API key, store the email in username and API key in password
                let email = std::env::var("JIRA_USERNAME").unwrap_or_default();
                let api_key = std::env::var("JIRA_API_KEY").unwrap_or_default();
                (email, api_key)
            }
            AuthMethod::UserPassword => {
                let username = std::env::var("JIRA_USERNAME").unwrap_or_default();
                let password = std::env::var("JIRA_PASSWORD").unwrap_or_default();
                (username, password)
            }
        };

        Ok(Self {
            url,
            username,
            password,
            auth_method,
        })
    }

    /// Creates a configuration from individual values
    #[allow(dead_code)]
    pub fn new(url: String, username: String, password: String, auth_method: AuthMethod) -> Self {
        Self {
            url,
            username,
            password,
            auth_method,
        }
    }

    /// Checks if credentials are configured
    pub fn is_configured(&self) -> bool {
        !self.username.is_empty() && !self.password.is_empty() && !self.url.is_empty()
    }

    /// Returns a display name (masking password)
    #[allow(dead_code)]
    pub fn display_summary(&self) -> String {
        let credential_mask = if self.password.is_empty() {
            "[not set]".to_string()
        } else {
            format!("[{} chars]", self.password.len())
        };

        let credential_label = match self.auth_method {
            AuthMethod::ApiKey => "API Key",
            AuthMethod::UserPassword => "Password",
        };

        let username_or_email = if self.username.is_empty() {
            "[not set]".to_string()
        } else {
            self.username.clone()
        };

        format!(
            "URL: {}\nAuth Method: {}\nUsername/Email: {}\n{}: {}",
            self.url,
            self.auth_method.label(),
            username_or_email,
            credential_label,
            credential_mask
        )
    }
}

/// Thread-safe configuration holder
pub type SharedConfig = Arc<Mutex<JiraConfig>>;

/// Creates a shared configuration
pub fn create_shared_config(config: JiraConfig) -> SharedConfig {
    Arc::new(Mutex::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new_user_password() {
        let config = JiraConfig::new(
            "http://localhost:8080".to_string(),
            "user".to_string(),
            "pass".to_string(),
            AuthMethod::UserPassword,
        );

        assert!(config.is_configured());
        assert_eq!(config.url, "http://localhost:8080");
        assert_eq!(config.username, "user");
        assert_eq!(config.password, "pass");
        assert_eq!(config.auth_method, AuthMethod::UserPassword);
    }

    #[test]
    fn test_config_new_api_key() {
        let config = JiraConfig::new(
            "http://localhost:8080".to_string(),
            "user@example.com".to_string(),
            "atatt1234567890".to_string(),
            AuthMethod::ApiKey,
        );

        assert!(config.is_configured());
        assert_eq!(config.auth_method, AuthMethod::ApiKey);
    }

    #[test]
    fn test_config_not_configured() {
        let config = JiraConfig::new(
            "http://localhost:8080".to_string(),
            "".to_string(),
            "".to_string(),
            AuthMethod::UserPassword,
        );

        assert!(!config.is_configured());
    }

    #[test]
    fn test_display_summary() {
        let config = JiraConfig::new(
            "http://localhost:8080".to_string(),
            "testuser".to_string(),
            "mypassword".to_string(),
            AuthMethod::UserPassword,
        );

        let summary = config.display_summary();
        assert!(summary.contains("http://localhost:8080"));
        assert!(summary.contains("testuser"));
        assert!(summary.contains("[10 chars]"));
        assert!(summary.contains("Username/Password"));
    }

    #[test]
    fn test_display_summary_api_key() {
        let config = JiraConfig::new(
            "http://localhost:8080".to_string(),
            "user@example.com".to_string(),
            "atatt1234567890".to_string(),
            AuthMethod::ApiKey,
        );

        let summary = config.display_summary();
        assert!(summary.contains("API Key"));
        assert!(summary.contains("user@example.com"));
    }
}
