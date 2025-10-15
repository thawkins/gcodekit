use serialport::{SerialPort, available_ports};
use std::any::Any;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{Read, Write};
use tracing::info;

use super::{CncController, ConnectionState};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MachineState {
    Idle,
    Run,
    Hold,
    Jog,
    Alarm,
    Door,
    Check,
    Home,
    Sleep,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum WcsCoordinate {
    #[default]
    G54,
    G55,
    G56,
    G57,
    G58,
    G59,
}

impl From<&str> for MachineState {
    fn from(s: &str) -> Self {
        match s {
            "Idle" => MachineState::Idle,
            "Run" => MachineState::Run,
            "Hold" => MachineState::Hold,
            "Jog" => MachineState::Jog,
            "Alarm" => MachineState::Alarm,
            "Door" => MachineState::Door,
            "Check" => MachineState::Check,
            "Home" => MachineState::Home,
            "Sleep" => MachineState::Sleep,
            _ => MachineState::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GrblResponse {
    Ok,
    Error(String),
    Status {
        state: MachineState,
        x: f32,
        y: f32,
        z: f32,
        feed: f32,
        spindle: f32,
    },
    Feedback(String),
    Version(String),
    Settings(String),
    Other(String),
}

pub struct SmoothiewareCommunication {
    pub selected_port: String,
    pub available_ports: Vec<String>,
    pub connection_state: ConnectionState,
    pub status_message: String,
    pub machine_state: MachineState,
    pub current_position: (f32, f32, f32),
    pub current_wcs: WcsCoordinate,
    pub grbl_version: String,
    pub recovery_config: crate::communication::ErrorRecoveryConfig,
    pub recovery_state: crate::communication::RecoveryState,
    pub health_metrics: crate::communication::HealthMetrics,
    serial_port: Option<Box<dyn SerialPort>>,
    response_queue: VecDeque<String>,
}

impl Default for SmoothiewareCommunication {
    fn default() -> Self {
        Self {
            selected_port: String::new(),
            available_ports: Vec::new(),
            connection_state: ConnectionState::Disconnected,
            status_message: "Disconnected".to_string(),
            machine_state: MachineState::Unknown,
            current_position: (0.0, 0.0, 0.0),
            current_wcs: WcsCoordinate::G54,
            grbl_version: String::new(),
            recovery_config: crate::communication::ErrorRecoveryConfig::default(),
            recovery_state: crate::communication::RecoveryState {
                reconnect_attempts: 0,
                last_reconnect_attempt: None,
                command_retry_count: 0,
                last_error: None,
                recovery_actions_taken: Vec::new(),
            },
            health_metrics: crate::communication::HealthMetrics::default(),
            serial_port: None,
            response_queue: VecDeque::new(),
        }
    }
}

impl SmoothiewareCommunication {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refresh_ports(&mut self) {
        self.available_ports.clear();
        match available_ports() {
            Ok(ports) => {
                for port in ports {
                    self.available_ports.push(port.port_name);
                }
            }
            Err(e) => {
                self.status_message = format!("Error listing ports: {}", e);
            }
        }
    }

    pub fn connect_to_device(&mut self) {
        if self.selected_port.is_empty() {
            self.status_message = "No port selected".to_string();
            return;
        }

        self.connection_state = ConnectionState::Connecting;
        self.status_message = format!("Connecting to {}...", self.selected_port);

        match serialport::new(&self.selected_port, 115200)
            .timeout(std::time::Duration::from_millis(100))
            .open()
        {
            Ok(port) => {
                self.serial_port = Some(port);
                self.connection_state = ConnectionState::Connected;
                self.status_message = format!("Connected to {}", self.selected_port);
                // Send version query
                if let Err(e) = self.send_command("version") {
                    self.status_message = format!("Error querying version: {}", e);
                }
            }
            Err(e) => {
                self.connection_state = ConnectionState::Error;
                self.status_message = format!("Connection failed: {}", e);
            }
        }
    }

    pub fn disconnect_from_device(&mut self) {
        self.serial_port = None;
        self.connection_state = ConnectionState::Disconnected;
        self.status_message = "Disconnected".to_string();
        self.machine_state = MachineState::Unknown;
        self.current_position = (0.0, 0.0, 0.0);
    }

    pub fn send_gcode_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".into());
        }

        let command = format!("{}\n", line.trim());
        self.send_command(&command)?;
        Ok(())
    }

    pub fn send_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut port) = self.serial_port {
            port.write_all(command.as_bytes())?;
            port.flush()?;
            Ok(())
        } else {
            Err("Not connected".into())
        }
    }

    pub fn read_smoothieware_responses(&mut self) -> Vec<String> {
        let mut messages = Vec::new();

        if let Some(ref mut port) = self.serial_port {
            let mut buffer = [0u8; 1024];
            match port.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                        for line in data.lines() {
                            if !line.is_empty() {
                                messages.push(line.to_string());
                                self.response_queue.push_back(line.to_string());
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Timeout is expected
                }
                Err(e) => {
                    self.status_message = format!("Read error: {}", e);
                    self.connection_state = ConnectionState::Error;
                }
            }
        }

        messages
    }

    pub fn parse_smoothieware_response(&mut self, response: &str) -> GrblResponse {
        let response = response.trim();

        if response.starts_with("ok") {
            GrblResponse::Ok
        } else if response.starts_with("error:") {
            GrblResponse::Error(response[6..].to_string())
        } else if response.starts_with("<") && response.ends_with(">") {
            // Status response: <Idle|MPos:0.000,0.000,0.000|FS:0,0>
            let content = &response[1..response.len() - 1];
            let parts: Vec<&str> = content.split('|').collect();

            if !parts.is_empty() {
                let state = MachineState::from(parts[0]);

                if state == MachineState::Unknown {
                    GrblResponse::Other(response.to_string())
                } else {
                    let mut x = 0.0;
                    let mut y = 0.0;
                    let mut z = 0.0;
                    let mut feed = 0.0;
                    let mut spindle = 0.0;

                    for part in &parts[1..] {
                        if part.starts_with("MPos:") {
                            let coords: Vec<&str> = part[5..].split(',').collect();
                            if coords.len() >= 3 {
                                x = coords[0].parse().unwrap_or(0.0);
                                y = coords[1].parse().unwrap_or(0.0);
                                z = coords[2].parse().unwrap_or(0.0);
                            }
                        } else if part.starts_with("FS:") {
                            let fs: Vec<&str> = part[3..].split(',').collect();
                            if fs.len() >= 2 {
                                feed = fs[0].parse().unwrap_or(0.0);
                                spindle = fs[1].parse().unwrap_or(0.0);
                            }
                        }
                    }

                    self.machine_state = state.clone();
                    self.current_position = (x, y, z);

                    GrblResponse::Status {
                        state,
                        x,
                        y,
                        z,
                        feed,
                        spindle,
                    }
                }
            } else {
                GrblResponse::Other(response.to_string())
            }
        } else if response.starts_with("[MSG:") {
            GrblResponse::Feedback(response[5..response.len() - 1].to_string())
        } else if response.starts_with("Smoothieware") {
            self.grbl_version = response.to_string();
            GrblResponse::Version(response.to_string())
        } else {
            GrblResponse::Other(response.to_string())
        }
    }

    pub fn query_realtime_status(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("?")
    }

    pub fn get_smoothieware_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("$$")
    }

    pub fn set_smoothieware_setting(
        &mut self,
        setting: &str,
        value: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(&format!("${}={}", setting, value))
    }

    pub fn feed_hold(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("!")
    }

    pub fn resume(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("~")
    }

    pub fn reset_smoothieware(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command("\x18") // Ctrl+X
    }

    pub fn jog_axis(&mut self, axis: char, distance: f32) {
        if self.connection_state != ConnectionState::Connected {
            self.status_message = "Not connected".to_string();
            return;
        }

        let feed_rate = 1000.0; // Default jog feed rate
        let command = format!("G91 G0 {}{} F{} G90", axis, distance, feed_rate);

        if let Err(e) = self.send_gcode_line(&command) {
            self.status_message = format!("Jog error: {}", e);
        } else {
            self.status_message = format!("Jogging {} by {}", axis, distance);
        }
    }

    pub fn home_all_axes(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            self.status_message = "Not connected".to_string();
            return;
        }

        if let Err(e) = self.send_gcode_line("G28") {
            self.status_message = format!("Home error: {}", e);
        } else {
            self.status_message = "Homing all axes".to_string();
        }
    }

    pub fn send_spindle_override(&mut self, override_percent: f32) {
        if self.connection_state != ConnectionState::Connected {
            self.status_message = "Not connected".to_string();
            return;
        }

        // Smoothieware spindle override command
        let command = format!("M221 S{}", override_percent);
        if let Err(e) = self.send_gcode_line(&command) {
            self.status_message = format!("Spindle override error: {}", e);
        } else {
            self.status_message = format!("Spindle override: {}%", override_percent);
        }
    }

    pub fn send_feed_override(&mut self, override_percent: f32) {
        if self.connection_state != ConnectionState::Connected {
            self.status_message = "Not connected".to_string();
            return;
        }

        // Smoothieware feed override command
        let command = format!("M220 S{}", override_percent);
        if let Err(e) = self.send_gcode_line(&command) {
            self.status_message = format!("Feed override error: {}", e);
        } else {
            self.status_message = format!("Feed override: {}%", override_percent);
        }
    }
}

