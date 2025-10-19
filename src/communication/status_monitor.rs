//! Real-time status monitoring task.
//!
//! This module provides an async task that periodically polls GRBL device status
//! using the "?" command, parses responses, and maintains a circular history buffer
//! with trend analysis capabilities.
//!
//! The monitor runs on a Tokio task and can be gracefully stopped via cancellation token.

use crate::communication::grbl_status::MachineStatus;
use crate::communication::status_parser::{parse_status_response, StatusParseError};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

/// Configuration for status monitor behavior.
#[derive(Debug, Clone)]
pub struct StatusMonitorConfig {
    /// Query interval in milliseconds (default: 250ms)
    pub query_interval_ms: u64,

    /// Maximum retries on parse failure (default: 3)
    pub max_parse_retries: u32,

    /// Enable adaptive query timing (faster during Run, slower during Idle)
    pub adaptive_timing: bool,

    /// History buffer size in samples (default: 300 = ~75 seconds @ 250ms)
    pub history_buffer_size: usize,

    /// Enable circular buffer (discard oldest when full)
    pub circular_buffer: bool,

    /// Track error patterns
    pub track_errors: bool,

    /// Maximum error patterns to track
    pub max_error_patterns: usize,
}

impl Default for StatusMonitorConfig {
    fn default() -> Self {
        StatusMonitorConfig {
            query_interval_ms: 250,
            max_parse_retries: 3,
            adaptive_timing: true,
            history_buffer_size: 300,
            circular_buffer: true,
            track_errors: true,
            max_error_patterns: 10,
        }
    }
}

/// Running status monitor instance.
///
/// Manages async polling of device status with history tracking and analytics.
pub struct StatusMonitor {
    config: StatusMonitorConfig,

    /// Current status snapshot (thread-safe)
    current_status: Arc<Mutex<MachineStatus>>,

    /// Status history (circular buffer)
    status_history: Arc<Mutex<VecDeque<MachineStatus>>>,

    /// Cancellation token for graceful shutdown
    cancel_token: CancellationToken,

    /// Monitor task handle
    task_handle: Option<tokio::task::JoinHandle<()>>,

    /// Parse error count
    parse_errors: Arc<Mutex<u32>>,

    /// Last error message
    last_error: Arc<Mutex<Option<String>>>,
}

