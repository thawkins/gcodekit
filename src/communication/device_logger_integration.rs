//! Integration layer between device communication and console logging.
//!
//! This module provides utilities for routing device commands and responses
//! to the console logger with automatic filtering.

use crate::communication::{DeviceLogger, ConsoleSeverity};
use std::sync::Arc;
use tracing::Span;

/// Helper to log a command with automatic "?" filtering.
pub async fn log_device_command(logger: &Arc<DeviceLogger>, command: &str) {
    logger.log_command(command).await;
}

/// Helper to log a response with automatic "ok" filtering.
pub async fn log_device_response(logger: &Arc<DeviceLogger>, response: &str) {
    logger.log_response(response).await;
}

/// Helper to log a trace message.
pub async fn log_trace_message(
    logger: &Arc<DeviceLogger>,
    severity: ConsoleSeverity,
    message: &str,
) {
    logger.log_trace(severity, message).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_device_command_helper() {
        let logger = Arc::new(DeviceLogger::new(100));
        log_device_command(&logger, "G0 X10").await;
        assert_eq!(logger.total_count().await, 1);
    }

    #[tokio::test]
    async fn test_log_device_response_helper() {
        let logger = Arc::new(DeviceLogger::new(100));
        log_device_response(&logger, "[MSG:test]").await;
        assert_eq!(logger.total_count().await, 1);
    }

    #[tokio::test]
    async fn test_log_trace_message_helper() {
        let logger = Arc::new(DeviceLogger::new(100));
        log_trace_message(&logger, ConsoleSeverity::Info, "Test trace").await;
        assert_eq!(logger.total_count().await, 1);
    }
}