impl CncController for SmoothiewareCommunication {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn set_port(&mut self, port: String) {
        self.selected_port = port;
    }

    fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.connect_to_device();
        Ok(())
    }

    fn disconnect(&mut self) {
        self.disconnect_from_device();
    }

    fn send_gcode_line(&mut self, line: &str) -> Result<(), Box<dyn Error>> {
        self.send_gcode_line(line)
    }

    fn read_response(&mut self) -> Option<String> {
        self.read_smoothieware_responses().first().cloned()
    }

    fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    fn get_status(&self) -> String {
        format!("{:?}", self.connection_state)
    }

    fn refresh_ports(&mut self) {
        self.refresh_ports();
    }

    fn get_available_ports(&self) -> &Vec<String> {
        &self.available_ports
    }

    fn get_selected_port(&self) -> &str {
        &self.selected_port
    }

    fn get_connection_state(&self) -> &ConnectionState {
        &self.connection_state
    }

    fn get_status_message(&self) -> &str {
        &self.status_message
    }

    fn jog_axis(&mut self, axis: char, distance: f32) {
        self.jog_axis(axis, distance);
    }

    fn home_all_axes(&mut self) {
        self.home_all_axes();
    }

    fn send_spindle_override(&mut self, percentage: f32) {
        self.send_spindle_override(percentage);
    }

    fn send_feed_override(&mut self, percentage: f32) {
        self.send_feed_override(percentage);
    }

    fn get_version(&self) -> &str {
        &self.grbl_version
    }

    fn handle_response(&mut self, _response: &str) -> Option<crate::MachinePosition> {
        // For now, do nothing. Could parse Smoothieware responses in the future.
        None
    }

    // Error recovery methods
    fn get_recovery_config(&self) -> &crate::communication::ErrorRecoveryConfig {
        &self.recovery_config
    }

    fn get_recovery_state(&self) -> &crate::communication::RecoveryState {
        &self.recovery_state
    }

    fn set_recovery_config(&mut self, config: crate::communication::ErrorRecoveryConfig) {
        self.recovery_config = config;
    }

    fn attempt_recovery(
        &mut self,
        error: &str,
    ) -> Result<crate::communication::RecoveryAction, String> {
        if !self.recovery_config.auto_recovery_enabled {
            info!("[RECOVERY] Auto recovery disabled for error: {}", error);
            return Err("Auto recovery disabled".to_string());
        }

        self.recovery_state.last_error = Some(error.to_string());
        self.health_metrics.update_error_pattern(error);
        info!("[RECOVERY] Attempting recovery for error: {}", error);

        // Classify error and determine recovery action
        let action = if error.contains("connection") || error.contains("timeout") {
            // Connection-related errors
            info!(
                "[RECOVERY] Classified as connection error (attempts: {}/{})",
                self.recovery_state.reconnect_attempts, self.recovery_config.max_reconnect_attempts
            );
            if self.recovery_state.reconnect_attempts < self.recovery_config.max_reconnect_attempts
            {
                self.recovery_state.reconnect_attempts += 1;
                self.recovery_state.last_reconnect_attempt = Some(std::time::Instant::now());
                self.connection_state = ConnectionState::Recovering;
                info!(
                    "[RECOVERY] Initiating reconnection attempt {}",
                    self.recovery_state.reconnect_attempts
                );
                crate::communication::RecoveryAction::Reconnect
            } else {
                info!("[RECOVERY] Max reconnection attempts reached, aborting job");
                crate::communication::RecoveryAction::AbortJob
            }
        } else if error.contains("command") || error.contains("syntax") {
            // Command-related errors
            info!(
                "[RECOVERY] Classified as command error (retries: {}/{})",
                self.recovery_state.command_retry_count, self.recovery_config.max_command_retries
            );
            if self.recovery_state.command_retry_count < self.recovery_config.max_command_retries {
                self.recovery_state.command_retry_count += 1;
                info!(
                    "[RECOVERY] Retrying command (attempt {})",
                    self.recovery_state.command_retry_count
                );
                crate::communication::RecoveryAction::RetryCommand
            } else {
                info!("[RECOVERY] Max command retries reached, skipping command");
                crate::communication::RecoveryAction::SkipCommand
            }
        } else if error.contains("alarm") || error.contains("emergency") {
            // Critical errors
            info!(
                "[RECOVERY] Classified as critical error (reset_on_critical: {})",
                self.recovery_config.reset_on_critical_error
            );
            if self.recovery_config.reset_on_critical_error {
                info!("[RECOVERY] Resetting controller due to critical error");
                crate::communication::RecoveryAction::ResetController
            } else {
                info!("[RECOVERY] Aborting job due to critical error");
                crate::communication::RecoveryAction::AbortJob
            }
        } else {
            // Unknown errors - try reset
            info!("[RECOVERY] Classified as unknown error, attempting controller reset");
            crate::communication::RecoveryAction::ResetController
        };

        self.recovery_state
            .recovery_actions_taken
            .push(action.clone());
        info!("[RECOVERY] Recovery action taken: {:?}", action);
        Ok(action)
    }

    fn reset_recovery_state(&mut self) {
        self.recovery_state = crate::communication::RecoveryState {
            reconnect_attempts: 0,
            last_reconnect_attempt: None,
            command_retry_count: 0,
            last_error: None,
            recovery_actions_taken: Vec::new(),
        };
    }

    fn is_recovering(&self) -> bool {
        self.connection_state == ConnectionState::Recovering
            || self.recovery_state.reconnect_attempts > 0
    }

    fn get_health_metrics(&self) -> &crate::communication::HealthMetrics {
        &self.health_metrics
    }

    fn get_health_metrics_mut(&mut self) -> &mut crate::communication::HealthMetrics {
        &mut self.health_metrics
    }

    fn perform_health_check(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();
        self.health_metrics.update_health_scores();
        warnings.extend(self.health_metrics.predict_potential_issues());
        warnings
    }

    fn optimize_settings_based_on_health(&mut self) -> Vec<String> {
        Vec::new() // Basic implementation
    }

    fn emergency_stop(&mut self) {
        // Send emergency stop command to Smoothieware
        let _ = self.send_gcode_line("M112 ; Emergency stop");
    }

    fn send_raw_command(&mut self, command: &str) {
        let _ = self.send_command(command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoothieware_communication_new() {
        let comm = SmoothiewareCommunication::new();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert!(comm.selected_port.is_empty());
        assert!(comm.available_ports.is_empty());
        assert_eq!(comm.status_message, "Disconnected");
        assert_eq!(comm.machine_state, MachineState::Unknown);
        assert_eq!(comm.current_position, (0.0, 0.0, 0.0));
        assert!(comm.grbl_version.is_empty());
        assert!(comm.serial_port.is_none());
    }

    #[test]
    fn test_machine_state_from_string() {
        assert_eq!(MachineState::from("Idle"), MachineState::Idle);
        assert_eq!(MachineState::from("Run"), MachineState::Run);
        assert_eq!(MachineState::from("Hold"), MachineState::Hold);
        assert_eq!(MachineState::from("Jog"), MachineState::Jog);
        assert_eq!(MachineState::from("Alarm"), MachineState::Alarm);
        assert_eq!(MachineState::from("Door"), MachineState::Door);
        assert_eq!(MachineState::from("Check"), MachineState::Check);
        assert_eq!(MachineState::from("Home"), MachineState::Home);
        assert_eq!(MachineState::from("Sleep"), MachineState::Sleep);
        assert_eq!(MachineState::from("UnknownState"), MachineState::Unknown);
    }

    #[test]
    fn test_parse_smoothieware_response_ok() {
        let mut comm = SmoothiewareCommunication::new();
        let response = comm.parse_smoothieware_response("ok");
        assert!(matches!(response, GrblResponse::Ok));
    }

    #[test]
    fn test_parse_smoothieware_response_error() {
        let mut comm = SmoothiewareCommunication::new();
        let response = comm.parse_smoothieware_response("error: Invalid command");
        assert!(matches!(response, GrblResponse::Error(_)));
        if let GrblResponse::Error(msg) = response {
            assert_eq!(msg, " Invalid command");
        }
    }

    #[test]
    fn test_parse_smoothieware_response_status() {
        let mut comm = SmoothiewareCommunication::new();
        let response =
            comm.parse_smoothieware_response("<Idle|MPos:10.000,20.000,30.000|FS:1000,12000>");

        match response {
            GrblResponse::Status {
                state,
                x,
                y,
                z,
                feed,
                spindle,
            } => {
                assert_eq!(state, MachineState::Idle);
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
                assert_eq!(z, 30.0);
                assert_eq!(feed, 1000.0);
                assert_eq!(spindle, 12000.0);
            }
            _ => panic!("Expected Status response"),
        }

        // Check that internal state was updated
        assert_eq!(comm.machine_state, MachineState::Idle);
        assert_eq!(comm.current_position, (10.0, 20.0, 30.0));
    }

    #[test]
    fn test_parse_smoothieware_response_feedback() {
        let mut comm = SmoothiewareCommunication::new();
        let response = comm.parse_smoothieware_response("[MSG: Test message]");
        assert!(matches!(response, GrblResponse::Feedback(_)));
        if let GrblResponse::Feedback(msg) = response {
            assert_eq!(msg, " Test message");
        }
    }

    #[test]
    fn test_parse_smoothieware_response_version() {
        let mut comm = SmoothiewareCommunication::new();
        let response = comm.parse_smoothieware_response("Smoothieware version 1.2.3");
        assert!(matches!(response, GrblResponse::Version(_)));
        assert_eq!(comm.grbl_version, "Smoothieware version 1.2.3");
    }

    #[test]
    fn test_parse_smoothieware_response_other() {
        let mut comm = SmoothiewareCommunication::new();
        let response = comm.parse_smoothieware_response("Some other response");
        assert!(matches!(response, GrblResponse::Other(_)));
    }

    #[test]
    fn test_connection_state_management() {
        let mut comm = SmoothiewareCommunication::new();

        // Initial state
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);

        // Test disconnect when already disconnected
        comm.disconnect_from_device();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert_eq!(comm.machine_state, MachineState::Unknown);
        assert_eq!(comm.current_position, (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_jog_command_formatting() {
        let mut comm = SmoothiewareCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test X axis jog (will fail due to no serial port)
        comm.jog_axis('X', 10.0);
        assert_eq!(comm.status_message, "Jog error: Not connected");

        // Test Y axis jog with negative (will fail due to no serial port)
        comm.jog_axis('Y', -5.5);
        assert_eq!(comm.status_message, "Jog error: Not connected");

        // Test Z axis jog (will fail due to no serial port)
        comm.jog_axis('Z', 2.25);
        assert_eq!(comm.status_message, "Jog error: Not connected");
    }

    #[test]
    fn test_home_command() {
        let mut comm = SmoothiewareCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        comm.home_all_axes();
        assert_eq!(comm.status_message, "Home error: Not connected");
    }

    #[test]
    fn test_override_commands() {
        let mut comm = SmoothiewareCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test spindle override (will fail due to no serial port)
        comm.send_spindle_override(75.0);
        assert_eq!(comm.status_message, "Spindle override error: Not connected");

        // Test feed override (will fail due to no serial port)
        comm.send_feed_override(120.0);
        assert_eq!(comm.status_message, "Feed override error: Not connected");
    }

    #[test]
    fn test_disconnected_operations() {
        let mut comm = SmoothiewareCommunication::new();
        comm.connection_state = ConnectionState::Disconnected;

        // Test jog when disconnected
        comm.jog_axis('X', 10.0);
        assert_eq!(comm.status_message, "Not connected");

        // Test home when disconnected
        comm.home_all_axes();
        assert_eq!(comm.status_message, "Not connected");

        // Test overrides when disconnected
        comm.send_spindle_override(50.0);
        assert_eq!(comm.status_message, "Not connected");
    }

    #[test]
    fn test_gcode_line_sending() {
        let mut comm = SmoothiewareCommunication::new();

        // Test disconnected state
        comm.connection_state = ConnectionState::Disconnected;
        let result = comm.send_gcode_line("G1 X10");
        assert!(result.is_err());

        // Test connected state (would need mock serial port for full test)
        comm.connection_state = ConnectionState::Connected;
        // Note: This would fail without a real serial port, but tests the logic
        let result = comm.send_gcode_line("G1 X10 Y20 F100");
        assert!(result.is_err()); // Expected to fail without serial port
    }

    #[test]
    fn test_error_recovery_connection_error() {
        let mut comm = SmoothiewareCommunication::new();
        comm.recovery_config.auto_recovery_enabled = true;
        comm.recovery_config.max_reconnect_attempts = 3;

        let action = comm.attempt_recovery("connection timeout").unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::Reconnect);
        assert_eq!(comm.recovery_state.reconnect_attempts, 1);
        assert_eq!(comm.connection_state, ConnectionState::Recovering);
    }

    #[test]
    fn test_error_recovery_command_error() {
        let mut comm = SmoothiewareCommunication::new();
        comm.recovery_config.auto_recovery_enabled = true;
        comm.recovery_config.max_command_retries = 2;

        let action = comm.attempt_recovery("command syntax error").unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::RetryCommand);
        assert_eq!(comm.recovery_state.command_retry_count, 1);
    }

    #[test]
    fn test_error_recovery_critical_error() {
        let mut comm = SmoothiewareCommunication::new();
        comm.recovery_config.auto_recovery_enabled = true;
        comm.recovery_config.reset_on_critical_error = true;

        let action = comm.attempt_recovery("alarm triggered").unwrap();
        assert_eq!(
            action,
            crate::communication::RecoveryAction::ResetController
        );
    }

    #[test]
    fn test_error_recovery_disabled() {
        let mut comm = SmoothiewareCommunication::new();
        comm.recovery_config.auto_recovery_enabled = false;

        let result = comm.attempt_recovery("some error");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Auto recovery disabled");
    }

    #[test]
    fn test_error_recovery_max_attempts() {
        let mut comm = SmoothiewareCommunication::new();
        comm.recovery_config.auto_recovery_enabled = true;
        comm.recovery_config.max_reconnect_attempts = 1;
        comm.recovery_state.reconnect_attempts = 1;

        let action = comm.attempt_recovery("connection failed").unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::AbortJob);
    }

    #[test]
    fn test_reset_recovery_state() {
        let mut comm = SmoothiewareCommunication::new();
        comm.recovery_state.reconnect_attempts = 2;
        comm.recovery_state.command_retry_count = 1;
        comm.recovery_state.last_error = Some("test error".to_string());
        comm.recovery_state.recovery_actions_taken =
            vec![crate::communication::RecoveryAction::Reconnect];

        comm.reset_recovery_state();

        assert_eq!(comm.recovery_state.reconnect_attempts, 0);
        assert_eq!(comm.recovery_state.command_retry_count, 0);
        assert!(comm.recovery_state.last_error.is_none());
        assert!(comm.recovery_state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_is_recovering() {
        let mut comm = SmoothiewareCommunication::new();

        // Initially not recovering
        assert!(!comm.is_recovering());

        // Set recovering state
        comm.connection_state = ConnectionState::Recovering;
        assert!(comm.is_recovering());

        // Reset and test reconnect attempts
        comm.connection_state = ConnectionState::Disconnected;
        comm.recovery_state.reconnect_attempts = 1;
        assert!(comm.is_recovering());
    }

    #[test]
    fn test_parse_smoothieware_response_edge_cases() {
        let mut comm = SmoothiewareCommunication::new();

        // Test empty response
        let response = comm.parse_smoothieware_response("");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test whitespace only
        let response = comm.parse_smoothieware_response("   ");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test malformed status response
        let response = comm.parse_smoothieware_response("<>");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test status response with missing fields
        let response = comm.parse_smoothieware_response("<Idle>");
        assert!(matches!(response, GrblResponse::Status { .. }));

        // Test error response without colon
        let response = comm.parse_smoothieware_response("error");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test alarm response (assuming Smoothieware uses similar format)
        let response = comm.parse_smoothieware_response("ALARM:1");
        assert!(matches!(response, GrblResponse::Other(_))); // Smoothieware may not use this format
    }

    #[test]
    fn test_parse_smoothieware_status_complex() {
        let mut comm = SmoothiewareCommunication::new();

        // Test complex status response with all fields
        let response = comm.parse_smoothieware_response(
            "<Run|MPos:10.500,20.750,5.250,1.000,2.000|WPos:0.000,0.000,0.000|FS:1500,200|Ln:123>",
        );

        match response {
            GrblResponse::Status {
                state,
                x,
                y,
                z,
                feed,
                spindle,
            } => {
                assert_eq!(state, MachineState::Run);
                assert_eq!(x, 10.5);
                assert_eq!(y, 20.75);
                assert_eq!(z, 5.25);
                assert_eq!(feed, 1500.0);
                assert_eq!(spindle, 200.0);
            }
            _ => panic!("Expected Status response"),
        }

        // Check that internal state was updated
        assert_eq!(comm.machine_state, MachineState::Run);
        assert_eq!(comm.current_position, (10.5, 20.75, 5.25));
    }

    #[test]
    fn test_parse_smoothieware_status_minimal() {
        let mut comm = SmoothiewareCommunication::new();

        // Test minimal status response
        let response = comm.parse_smoothieware_response("<Idle|MPos:1.000,2.000,3.000>");
        assert!(matches!(response, GrblResponse::Status { .. }));

        // Check that internal state was updated
        assert_eq!(comm.machine_state, MachineState::Idle);
        assert_eq!(comm.current_position, (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_parse_smoothieware_status_invalid() {
        let mut comm = SmoothiewareCommunication::new();

        // Test invalid status responses
        assert!(matches!(
            comm.parse_smoothieware_response(""),
            GrblResponse::Other(_)
        ));
        assert!(matches!(
            comm.parse_smoothieware_response("not a status"),
            GrblResponse::Other(_)
        ));
        assert!(matches!(
            comm.parse_smoothieware_response("<>"),
            GrblResponse::Other(_)
        ));
        assert!(matches!(
            comm.parse_smoothieware_response("<InvalidState>"),
            GrblResponse::Other(_)
        ));
    }

    #[test]
    fn test_jog_command_edge_cases() {
        let mut comm = SmoothiewareCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test very small jog distance
        comm.jog_axis('X', 0.001);
        assert_eq!(comm.status_message, "Jog error: Not connected");

        // Test negative jog distance
        comm.jog_axis('Y', -5.0);
        assert_eq!(comm.status_message, "Jog error: Not connected");

        // Test large jog distance
        comm.jog_axis('Z', 1000.0);
        assert_eq!(comm.status_message, "Jog error: Not connected");

        // Test different axes
        comm.jog_axis('A', 45.0);
        assert_eq!(comm.status_message, "Jog error: Not connected");

        comm.jog_axis('B', -30.0);
        assert_eq!(comm.status_message, "Jog error: Not connected");
    }

    #[test]
    fn test_override_commands_edge_cases() {
        let mut comm = SmoothiewareCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test zero override
        comm.send_spindle_override(0.0);
        assert_eq!(comm.status_message, "Spindle override error: Not connected");

        // Test maximum override
        comm.send_feed_override(200.0);
        assert_eq!(comm.status_message, "Feed override error: Not connected");

        // Test negative override
        comm.send_spindle_override(-10.0);
        assert_eq!(comm.status_message, "Spindle override error: Not connected");

        // Test very high override
        comm.send_feed_override(500.0);
        assert_eq!(comm.status_message, "Feed override error: Not connected");
    }
}