impl StatusMonitor {
    /// Create a new status monitor with the given configuration.
    pub fn new(config: StatusMonitorConfig) -> Self {
        StatusMonitor {
            config,
            current_status: Arc::new(Mutex::new(MachineStatus::default())),
            status_history: Arc::new(Mutex::new(VecDeque::new())),
            cancel_token: CancellationToken::new(),
            task_handle: None,
            parse_errors: Arc::new(Mutex::new(0)),
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Start monitoring (spawns async task).
    ///
    /// # Arguments
    ///
    /// * `query_fn` - Function to call for device queries, returns response string
    ///
    /// # Example
    ///
    /// ```ignore
    /// monitor.start(|_| async { "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100>".to_string() }).await;
    /// ```
    pub async fn start<F, Fut>(&mut self, query_fn: F)
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = String> + Send,
    {
        let config = self.config.clone();
        let current_status = Arc::clone(&self.current_status);
        let status_history = Arc::clone(&self.status_history);
        let cancel_token = self.cancel_token.clone();
        let parse_errors = Arc::clone(&self.parse_errors);
        let last_error = Arc::clone(&self.last_error);

        let handle = tokio::spawn(async move {
            Self::monitor_loop(
                query_fn,
                config,
                current_status,
                status_history,
                cancel_token,
                parse_errors,
                last_error,
            )
            .await;
        });

        self.task_handle = Some(handle);
        debug!("Status monitor started");
    }

    /// Stop monitoring gracefully.
    pub async fn stop(&mut self) {
        self.cancel_token.cancel();

        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }

        debug!("Status monitor stopped");
    }

    /// Get current status snapshot.
    pub async fn get_current_status(&self) -> MachineStatus {
        self.current_status.lock().await.clone()
    }

    /// Get status history (last N samples).
    pub async fn get_status_history(&self, count: usize) -> Vec<MachineStatus> {
        let history = self.status_history.lock().await;
        history
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get all available history.
    pub async fn get_all_history(&self) -> Vec<MachineStatus> {
        self.status_history.lock().await.iter().cloned().collect()
    }

    /// Get history buffer size.
    pub async fn get_history_size(&self) -> usize {
        self.status_history.lock().await.len()
    }

    /// Clear history buffer.
    pub async fn clear_history(&self) {
        self.status_history.lock().await.clear();
    }

    /// Get number of parse errors.
    pub async fn get_parse_error_count(&self) -> u32 {
        *self.parse_errors.lock().await
    }

    /// Get last error message.
    pub async fn get_last_error(&self) -> Option<String> {
        self.last_error.lock().await.clone()
    }

    /// Check if monitor is running.
    pub fn is_running(&self) -> bool {
        self.task_handle.is_some() && !self.cancel_token.is_cancelled()
    }

    /// Calculate average feed rate from history.
    pub async fn get_average_feedrate(&self, samples: Option<usize>) -> f32 {
        let history = self.status_history.lock().await;

        if history.is_empty() {
            return 0.0;
        }

        let count = samples.unwrap_or_else(|| history.len());
        let take_count = count.min(history.len());

        let sum: f32 = history
            .iter()
            .rev()
            .take(take_count)
            .map(|s| s.feed_speed.feed_rate)
            .sum();

        sum / take_count as f32
    }

    /// Calculate average spindle speed from history.
    pub async fn get_average_spindle_speed(&self, samples: Option<usize>) -> f32 {
        let history = self.status_history.lock().await;

        if history.is_empty() {
            return 0.0;
        }

        let count = samples.unwrap_or_else(|| history.len());
        let take_count = count.min(history.len());

        let sum: f32 = history
            .iter()
            .rev()
            .take(take_count)
            .map(|s| s.feed_speed.spindle_speed)
            .sum();

        sum / take_count as f32
    }

    /// Get buffer fill statistics from history.
    pub async fn get_buffer_stats(&self) -> (f32, u8, u8) {
        let history = self.status_history.lock().await;

        if history.is_empty() {
            return (0.0, 0, 0);
        }

        let avg_planner: f32 = history
            .iter()
            .map(|s| s.buffer_state.planner_buffer as f32)
            .sum::<f32>()
            / history.len() as f32;

        let max_planner = history
            .iter()
            .map(|s| s.buffer_state.planner_buffer)
            .max()
            .unwrap_or(0);

        let max_rx = history
            .iter()
            .map(|s| s.buffer_state.rx_buffer)
            .max()
            .unwrap_or(0);

        (avg_planner, max_planner, max_rx)
    }

    /// The monitor loop (runs in spawned task).
    async fn monitor_loop<F, Fut>(
        query_fn: F,
        config: StatusMonitorConfig,
        current_status: Arc<Mutex<MachineStatus>>,
        status_history: Arc<Mutex<VecDeque<MachineStatus>>>,
        cancel_token: CancellationToken,
        parse_errors: Arc<Mutex<u32>>,
        last_error: Arc<Mutex<Option<String>>>,
    ) where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = String>,
    {
        let mut interval = tokio::time::interval(Duration::from_millis(config.query_interval_ms));

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    debug!("Status monitor received cancel signal");
                    break;
                }

                _ = interval.tick() => {
                    // Calculate adaptive interval if enabled
                    if config.adaptive_timing {
                        let current = current_status.lock().await.clone();
                        let new_interval = if current.is_executing() {
                            100 // Faster when running (10 FPS)
                        } else {
                            500 // Slower when idle (2 FPS)
                        };
                        interval = tokio::time::interval(Duration::from_millis(new_interval));
                    }

                    // Query device
                    let response = query_fn().await;

                    // Parse response
                    match parse_status_response(&response) {
                        Ok(status) => {
                            // Update current status
                            {
                                let mut current = current_status.lock().await;
                                *current = status.clone();
                            }

                            // Add to history
                            {
                                let mut history = status_history.lock().await;
                                history.push_back(status);

                                // Enforce circular buffer
                                if config.circular_buffer
                                    && history.len() > config.history_buffer_size
                                {
                                    history.pop_front();
                                }
                            }

                            // Clear error tracking
                            {
                                let mut errors = parse_errors.lock().await;
                                *errors = 0;
                            }
                        }
                        Err(e) => {
                            // Track parse error
                            {
                                let mut errors = parse_errors.lock().await;
                                *errors = errors.saturating_add(1);
                            }

                            let error_msg = format!("Parse error: {}", e);
                            {
                                let mut last_err = last_error.lock().await;
                                *last_err = Some(error_msg.clone());
                            }

                            warn!("{}", error_msg);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let config = StatusMonitorConfig::default();
        let monitor = StatusMonitor::new(config);

        assert!(!monitor.is_running());
    }

    #[test]
    fn test_config_default() {
        let config = StatusMonitorConfig::default();
        assert_eq!(config.query_interval_ms, 250);
        assert_eq!(config.history_buffer_size, 300);
        assert!(config.adaptive_timing);
        assert!(config.circular_buffer);
    }

    #[test]
    fn test_average_feedrate_empty() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let monitor = StatusMonitor::new(StatusMonitorConfig::default());
            let avg = monitor.get_average_feedrate(None).await;
            assert_eq!(avg, 0.0);
        });
    }

    #[test]
    fn test_history_size() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let monitor = StatusMonitor::new(StatusMonitorConfig::default());
            let size = monitor.get_history_size().await;
            assert_eq!(size, 0);
        });
    }

    #[test]
    fn test_error_tracking() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let monitor = StatusMonitor::new(StatusMonitorConfig::default());
            let errors = monitor.get_parse_error_count().await;
            assert_eq!(errors, 0);

            let last_error = monitor.get_last_error().await;
            assert!(last_error.is_none());
        });
    }
}
