//! Device communication logging and filtering.
//!
//! Captures all commands sent to and responses from the GRBL device, with intelligent
//! filtering to exclude status queries ("?") and simple acknowledgments ("ok"), while
//! providing severity-based filtering for console output.

use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

/// Severity level for console messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConsoleSeverity {
    /// Error: Errors and alarms from device
    Error = 4,
    /// Warning: Warnings and recoverable issues
    Warning = 3,
    /// Info: General informational messages
    Info = 2,
    /// Debug: Debug-level tracing information
    Debug = 1,
}

impl ConsoleSeverity {
    /// Get all severity levels.
    pub fn all() -> &'static [ConsoleSeverity] {
        &[
            ConsoleSeverity::Debug,
            ConsoleSeverity::Info,
            ConsoleSeverity::Warning,
            ConsoleSeverity::Error,
        ]
    }

    /// Convert string to severity.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(ConsoleSeverity::Error),
            "warning" | "warn" => Some(ConsoleSeverity::Warning),
            "info" => Some(ConsoleSeverity::Info),
            "debug" => Some(ConsoleSeverity::Debug),
            _ => None,
        }
    }

    /// Get human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            ConsoleSeverity::Error => "ERROR",
            ConsoleSeverity::Warning => "WARN",
            ConsoleSeverity::Info => "INFO",
            ConsoleSeverity::Debug => "DEBUG",
        }
    }
}

/// Type of console message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// Command sent to device
    Command,
    /// Response received from device
    Response,
    /// Trace output from application
    Trace,
}

impl MessageType {
    /// Get human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            MessageType::Command => "CMD",
            MessageType::Response => "RES",
            MessageType::Trace => "TRC",
        }
    }
}

/// A single console message.
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    /// Timestamp when message was logged
    pub timestamp: DateTime<Utc>,
    /// Message severity
    pub severity: ConsoleSeverity,
    /// Type of message
    pub message_type: MessageType,
    /// The actual message content
    pub content: String,
    /// Whether to show in console (used for filtering)
    pub visible: bool,
}

impl ConsoleMessage {
    /// Create a new console message.
    pub fn new(
        severity: ConsoleSeverity,
        message_type: MessageType,
        content: String,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            severity,
            message_type,
            content,
            visible: true,
        }
    }

    /// Create a command message.
    pub fn command(content: String) -> Self {
        Self::new(ConsoleSeverity::Info, MessageType::Command, content)
    }

    /// Create a response message.
    pub fn response(content: String) -> Self {
        Self::new(ConsoleSeverity::Info, MessageType::Response, content)
    }

    /// Create a trace message.
    pub fn trace(severity: ConsoleSeverity, content: String) -> Self {
        Self::new(severity, MessageType::Trace, content)
    }

    /// Format message for display.
    pub fn format_display(&self) -> String {
        format!(
            "[{}] {} {}: {}",
            self.timestamp.format("%H:%M:%S%.3f"),
            self.message_type.label(),
            self.severity.label(),
            self.content
        )
    }
}

/// Device command/response logger with filtering.
pub struct DeviceLogger {
    /// Message history
    messages: Arc<Mutex<VecDeque<ConsoleMessage>>>,
    /// Maximum messages to keep in history
    max_messages: usize,
    /// Active severity filters (enabled levels)
    active_severities: Arc<Mutex<Vec<ConsoleSeverity>>>,
}

