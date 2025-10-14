use crate::MachinePosition;
use crate::communication::grbl::Position;
use crate::communication::*;
use serde::Deserialize;
use serde_json;
use serialport::{self, SerialPort};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::time::Duration;

#[derive(Debug)]
pub struct G2coreCommunication {
    pub connection_state: ConnectionState,
    pub selected_port: String,
    pub available_ports: Vec<String>,
    pub status_message: String,
    pub version: String,
    pub current_status: G2coreStatus,
    pub gcode_queue: VecDeque<String>,
    pub last_response: Option<String>,
    pub recovery_config: ErrorRecoveryConfig,
    pub recovery_state: RecoveryState,
    pub health_metrics: crate::communication::HealthMetrics,
    serial_port: Option<Box<dyn SerialPort>>,
}

#[derive(Clone, Debug, Default)]
pub struct G2coreStatus {
    pub machine_state: String,
    pub position: Position,
    pub feed_rate: Option<f32>,
    pub spindle_speed: Option<f32>,
}

/// G2core JSON status report structure
#[derive(Deserialize, Debug)]
struct G2coreStatusReport {
    sr: G2coreStatusReportData,
}

#[derive(Deserialize, Debug)]
struct G2coreStatusReportData {
    posx: Option<f32>,
    posy: Option<f32>,
    posz: Option<f32>,
    posa: Option<f32>,
    posb: Option<f32>,
    posc: Option<f32>,
    feed: Option<f32>,
    vel: Option<f32>,
    stat: Option<u32>, // Machine state
}

impl Default for G2coreCommunication {
    fn default() -> Self {
        Self::new()
    }
}

