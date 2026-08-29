use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use std::fmt;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

pub type SharedLogCollector = Arc<Mutex<LogCollector>>;

/// Represents a single log entry for display in the logs panel
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Log level (INFO, DEBUG, WARN, ERROR, etc.)
    pub level: String,
    /// Log message
    pub message: String,
    /// Target module that produced the log
    pub target: String,
    /// Timestamp when the log was created
    pub timestamp: DateTime<Utc>,
}

impl LogEntry {
    /// Creates a new log entry
    pub fn new(level: String, message: String, target: String) -> Self {
        Self {
            level,
            message,
            target,
            timestamp: Utc::now(),
        }
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} - {} ({})",
            self.timestamp.format("%H:%M:%S%.3f"),
            self.level,
            self.message,
            self.target
        )
    }
}

/// Maintains a log of all application events and errors
#[derive(Debug)]
pub struct LogCollector {
    /// List of log entries, with newest first
    entries: Vec<LogEntry>,
    /// Maximum number of entries to keep in memory
    max_entries: usize,
}

impl LogCollector {
    /// Creates a new empty log collector
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 200,
        }
    }

    /// Adds a new log entry to the collector
    pub fn add_entry(&mut self, entry: LogEntry) {
        self.entries.insert(0, entry);
        // Keep only the most recent entries to avoid unbounded memory growth
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Returns a copy of all log entries
    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.clone()
    }

    /// Clears all entries from the log
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of entries in the log
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the log is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for LogCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom tracing layer that writes to a LogCollector
struct LogCollectorLayer {
    collector: SharedLogCollector,
}

impl<S> Layer<S> for LogCollectorLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Extract metadata
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let target = metadata.target().to_string();

        // Extract message
        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        // Add to collector
        let entry = LogEntry::new(level, message, target);
        self.collector.lock().add_entry(entry);
    }
}

/// Visitor for extracting message from tracing events
struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0.push_str(&format!("{:?}", value));
        } else {
            self.0.push_str(&format!("{}={:?}", field.name(), value));
        }
    }
}

/// Initializes the tracing system to write to the LogCollector
pub fn init_tracing(log_collector: SharedLogCollector) {
    let layer = LogCollectorLayer {
        collector: log_collector,
    };

    tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::sink))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(
            "INFO".to_string(),
            "Test message".to_string(),
            "test_module".to_string(),
        );
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.target, "test_module");
    }

    #[test]
    fn test_log_collector_add_and_retrieve() {
        let mut collector = LogCollector::new();
        let entry = LogEntry::new(
            "INFO".to_string(),
            "Test message".to_string(),
            "test_module".to_string(),
        );
        collector.add_entry(entry.clone());
        let entries = collector.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "Test message");
    }

    #[test]
    fn test_log_collector_max_size() {
        let mut collector = LogCollector::new();
        collector.max_entries = 5;
        for i in 0..10 {
            let entry = LogEntry::new(
                "INFO".to_string(),
                format!("Message {}", i),
                "test_module".to_string(),
            );
            collector.add_entry(entry);
        }
        assert_eq!(collector.len(), 5);
    }
}