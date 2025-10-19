//! Status manager - orchestrates real-time monitoring and UI updates.
//!
//! This module provides the main orchestration layer that connects the status
//! monitor with the UI components, handling real-time updates, configuration,
//! and state management for the status display system.

use crate::communication::status_monitor::StatusMonitor;
use crate::communication::grbl_status::MachineStatus;
use std::sync::Arc;

/// Configuration for the status manager.
#[derive(Debug, Clone)]
pub struct StatusManagerConfig {
    /// Enable real-time monitoring
    pub enabled: bool,
    /// Query interval in milliseconds
    pub query_interval_ms: u64,
    /// Maximum history samples to keep
    pub max_history: usize,
    /// Enable chart caching
    pub cache_charts: bool,
    /// Chart cache update interval (ms)
    pub cache_update_interval_ms: u64,
    /// Auto-clear history after inactivity (seconds)
    pub auto_clear_after_secs: Option<u64>,
    /// Theme color (theme name or hex)
    pub theme: String,
}

impl Default for StatusManagerConfig {
    fn default() -> Self {
        StatusManagerConfig {
            enabled: true,
            query_interval_ms: 250,
            max_history: 300,
            cache_charts: true,
            cache_update_interval_ms: 100,
            auto_clear_after_secs: Some(3600), // 1 hour
            theme: "dark".to_string(),
        }
    }
}

impl StatusManagerConfig {
    /// Create new config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable monitoring.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set query interval.
    pub fn with_query_interval(mut self, ms: u64) -> Self {
        self.query_interval_ms = ms.clamp(10, 5000);
        self
    }

    /// Set maximum history size.
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max.clamp(10, 10000);
        self
    }

    /// Enable or disable chart caching.
    pub fn with_cache_charts(mut self, cache: bool) -> Self {
        self.cache_charts = cache;
        self
    }

    /// Set theme.
    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }
}

/// Manages real-time status monitoring and UI updates.
pub struct StatusManager {
    /// Configuration
    config: StatusManagerConfig,
    /// Status monitor
    monitor: Arc<StatusMonitor>,
    /// Whether manager is running
    is_running: bool,
}

impl StatusManager {
    /// Create new status manager.
    pub fn new(config: StatusManagerConfig) -> Self {
        let monitor_config = crate::communication::status_monitor::StatusMonitorConfig {
            query_interval_ms: config.query_interval_ms,
            max_parse_retries: 3,
            adaptive_timing: true,
            history_buffer_size: config.max_history,
            circular_buffer: true,
            track_errors: true,
            max_error_patterns: 10,
        };

        StatusManager {
            config,
            monitor: Arc::new(StatusMonitor::new(monitor_config)),
            is_running: false,
        }
    }

    /// Get configuration.
    pub fn config(&self) -> &StatusManagerConfig {
        &self.config
    }

    /// Get mutable configuration (requires restart).
    pub fn config_mut(&mut self) -> &mut StatusManagerConfig {
        &mut self.config
    }

    /// Check if manager is running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Start monitoring with query function (must be async closure).
    pub fn get_monitor(&self) -> Arc<StatusMonitor> {
        Arc::clone(&self.monitor)
    }

    /// Check if manager is initialized.
    pub fn is_initialized(&self) -> bool {
        true
    }

    /// Get current status.
    pub fn get_current_status(&self) -> Option<MachineStatus> {
        // Note: This would need to be called from async context
        // For now, return None - the UI should get data from monitor directly
        None
    }

    /// Get status history (limited to count).
    pub fn get_status_history(&self, count: usize) -> Vec<MachineStatus> {
        // Note: This would need to be called from async context
        // For now, return empty - the UI should get data from monitor directly
        Vec::new()
    }

    /// Get all history.
    pub fn get_all_history(&self) -> Vec<MachineStatus> {
        // Note: This would need to be called from async context
        // For now, return empty - the UI should get data from monitor directly
        Vec::new()
    }

    /// Clear history.
    /// Note: History clearing requires async context, so this is handled
    /// through the monitor's async methods when needed.
    pub fn clear_history(&self) {
        // History is cleared through monitor.clear_history() in async context
        // This method provides a sync interface for the type system
    }

    /// Get manager statistics.
    pub fn get_stats(&self) -> ManagerStats {
        ManagerStats {
            is_running: self.is_running,
            config_theme: self.config.theme.clone(),
            max_history: self.config.max_history,
            cache_enabled: self.config.cache_charts,
        }
    }
}

/// Manager statistics.
#[derive(Debug, Clone)]
pub struct ManagerStats {
    /// Whether monitoring is running
    pub is_running: bool,
    /// Current theme
    pub config_theme: String,
    /// Maximum history samples
    pub max_history: usize,
    /// Whether chart caching is enabled
    pub cache_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = StatusManagerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.query_interval_ms, 250);
        assert_eq!(config.max_history, 300);
        assert!(config.cache_charts);
    }

    #[test]
    fn test_config_with_enabled() {
        let config = StatusManagerConfig::new().with_enabled(false);
        assert!(!config.enabled);
    }

    #[test]
    fn test_config_with_query_interval() {
        let config = StatusManagerConfig::new().with_query_interval(100);
        assert_eq!(config.query_interval_ms, 100);
    }

    #[test]
    fn test_config_interval_bounds() {
        let config = StatusManagerConfig::new()
            .with_query_interval(1)
            .with_query_interval(10000);
        assert!(config.query_interval_ms >= 10);
        assert!(config.query_interval_ms <= 5000);
    }

    #[test]
    fn test_config_with_max_history() {
        let config = StatusManagerConfig::new().with_max_history(500);
        assert_eq!(config.max_history, 500);
    }

    #[test]
    fn test_config_history_bounds() {
        let config = StatusManagerConfig::new()
            .with_max_history(5)
            .with_max_history(50000);
        assert!(config.max_history >= 10);
        assert!(config.max_history <= 10000);
    }

    #[test]
    fn test_config_with_cache_charts() {
        let config = StatusManagerConfig::new().with_cache_charts(false);
        assert!(!config.cache_charts);
    }

    #[test]
    fn test_config_with_theme() {
        let config = StatusManagerConfig::new().with_theme("light");
        assert_eq!(config.theme, "light");
    }

    #[test]
    fn test_manager_creation() {
        let config = StatusManagerConfig::new();
        let manager = StatusManager::new(config);
        assert!(!manager.is_running);
    }

    #[test]
    fn test_manager_config_access() {
        let config = StatusManagerConfig::new().with_query_interval(300);
        let manager = StatusManager::new(config);
        assert_eq!(manager.config().query_interval_ms, 300);
    }

    #[test]
    fn test_manager_get_stats() {
        let config = StatusManagerConfig::new().with_max_history(500);
        let manager = StatusManager::new(config);
        let stats = manager.get_stats();
        assert!(!stats.is_running);
        assert_eq!(stats.max_history, 500);
    }
}