impl G2coreCommunication {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            selected_port: String::new(),
            available_ports: Vec::new(),
            status_message: String::new(),
            version: String::new(),
            current_status: G2coreStatus::default(),
            gcode_queue: VecDeque::new(),
            last_response: None,
            recovery_config: ErrorRecoveryConfig::default(),
            recovery_state: RecoveryState {
                reconnect_attempts: 0,
                last_reconnect_attempt: None,
                command_retry_count: 0,
                last_error: None,
                recovery_actions_taken: Vec::new(),
            },
            health_metrics: crate::communication::HealthMetrics::default(),
            serial_port: None,
        }
    }

    pub fn refresh_ports(&mut self) {
        self.available_ports = serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect();
    }

    pub fn connect_to_device(&mut self) {
        if self.selected_port.is_empty() {
            self.status_message = "No port selected".to_string();
            return;
        }

        match serialport::new(&self.selected_port, 115200)
            .timeout(Duration::from_millis(100))
            .open()
        {
            Ok(port) => {
                self.serial_port = Some(port);
                self.connection_state = ConnectionState::Connected;
                self.status_message = format!("Connected to {}", self.selected_port);
            }
            Err(e) => {
                self.status_message = format!("Connection failed: {}", e);
            }
        }
    }

    pub fn disconnect_from_device(&mut self) {
        self.serial_port = None;
        self.connection_state = ConnectionState::Disconnected;
        self.status_message = "Disconnected".to_string();
    }

    pub fn send_gcode_line(&mut self, line: &str) -> Result<(), String> {
        if let Some(ref mut port) = self.serial_port {
            let json_cmd = format!("{{\"gc\":\"{}\"}}\n", line);
            port.write_all(json_cmd.as_bytes())
                .map_err(|e| e.to_string())?;
            port.flush().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn send_json_command(&mut self, json_cmd: &str) -> Result<(), String> {
        if let Some(ref mut port) = self.serial_port {
            let cmd = format!("{}\n", json_cmd);
            port.write_all(cmd.as_bytes()).map_err(|e| e.to_string())?;
            port.flush().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn read_responses(&mut self) -> Vec<String> {
        let mut responses = Vec::new();
        if let Some(ref mut port) = self.serial_port {
            let mut buffer = [0; 1024];
            if let Ok(bytes_read) = port.read(&mut buffer) {
                if bytes_read > 0 {
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                    for line in response.lines() {
                        if !line.trim().is_empty() {
                            responses.push(line.to_string());
                            self.last_response = Some(line.to_string());
                        }
                    }
                }
            }
        }
        responses
    }

    pub fn parse_g2core_status(&mut self, response: &str) -> Option<G2coreStatus> {
        // Parse G2core JSON status report
        if let Ok(status_report) = serde_json::from_str::<G2coreStatusReport>(response) {
            let data = status_report.sr;
            let mut status = G2coreStatus::default();

            // Parse position
            status.position.x = data.posx.unwrap_or(0.0);
            status.position.y = data.posy.unwrap_or(0.0);
            status.position.z = data.posz.unwrap_or(0.0);
            status.position.a = data.posa;
            status.position.b = data.posb;
            status.position.c = data.posc;

            // Parse feed rate and velocity
            status.feed_rate = data.feed;
            status.spindle_speed = data.vel;

            // Parse machine state (simplified mapping)
            if let Some(stat) = data.stat {
                status.machine_state = match stat {
                    0 => "Initializing".to_string(),
                    1 => "Ready".to_string(),
                    2 => "Alarm".to_string(),
                    3 => "Stop".to_string(),
                    4 => "End".to_string(),
                    5 => "Run".to_string(),
                    6 => "Hold".to_string(),
                    7 => "Probe".to_string(),
                    8 => "Cycle".to_string(),
                    9 => "Homing".to_string(),
                    10 => "Jog".to_string(),
                    11 => "Interlock".to_string(),
                    _ => format!("Unknown({})", stat),
                };
            }

            Some(status)
        } else {
            None
        }
    }
}

impl CncController for G2coreCommunication {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn set_port(&mut self, port: String) {
        self.selected_port = port;
    }

    fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.connect_to_device();
        Ok(())
    }

    fn disconnect(&mut self) {
        self.disconnect_from_device();
    }

    fn send_gcode_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.send_gcode_line(line).map_err(|e| e.into())
    }

    fn read_response(&mut self) -> Option<String> {
        self.read_responses().first().cloned()
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
        let cmd = format!("G0 {}{:.3}", axis, distance);
        let _ = self.send_gcode_line(&cmd);
    }

    fn home_all_axes(&mut self) {
        let _ = self.send_gcode_line("G28");
    }

    fn send_spindle_override(&mut self, percentage: f32) {
        // G2core spindle override command
        let cmd = format!("{{\"mto\":{:.0}}}", percentage);
        let _ = self.send_json_command(&cmd);
    }

    fn send_feed_override(&mut self, percentage: f32) {
        // G2core feed override command
        let cmd = format!("{{\"mfo\":{:.0}}}", percentage);
        let _ = self.send_json_command(&cmd);
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn handle_response(&mut self, response: &str) -> Option<MachinePosition> {
        if let Some(status) = self.parse_g2core_status(response) {
            self.current_status = status;
            Some(MachinePosition {
                x: self.current_status.position.x,
                y: self.current_status.position.y,
                z: self.current_status.position.z,
                a: None,
                b: None,
                c: None,
                d: None,
            })
        } else {
            None
        }
    }

    fn emergency_stop(&mut self) {
        let _ = self.send_gcode_line("!"); // Feed hold
    }

    fn get_recovery_config(&self) -> &ErrorRecoveryConfig {
        &self.recovery_config
    }

    fn get_recovery_state(&self) -> &RecoveryState {
        &self.recovery_state
    }

    fn set_recovery_config(&mut self, config: ErrorRecoveryConfig) {
        self.recovery_config = config;
    }

    fn attempt_recovery(&mut self, error: &str) -> Result<RecoveryAction, String> {
        if !self.recovery_config.auto_recovery_enabled {
            println!("[RECOVERY] Auto recovery disabled for error: {}", error);
            return Err("Auto recovery disabled".to_string());
        }

        self.recovery_state.last_error = Some(error.to_string());
        self.health_metrics.update_error_pattern(error);
        println!("[RECOVERY] Attempting recovery for error: {}", error);

        // Classify error and determine recovery action
        let action = if error.contains("connection") || error.contains("timeout") {
            // Connection-related errors
            println!(
                "[RECOVERY] Classified as connection error (attempts: {}/{})",
                self.recovery_state.reconnect_attempts, self.recovery_config.max_reconnect_attempts
            );
            if self.recovery_state.reconnect_attempts < self.recovery_config.max_reconnect_attempts
            {
                self.recovery_state.reconnect_attempts += 1;
                self.recovery_state.last_reconnect_attempt = Some(std::time::Instant::now());
                self.connection_state = ConnectionState::Recovering;
                println!(
                    "[RECOVERY] Initiating reconnection attempt {}",
                    self.recovery_state.reconnect_attempts
                );
                RecoveryAction::Reconnect
            } else {
                println!("[RECOVERY] Max reconnection attempts reached, aborting job");
                RecoveryAction::AbortJob
            }
        } else if error.contains("command") || error.contains("syntax") {
            // Command-related errors
            println!(
                "[RECOVERY] Classified as command error (retries: {}/{})",
                self.recovery_state.command_retry_count, self.recovery_config.max_command_retries
            );
            if self.recovery_state.command_retry_count < self.recovery_config.max_command_retries {
                self.recovery_state.command_retry_count += 1;
                println!(
                    "[RECOVERY] Retrying command (attempt {})",
                    self.recovery_state.command_retry_count
                );
                RecoveryAction::RetryCommand
            } else {
                println!("[RECOVERY] Max command retries reached, skipping command");
                RecoveryAction::SkipCommand
            }
        } else if error.contains("alarm") || error.contains("emergency") {
            // Critical errors
            println!(
                "[RECOVERY] Classified as critical error (reset_on_critical: {})",
                self.recovery_config.reset_on_critical_error
            );
            if self.recovery_config.reset_on_critical_error {
                println!("[RECOVERY] Resetting controller due to critical error");
                RecoveryAction::ResetController
            } else {
                println!("[RECOVERY] Aborting job due to critical error");
                RecoveryAction::AbortJob
            }
        } else {
            // Unknown errors - try reset
            println!("[RECOVERY] Classified as unknown error, attempting controller reset");
            RecoveryAction::ResetController
        };

        self.recovery_state
            .recovery_actions_taken
            .push(action.clone());
        println!("[RECOVERY] Recovery action taken: {:?}", action);
        Ok(action)
    }

    fn reset_recovery_state(&mut self) {
        self.recovery_state = RecoveryState {
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
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g2core_communication_new() {
        let comm = G2coreCommunication::new();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert!(comm.selected_port.is_empty());
        assert!(comm.available_ports.is_empty());
        assert!(comm.status_message.is_empty());
        assert!(comm.version.is_empty());
        assert!(comm.serial_port.is_none());
    }

    #[test]
    fn test_connection_state_management() {
        let mut comm = G2coreCommunication::new();

        // Initial state
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);

        // Test disconnect when already disconnected
        comm.disconnect_from_device();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert_eq!(comm.status_message, "Disconnected".to_string());
    }

    #[test]
    fn test_gcode_line_sending() {
        let mut comm = G2coreCommunication::new();

        // Test disconnected state
        comm.connection_state = ConnectionState::Disconnected;
        let result = comm.send_gcode_line("G1 X10");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected".to_string());
    }

    #[test]
    fn test_jog_command_formatting() {
        let mut comm = G2coreCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test X axis jog
        comm.jog_axis('X', 10.0);

        // Test Y axis jog with decimal
        comm.jog_axis('Y', -5.5);

        // Test Z axis jog
        comm.jog_axis('Z', 2.25);
    }

    #[test]
    fn test_home_command() {
        let mut comm = G2coreCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        comm.home_all_axes();
        // Command sent, no panic
    }

    #[test]
    fn test_override_commands() {
        let mut comm = G2coreCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test spindle override (not implemented)
        comm.send_spindle_override(75.0);

        // Test feed override (not implemented)
        comm.send_feed_override(120.0);
    }

    #[test]
    fn test_disconnected_operations() {
        let mut comm = G2coreCommunication::new();
        // Ensure disconnected
        comm.connection_state = ConnectionState::Disconnected;

        // Test jog when disconnected (should not panic)
        comm.jog_axis('X', 10.0);

        // Test home when disconnected
        comm.home_all_axes();

        // Test overrides when disconnected
        comm.send_spindle_override(50.0);
    }

    #[test]
    fn test_parse_g2core_status() {
        let mut comm = G2coreCommunication::new();

        // Test complete status report response
        let status_str = r#"{"sr":{"posx":10.5,"posy":20.75,"posz":5.25,"posa":45.0,"feed":1500.0,"vel":200.0,"stat":5}}"#;
        let status = comm.parse_g2core_status(status_str).unwrap();

        assert_eq!(status.position.x, 10.5);
        assert_eq!(status.position.y, 20.75);
        assert_eq!(status.position.z, 5.25);
        assert_eq!(status.position.a, Some(45.0));
        assert_eq!(status.feed_rate, Some(1500.0));
        assert_eq!(status.spindle_speed, Some(200.0));
        assert_eq!(status.machine_state, "Run");

        // Test minimal status response
        let status_str = r#"{"sr":{"posx":1.0,"posy":2.0,"posz":3.0}}"#;
        let status = comm.parse_g2core_status(status_str).unwrap();

        assert_eq!(status.position.x, 1.0);
        assert_eq!(status.position.y, 2.0);
        assert_eq!(status.position.z, 3.0);
        assert!(status.position.a.is_none());
        assert!(status.feed_rate.is_none());
        assert!(status.spindle_speed.is_none());
        assert_eq!(status.machine_state, "");

        // Test non-status response
        let status = comm.parse_g2core_status("some other response");
        assert!(status.is_none());

        // Test invalid JSON
        let status = comm.parse_g2core_status(r#"{"invalid": json}"#);
        assert!(status.is_none());
    }

    #[test]
    fn test_error_recovery_config() {
        let mut comm = G2coreCommunication::new();
        let config = comm.get_recovery_config();
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 2000);
        assert_eq!(config.max_command_retries, 3);
        assert_eq!(config.command_retry_delay_ms, 1000);
        assert!(config.reset_on_critical_error);
        assert!(config.auto_recovery_enabled);

        // Test setting custom config
        let custom_config = crate::communication::ErrorRecoveryConfig {
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 3000,
            max_command_retries: 2,
            command_retry_delay_ms: 1500,
            reset_on_critical_error: false,
            auto_recovery_enabled: false,
        };
        comm.set_recovery_config(custom_config.clone());
        let retrieved_config = comm.get_recovery_config();
        assert_eq!(retrieved_config.max_reconnect_attempts, 5);
        assert_eq!(retrieved_config.reconnect_delay_ms, 3000);
        assert_eq!(retrieved_config.max_command_retries, 2);
        assert_eq!(retrieved_config.command_retry_delay_ms, 1500);
        assert!(!retrieved_config.reset_on_critical_error);
        assert!(!retrieved_config.auto_recovery_enabled);
    }

    #[test]
    fn test_recovery_state_initialization() {
        let comm = G2coreCommunication::new();
        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 0);
        assert!(state.last_reconnect_attempt.is_none());
        assert_eq!(state.command_retry_count, 0);
        assert!(state.last_error.is_none());
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_attempt_recovery() {
        let mut comm = G2coreCommunication::new();

        // Test recovery (simplified implementation)
        let result = comm.attempt_recovery("some error");
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::RetryCommand);
    }

    #[test]
    fn test_recovery_state_reset() {
        let mut comm = G2coreCommunication::new();

        // Perform some recovery actions (simplified)
        let _ = comm.attempt_recovery("error");

        let state = comm.get_recovery_state();
        assert_eq!(state.recovery_actions_taken.len(), 0); // Simplified implementation doesn't track actions

        // Reset recovery state
        comm.reset_recovery_state();

        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 0);
        assert!(state.last_reconnect_attempt.is_none());
        assert_eq!(state.command_retry_count, 0);
        assert!(state.last_error.is_none());
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_is_recovering() {
        let comm = G2coreCommunication::new();

        // Currently always returns false
        assert!(!comm.is_recovering());
    }

    #[test]
    fn test_g2core_status_default() {
        let status = G2coreStatus::default();
        assert_eq!(status.machine_state, "");
        assert_eq!(status.position.x, 0.0);
        assert_eq!(status.position.y, 0.0);
        assert_eq!(status.position.z, 0.0);
        assert!(status.feed_rate.is_none());
        assert!(status.spindle_speed.is_none());
    }
}