impl DeviceLogger {
    /// Create a new device logger.
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
            max_messages,
            active_severities: Arc::new(Mutex::new(vec![
                ConsoleSeverity::Error,
                ConsoleSeverity::Warning,
                ConsoleSeverity::Info,
                ConsoleSeverity::Debug,
            ])),
        }
    }

    /// Log a command sent to device (automatically filtered).
    ///
    /// Status queries ("?") are not logged.
    pub async fn log_command(&self, command: &str) {
        // Filter out status queries
        if command.trim() == "?" {
            debug!("Skipping status query log");
            return;
        }

        let message = ConsoleMessage::command(command.to_string());
        self.add_message(message).await;
    }

    /// Log a response from device (automatically filtered).
    ///
    /// Simple "ok" responses are not logged, but status responses and errors are.
    pub async fn log_response(&self, response: &str) {
        let response_trimmed = response.trim();

        // Filter out simple "ok" responses
        if response_trimmed == "ok" {
            debug!("Skipping ok response log");
            return;
        }

        // Determine severity based on response content
        let severity = if response_trimmed.starts_with("error:")
            || response_trimmed.starts_with("ALARM:")
        {
            ConsoleSeverity::Error
        } else if response_trimmed.starts_with("[MSG:")
            || response_trimmed.starts_with("$")
        {
            ConsoleSeverity::Info
        } else {
            ConsoleSeverity::Info
        };

        let message = ConsoleMessage::new(
            severity,
            MessageType::Response,
            response_trimmed.to_string(),
        );
        self.add_message(message).await;
    }

    /// Log a trace message from application.
    pub async fn log_trace(&self, severity: ConsoleSeverity, message: &str) {
        let msg = ConsoleMessage::trace(severity, message.to_string());
        self.add_message(msg).await;
    }

    /// Add a message to the log, respecting circular buffer.
    async fn add_message(&self, message: ConsoleMessage) {
        let mut messages = self.messages.lock().await;

        messages.push_back(message);

        // Enforce max size
        while messages.len() > self.max_messages {
            messages.pop_front();
        }
    }

    /// Get filtered messages (only those with active severity levels).
    pub async fn get_filtered_messages(&self) -> Vec<ConsoleMessage> {
        let messages = self.messages.lock().await;
        let active = self.active_severities.lock().await;

        messages
            .iter()
            .filter(|m| active.contains(&m.severity))
            .cloned()
            .collect()
    }

    /// Get all messages (unfiltered).
    pub async fn get_all_messages(&self) -> Vec<ConsoleMessage> {
        self.messages.lock().await.iter().cloned().collect()
    }

    /// Get formatted display strings (filtered).
    pub async fn get_display_strings(&self) -> Vec<String> {
        let messages = self.get_filtered_messages().await;
        messages.iter().map(|m| m.format_display()).collect()
    }

    /// Clear all messages.
    pub async fn clear(&self) {
        self.messages.lock().await.clear();
    }

    /// Set active severity filters.
    pub async fn set_active_severities(&self, severities: Vec<ConsoleSeverity>) {
        let mut active = self.active_severities.lock().await;
        *active = severities;
    }

    /// Check if severity is active.
    pub async fn is_severity_active(&self, severity: ConsoleSeverity) -> bool {
        self.active_severities
            .lock()
            .await
            .contains(&severity)
    }

    /// Get current active severity filters.
    pub async fn get_active_severities(&self) -> Vec<ConsoleSeverity> {
        self.active_severities.lock().await.clone()
    }

    /// Get message count by severity.
    pub async fn count_by_severity(&self) -> std::collections::HashMap<ConsoleSeverity, usize> {
        let messages = self.messages.lock().await;
        let mut counts = std::collections::HashMap::new();

        for msg in messages.iter() {
            *counts.entry(msg.severity).or_insert(0) += 1;
        }

        counts
    }

    /// Get total message count.
    pub async fn total_count(&self) -> usize {
        self.messages.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_severity_ordering() {
        assert!(ConsoleSeverity::Error > ConsoleSeverity::Warning);
        assert!(ConsoleSeverity::Warning > ConsoleSeverity::Info);
        assert!(ConsoleSeverity::Info > ConsoleSeverity::Debug);
    }

    #[test]
    fn test_console_severity_from_str() {
        assert_eq!(
            ConsoleSeverity::from_str("error"),
            Some(ConsoleSeverity::Error)
        );
        assert_eq!(
            ConsoleSeverity::from_str("WARNING"),
            Some(ConsoleSeverity::Warning)
        );
        assert_eq!(
            ConsoleSeverity::from_str("info"),
            Some(ConsoleSeverity::Info)
        );
        assert_eq!(
            ConsoleSeverity::from_str("debug"),
            Some(ConsoleSeverity::Debug)
        );
        assert_eq!(ConsoleSeverity::from_str("invalid"), None);
    }

    #[test]
    fn test_message_type_labels() {
        assert_eq!(MessageType::Command.label(), "CMD");
        assert_eq!(MessageType::Response.label(), "RES");
        assert_eq!(MessageType::Trace.label(), "TRC");
    }

    #[tokio::test]
    async fn test_logger_creation() {
        let logger = DeviceLogger::new(100);
        assert_eq!(logger.total_count().await, 0);
    }

    #[tokio::test]
    async fn test_log_command() {
        let logger = DeviceLogger::new(100);

        logger.log_command("G0 X10 Y20").await;

        let messages = logger.get_all_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message_type, MessageType::Command);
        assert_eq!(messages[0].content, "G0 X10 Y20");
    }

    #[tokio::test]
    async fn test_filter_status_query() {
        let logger = DeviceLogger::new(100);

        logger.log_command("?").await;

        let messages = logger.get_all_messages().await;
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_log_response() {
        let logger = DeviceLogger::new(100);

        logger.log_response("ok").await;

        let messages = logger.get_all_messages().await;
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_log_error_response() {
        let logger = DeviceLogger::new(100);

        logger.log_response("error:1 - G-code words consist of a letter and a value").await;

        let messages = logger.get_all_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].severity, ConsoleSeverity::Error);
    }

    #[tokio::test]
    async fn test_severity_filtering() {
        let logger = DeviceLogger::new(100);

        logger.log_trace(ConsoleSeverity::Error, "Error message").await;
        logger.log_trace(ConsoleSeverity::Warning, "Warning message").await;
        logger.log_trace(ConsoleSeverity::Info, "Info message").await;
        logger.log_trace(ConsoleSeverity::Debug, "Debug message").await;

        assert_eq!(logger.total_count().await, 4);

        // Only show error and warning
        logger
            .set_active_severities(vec![ConsoleSeverity::Error, ConsoleSeverity::Warning])
            .await;

        let filtered = logger.get_filtered_messages().await;
        assert_eq!(filtered.len(), 2);
    }

    #[tokio::test]
    async fn test_circular_buffer() {
        let logger = DeviceLogger::new(5);

        for i in 0..10 {
            logger.log_trace(ConsoleSeverity::Info, &format!("Message {}", i)).await;
        }

        let messages = logger.get_all_messages().await;
        assert_eq!(messages.len(), 5);
        assert!(messages[0].content.contains("5"));
        assert!(messages[4].content.contains("9"));
    }

    #[tokio::test]
    async fn test_count_by_severity() {
        let logger = DeviceLogger::new(100);

        logger.log_trace(ConsoleSeverity::Error, "Error 1").await;
        logger.log_trace(ConsoleSeverity::Error, "Error 2").await;
        logger.log_trace(ConsoleSeverity::Warning, "Warning 1").await;

        let counts = logger.count_by_severity().await;
        assert_eq!(counts.get(&ConsoleSeverity::Error), Some(&2));
        assert_eq!(counts.get(&ConsoleSeverity::Warning), Some(&1));
    }
}
