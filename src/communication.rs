pub mod grbl;
pub mod smoothieware;

pub use grbl::GrblCommunication;
pub use smoothieware::SmoothiewareCommunication;

use std::any::Any;
use std::error::Error;
use std::time::Instant;

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

#[derive(Debug, Clone)]
pub struct RecoveryState {
    pub reconnect_attempts: u32,
    pub last_reconnect_attempt: Option<Instant>,
    pub command_retry_count: u32,
    pub last_error: Option<String>,
    pub recovery_actions_taken: Vec<RecoveryAction>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ControllerType {
    Grbl,
    Smoothieware,
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
}
