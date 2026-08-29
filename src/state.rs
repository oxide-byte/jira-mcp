/// Represents a single MCP call for tracking and display.
#[derive(Clone, Debug)]
pub struct Call {
    /// Unique identifier for the call
    #[allow(dead_code)]
    pub id: String,
    /// HTTP method used (GET, POST, etc.)
    pub method: String,
    /// Endpoint path that was called
    pub path: String,
    /// HTTP status code returned
    pub status_code: u16,
    /// Timestamp when the call was made
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response body or error message
    #[allow(dead_code)]
    pub response: String,
}

impl Call {
    /// Creates a new call record.
    pub fn new(method: String, path: String, status_code: u16, response: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            method,
            path,
            status_code,
            timestamp: chrono::Utc::now(),
            response,
        }
    }
}

/// Maintains a log of all MCP calls made to the server.
#[derive(Debug)]
pub struct CallLog {
    /// List of recorded calls, with newest first
    calls: Vec<Call>,
    /// Maximum number of calls to keep in memory
    max_calls: usize,
}

impl CallLog {
    /// Creates a new empty call log.
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            max_calls: 100,
        }
    }

    /// Adds a new call to the log.
    pub fn add_call(&mut self, call: Call) {
        self.calls.insert(0, call);
        // Keep only the most recent calls to avoid unbounded memory growth
        if self.calls.len() > self.max_calls {
            self.calls.truncate(self.max_calls);
        }
    }

    /// Returns a copy of all calls in the log.
    pub fn get_calls(&self) -> Vec<Call> {
        self.calls.clone()
    }

    /// Clears all calls from the log.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.calls.clear();
    }

    /// Returns the number of calls in the log.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Checks if the log is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}

impl Default for CallLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_creation() {
        let call = Call::new(
            "GET".to_string(),
            "/jira/TEST-1".to_string(),
            200,
            "{}".to_string(),
        );
        assert_eq!(call.method, "GET");
        assert_eq!(call.path, "/jira/TEST-1");
        assert_eq!(call.status_code, 200);
    }

    #[test]
    fn test_call_log_add_and_retrieve() {
        let mut log = CallLog::new();
        let call = Call::new(
            "GET".to_string(),
            "/jira/TEST-1".to_string(),
            200,
            "{}".to_string(),
        );
        log.add_call(call.clone());
        let calls = log.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].path, "/jira/TEST-1");
    }

    #[test]
    fn test_call_log_max_size() {
        let mut log = CallLog::new();
        log.max_calls = 5;
        for i in 0..10 {
            let call = Call::new(
                "GET".to_string(),
                format!("/jira/TEST-{}", i),
                200,
                "{}".to_string(),
            );
            log.add_call(call);
        }
        assert_eq!(log.len(), 5);
    }
}
