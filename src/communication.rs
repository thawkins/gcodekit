pub mod fluidnc;
pub mod g2core;
pub mod grbl;
pub mod smoothieware;
pub mod tinyg;

pub use fluidnc::FluidNCCommunication;
pub use g2core::G2coreCommunication;
pub use grbl::GrblCommunication;
pub use smoothieware::SmoothiewareCommunication;

use std::time::{Duration, Instant};

use std::any::Any;
use std::error::Error;

#[derive(Default, PartialEq, Debug, Clone)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
    Recovering,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    Reconnect,
    RetryCommand,
    ResetController,
    SkipCommand,
    AbortJob,
}

#[derive(Debug, Clone)]
pub struct ErrorRecoveryConfig {
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    pub max_command_retries: u32,
    pub command_retry_delay_ms: u64,
    pub reset_on_critical_error: bool,
    pub auto_recovery_enabled: bool,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            max_reconnect_attempts: 3,
            reconnect_delay_ms: 2000,
            max_command_retries: 3,
            command_retry_delay_ms: 1000,
            reset_on_critical_error: true,
            auto_recovery_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecoveryState {
    pub reconnect_attempts: u32,
    pub last_reconnect_attempt: Option<Instant>,
    pub command_retry_count: u32,
    pub last_error: Option<String>,
    pub recovery_actions_taken: Vec<RecoveryAction>,
}

#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub error_type: String,
    pub frequency: u32,
    pub last_occurrence: Instant,
    pub average_time_between: Duration,
    pub severity_score: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub connection_stability: f32, // 0.0 to 1.0
    pub command_success_rate: f32, // 0.0 to 1.0
    pub average_response_time: Duration,
    pub error_patterns: Vec<ErrorPattern>,
    pub uptime_percentage: f32, // 0.0 to 1.0
    pub last_health_check: Instant,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            connection_stability: 1.0,
            command_success_rate: 1.0,
            average_response_time: Duration::from_millis(100),
            error_patterns: Vec::new(),
            uptime_percentage: 1.0,
            last_health_check: Instant::now(),
        }
    }
}

impl HealthMetrics {
    pub fn update_error_pattern(&mut self, error: &str) {
        let now = Instant::now();
        let error_type = error.to_string();

        // Find existing pattern or create new one
        if let Some(pattern) = self
            .error_patterns
            .iter_mut()
            .find(|p| p.error_type == error_type)
        {
            let time_since_last = now.duration_since(pattern.last_occurrence);
            pattern.average_time_between = (pattern.average_time_between + time_since_last) / 2;
            pattern.frequency += 1;
            pattern.last_occurrence = now;

            // Increase severity if errors are happening more frequently
            if time_since_last < Duration::from_secs(60) {
                pattern.severity_score = (pattern.severity_score + 0.1).min(1.0);
            }
        } else {
            self.error_patterns.push(ErrorPattern {
                error_type,
                frequency: 1,
                last_occurrence: now,
                average_time_between: Duration::from_secs(3600), // Default 1 hour
                severity_score: 0.1,
            });
        }

        // Update overall health metrics
        self.update_health_scores();
    }

    pub fn predict_potential_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for frequent connection errors
        if let Some(conn_pattern) = self
            .error_patterns
            .iter()
            .find(|p| p.error_type.contains("connection") || p.error_type.contains("timeout"))
        {
            if conn_pattern.frequency > 5 && conn_pattern.severity_score > 0.5 {
                issues.push("High frequency of connection errors detected. Consider checking network stability.".to_string());
            }
        }

        // Check for command errors
        if let Some(cmd_pattern) = self
            .error_patterns
            .iter()
            .find(|p| p.error_type.contains("command") || p.error_type.contains("syntax"))
        {
            if cmd_pattern.frequency > 3 {
                issues.push(
                    "Frequent command errors detected. G-code may need validation.".to_string(),
                );
            }
        }

        // Check connection stability
        if self.connection_stability < 0.8 {
            issues.push(
                "Connection stability is low. Consider checking hardware connections.".to_string(),
            );
        }

        // Check command success rate
        if self.command_success_rate < 0.9 {
            issues.push("Command success rate is low. System may need maintenance.".to_string());
        }

        issues
    }

    fn update_health_scores(&mut self) {
        let now = Instant::now();
        let time_window = Duration::from_secs(3600); // Last hour

        // Calculate connection stability based on recent errors
        let recent_conn_errors = self
            .error_patterns
            .iter()
            .filter(|p| {
                (p.error_type.contains("connection") || p.error_type.contains("timeout"))
                    && now.duration_since(p.last_occurrence) < time_window
            })
            .map(|p| p.frequency)
            .sum::<u32>();

        self.connection_stability = (100.0 - recent_conn_errors as f32 * 5.0).max(0.0) / 100.0;

        // Calculate command success rate (simplified - would need actual command tracking)
        let recent_cmd_errors = self
            .error_patterns
            .iter()
            .filter(|p| {
                p.error_type.contains("command")
                    && now.duration_since(p.last_occurrence) < time_window
            })
            .map(|p| p.frequency)
            .sum::<u32>();

        self.command_success_rate = (100.0 - recent_cmd_errors as f32 * 2.0).max(0.0) / 100.0;

        self.last_health_check = now;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ControllerType {
    Grbl,
    Smoothieware,
    TinyG,
    G2core,
    FluidNC,
}

pub trait CncController {
    fn as_any(&self) -> &dyn Any;
    fn set_port(&mut self, port: String);
    fn connect(&mut self) -> Result<(), Box<dyn Error>>;
    fn disconnect(&mut self);
    fn send_gcode_line(&mut self, line: &str) -> Result<(), Box<dyn Error>>;
    fn read_response(&mut self) -> Option<String>;
    fn is_connected(&self) -> bool;
    fn get_status(&self) -> String;
    fn refresh_ports(&mut self);
    fn get_available_ports(&self) -> &Vec<String>;
    fn get_selected_port(&self) -> &str;
    fn get_connection_state(&self) -> &ConnectionState;
    fn get_status_message(&self) -> &str;
    fn jog_axis(&mut self, axis: char, distance: f32);
    fn home_all_axes(&mut self);
    fn emergency_stop(&mut self);
    fn send_spindle_override(&mut self, percentage: f32);
    fn send_feed_override(&mut self, percentage: f32);
    fn get_version(&self) -> &str;
    fn handle_response(&mut self, response: &str) -> Option<crate::MachinePosition>;

    // Error recovery methods
    fn get_recovery_config(&self) -> &ErrorRecoveryConfig;
    fn get_recovery_state(&self) -> &RecoveryState;
    fn set_recovery_config(&mut self, config: ErrorRecoveryConfig);
    fn attempt_recovery(&mut self, error: &str) -> Result<RecoveryAction, String>;
    fn reset_recovery_state(&mut self);
    fn is_recovering(&self) -> bool;

    // Advanced health monitoring and predictive error prevention
    fn get_health_metrics(&self) -> &HealthMetrics;
    fn get_health_metrics_mut(&mut self) -> &mut HealthMetrics;
    fn perform_health_check(&mut self) -> Vec<String>; // Returns warnings/issues
    fn optimize_settings_based_on_health(&mut self) -> Vec<String>; // Returns applied optimizations
}
