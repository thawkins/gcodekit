use crate::MachinePosition;
use crate::communication::grbl::{GrblResponse, GrblStatus, MachineState, Position, WcsCoordinate};
use crate::communication::*;
use serialport::{self, SerialPort};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::time::Duration;

// FluidNC is GRBL-compatible, so this is similar to GrblCommunication
#[derive(Debug)]
pub struct FluidNCCommunication {
    pub connection_state: ConnectionState,
    pub selected_port: String,
    pub available_ports: Vec<String>,
    pub status_message: String,
    pub grbl_version: String,
    pub current_status: GrblStatus,
    pub gcode_queue: VecDeque<String>,
    pub last_response: Option<GrblResponse>,
    pub current_wcs: WcsCoordinate,
    pub recovery_config: ErrorRecoveryConfig,
    pub recovery_state: RecoveryState,
    pub health_metrics: crate::communication::HealthMetrics,
    feed_hold_sent: bool,
    serial_port: Option<Box<dyn SerialPort>>,
}

impl Default for FluidNCCommunication {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            selected_port: String::new(),
            available_ports: Vec::new(),
            status_message: String::new(),
            grbl_version: String::new(),
            current_status: GrblStatus::default(),
            gcode_queue: VecDeque::new(),
            last_response: None,
            current_wcs: WcsCoordinate::G54,
            recovery_config: ErrorRecoveryConfig::default(),
            recovery_state: RecoveryState {
                reconnect_attempts: 0,
                last_reconnect_attempt: None,
                command_retry_count: 0,
                last_error: None,
                recovery_actions_taken: Vec::new(),
            },
            health_metrics: crate::communication::HealthMetrics::default(),
            feed_hold_sent: false,
            serial_port: None,
        }
    }
}

impl FluidNCCommunication {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            selected_port: String::new(),
            available_ports: Vec::new(),
            status_message: String::new(),
            grbl_version: String::new(),
            current_status: GrblStatus::default(),
            gcode_queue: VecDeque::new(),
            last_response: None,
            current_wcs: WcsCoordinate::G54,
            recovery_config: ErrorRecoveryConfig::default(),
            recovery_state: RecoveryState {
                reconnect_attempts: 0,
                last_reconnect_attempt: None,
                command_retry_count: 0,
                last_error: None,
                recovery_actions_taken: Vec::new(),
            },
            health_metrics: crate::communication::HealthMetrics::default(),
            feed_hold_sent: false,
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
            port.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
            port.write_all(b"\n").map_err(|e| e.to_string())?;
            port.flush().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Not connected".to_string())
        }
    }

    pub fn read_grbl_responses(&mut self) -> Vec<String> {
        let mut responses = Vec::new();
        if let Some(ref mut port) = self.serial_port {
            let mut buffer = [0; 1024];
            if let Ok(bytes_read) = port.read(&mut buffer) {
                if bytes_read > 0 {
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                    for line in response.lines() {
                        if !line.trim().is_empty() {
                            responses.push(line.to_string());
                            self.last_response = Some(GrblResponse::Other(line.to_string()));
                        }
                    }
                }
            }
        }
        responses
    }

    pub fn parse_grbl_status(&mut self, response: &str) -> Option<GrblStatus> {
        // GRBL status response format: <State|MPos:X,Y,Z|WPos:X,Y,Z|F:feed|S:speed>
        if !response.starts_with('<') || !response.ends_with('>') {
            return None;
        }

        let content = &response[1..response.len() - 1];
        let parts: Vec<&str> = content.split('|').collect();

        if parts.is_empty() || parts[0].is_empty() {
            return None;
        }

        let machine_state = MachineState::from(parts[0]);
        // If the state is Unknown, it means the input was invalid
        if machine_state == MachineState::Unknown {
            return None;
        }

        let mut status = GrblStatus::default();
        status.machine_state = machine_state;

        for part in &parts[1..] {
            if let Some(colon_pos) = part.find(':') {
                let key = &part[..colon_pos];
                let value = &part[colon_pos + 1..];

                match key {
                    "MPos" => {
                        if let Some(pos) = self.parse_position(value) {
                            status.machine_position = pos;
                        }
                    }
                    "WPos" => {
                        if let Some(pos) = self.parse_position(value) {
                            status.work_position = pos;
                        }
                    }
                    "F" => {
                        if let Ok(feed) = value.parse::<f32>() {
                            status.feed_rate = Some(feed);
                        }
                    }
                    "S" => {
                        if let Ok(speed) = value.parse::<f32>() {
                            status.spindle_speed = Some(speed);
                        }
                    }
                    "Ln" => {
                        if let Ok(line) = value.parse::<u32>() {
                            status.line_number = Some(line);
                        }
                    }
                    "Pn" => {
                        status.input_pin_state = Some(value.to_string());
                    }
                    "Ov" => {
                        status.override_values = Some(value.to_string());
                    }
                    _ => {} // Unknown field, ignore
                }
            }
        }

        // Handle safety door
        if status.machine_state == MachineState::Door && !self.feed_hold_sent {
            let _ = self.send_gcode_line("!");
            self.feed_hold_sent = true;
        } else if status.machine_state != MachineState::Door && self.feed_hold_sent {
            let _ = self.send_gcode_line("~");
            self.feed_hold_sent = false;
        }

        Some(status)
    }

    fn parse_position(&self, pos_str: &str) -> Option<Position> {
        let coords: Vec<&str> = pos_str.split(',').collect();
        if coords.len() < 3 || coords.len() > 6 {
            return None;
        }

        // All coordinates must be valid floats
        let parsed_coords: Vec<f32> = coords
            .iter()
            .map(|s| s.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()
            .ok()?;

        if parsed_coords.len() < 3 {
            return None;
        }

        let mut pos = Position {
            x: parsed_coords[0],
            y: parsed_coords[1],
            z: parsed_coords[2],
            a: None,
            b: None,
            c: None,
            d: None,
        };

        if parsed_coords.len() >= 4 {
            pos.a = Some(parsed_coords[3]);
        }
        if parsed_coords.len() >= 5 {
            pos.b = Some(parsed_coords[4]);
        }
        if parsed_coords.len() >= 6 {
            pos.c = Some(parsed_coords[5]);
        }

        Some(pos)
    }
}

impl CncController for FluidNCCommunication {
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
        self.read_grbl_responses().first().cloned()
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
        let cmd = format!("$J=G21G91{}{:.3}F1000", axis, distance);
        let _ = self.send_gcode_line(&cmd);
    }

    fn home_all_axes(&mut self) {
        let _ = self.send_gcode_line("$H");
    }

    fn send_spindle_override(&mut self, percentage: f32) {
        let cmd = format!("M51 S{:.0}", percentage);
        let _ = self.send_gcode_line(&cmd);
    }

    fn send_feed_override(&mut self, percentage: f32) {
        let cmd = format!("M50 S{:.0}", percentage);
        let _ = self.send_gcode_line(&cmd);
    }

    fn get_version(&self) -> &str {
        &self.grbl_version
    }

    fn handle_response(&mut self, response: &str) -> Option<MachinePosition> {
        if let Some(status) = self.parse_grbl_status(response) {
            self.current_status = status;
            Some(MachinePosition {
                x: self.current_status.work_position.x,
                y: self.current_status.work_position.y,
                z: self.current_status.work_position.z,
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
        // Send feed hold command to GRBL
        let _ = self.send_gcode_line("!");
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
                self.recovery_state.last_reconnect_attempt = Some(Instant::now());
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
        // Reset connection state if it was in recovering mode
        if self.connection_state == ConnectionState::Recovering {
            self.connection_state = ConnectionState::Disconnected;
        }
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
    fn test_fluidnc_communication_new() {
        let comm = FluidNCCommunication::new();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert!(comm.selected_port.is_empty());
        assert!(comm.available_ports.is_empty());
        assert!(comm.status_message.is_empty());
        assert!(comm.grbl_version.is_empty());
        assert!(comm.serial_port.is_none());
    }

    #[test]
    fn test_parse_grbl_status() {
        let mut comm = FluidNCCommunication::new();

        // Test complete status response
        let status_str =
            "<Idle|MPos:10.000,20.000,30.000|WPos:5.000,15.000,25.000|F:1000.0|S:12000.0>";
        let status = comm.parse_grbl_status(status_str).unwrap();

        assert_eq!(status.machine_state, MachineState::Idle);
        assert_eq!(status.machine_position.x, 10.0);
        assert_eq!(status.machine_position.y, 20.0);
        assert_eq!(status.machine_position.z, 30.0);
        assert_eq!(status.work_position.x, 5.0);
        assert_eq!(status.work_position.y, 15.0);
        assert_eq!(status.work_position.z, 25.0);
        assert_eq!(status.feed_rate, Some(1000.0));
        assert_eq!(status.spindle_speed, Some(12000.0));
    }

    #[test]
    fn test_parse_grbl_status_minimal() {
        let mut comm = FluidNCCommunication::new();

        // Test minimal status response
        let status_str = "<Run|MPos:1.000,2.000,3.000>";
        let status = comm.parse_grbl_status(status_str).unwrap();

        assert_eq!(status.machine_state, MachineState::Run);
        assert_eq!(status.machine_position.x, 1.0);
        assert_eq!(status.machine_position.y, 2.0);
        assert_eq!(status.machine_position.z, 3.0);
        assert_eq!(status.work_position.x, 0.0); // Default
        assert_eq!(status.feed_rate, None);
    }

    #[test]
    fn test_parse_grbl_status_complex() {
        let mut comm = FluidNCCommunication::new();

        // Test complex status response with all fields
        let status = comm
            .parse_grbl_status(
                "<Run|MPos:10.500,20.750,5.250|WPos:0.000,0.000,0.000|F:1500|S:200|Ln:5|Pn:XYZ>",
            )
            .unwrap();

        assert_eq!(status.machine_state, MachineState::Run);
        assert_eq!(status.machine_position.x, 10.5);
        assert_eq!(status.machine_position.y, 20.75);
        assert_eq!(status.machine_position.z, 5.25);
        assert_eq!(status.work_position.x, 0.0);
        assert_eq!(status.work_position.y, 0.0);
        assert_eq!(status.work_position.z, 0.0);
        assert_eq!(status.feed_rate, Some(1500.0));
        assert_eq!(status.spindle_speed, Some(200.0));
        assert_eq!(status.line_number, Some(5));
        assert_eq!(status.input_pin_state, Some("XYZ".to_string()));
    }

    #[test]
    fn test_parse_grbl_status_invalid() {
        let mut comm = FluidNCCommunication::new();

        // Test invalid status responses
        assert!(comm.parse_grbl_status("").is_none());
        assert!(comm.parse_grbl_status("not a status").is_none());
        assert!(comm.parse_grbl_status("<>").is_none());
        assert!(comm.parse_grbl_status("<InvalidState>").is_none());
    }

    #[test]
    fn test_parse_position() {
        let comm = FluidNCCommunication::new();

        // Test 3-axis position
        let pos = comm.parse_position("10.5,20.75,5.25").unwrap();
        assert_eq!(pos.x, 10.5);
        assert_eq!(pos.y, 20.75);
        assert_eq!(pos.z, 5.25);
        assert!(pos.a.is_none());

        // Test 4-axis position
        let pos = comm.parse_position("1.0,2.0,3.0,45.0").unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
        assert_eq!(pos.a, Some(45.0));

        // Test 6-axis position
        let pos = comm.parse_position("0.0,0.0,0.0,90.0,180.0,270.0").unwrap();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
        assert_eq!(pos.a, Some(90.0));
        assert_eq!(pos.b, Some(180.0));
        assert_eq!(pos.c, Some(270.0));

        // Test invalid positions
        assert!(comm.parse_position("").is_none());
        assert!(comm.parse_position("1.0,2.0").is_none());
        assert!(comm.parse_position("1.0,2.0,3.0,invalid").is_none());
    }

    #[test]
    fn test_connection_state_management() {
        let mut comm = FluidNCCommunication::new();

        // Initial state
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);

        // Test disconnect when already disconnected
        comm.disconnect_from_device();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert_eq!(comm.status_message, "Disconnected".to_string());
    }

    #[test]
    fn test_gcode_line_sending() {
        let mut comm = FluidNCCommunication::new();

        // Test disconnected state
        comm.connection_state = ConnectionState::Disconnected;
        let result = comm.send_gcode_line("G1 X10");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected".to_string());
    }

    #[test]
    fn test_jog_command_formatting() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test X axis jog
        comm.jog_axis('X', 10.0);
        // Since we can't mock the serial port, we just ensure no panic occurs
        // The actual command format is checked in integration tests

        // Test Y axis jog with decimal
        comm.jog_axis('Y', -5.5);

        // Test Z axis jog
        comm.jog_axis('Z', 2.25);
    }

    #[test]
    fn test_override_commands() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test spindle override
        comm.send_spindle_override(75.0);

        // Test feed override
        comm.send_feed_override(120.0);

        // Test decimal values
        comm.send_spindle_override(33.7);
    }

    #[test]
    fn test_override_commands_edge_cases() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test zero override
        comm.send_spindle_override(0.0);

        // Test maximum override
        comm.send_feed_override(200.0);

        // Test negative override
        comm.send_spindle_override(-10.0);

        // Test very high override
        comm.send_feed_override(500.0);
    }

    #[test]
    fn test_home_command() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        comm.home_all_axes();
        // Command sent, no panic
    }

    #[test]
    fn test_disconnected_operations() {
        let mut comm = FluidNCCommunication::new();
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
    fn test_error_recovery_config() {
        let mut comm = FluidNCCommunication::new();
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
        let comm = FluidNCCommunication::new();
        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 0);
        assert!(state.last_reconnect_attempt.is_none());
        assert_eq!(state.command_retry_count, 0);
        assert!(state.last_error.is_none());
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_recovery_connection_error() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Error;

        // Test recovery for connection error
        let result = comm.attempt_recovery("connection lost");
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::Reconnect);

        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 1);
        assert!(state.last_reconnect_attempt.is_some());
        assert_eq!(state.last_error.as_ref().unwrap(), "connection lost");
        assert_eq!(state.recovery_actions_taken.len(), 1);
        assert_eq!(
            state.recovery_actions_taken[0],
            crate::communication::RecoveryAction::Reconnect
        );
    }

    #[test]
    fn test_recovery_command_error() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test recovery for command error
        let result = comm.attempt_recovery("Command syntax error");
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::RetryCommand);

        let state = comm.get_recovery_state();
        assert_eq!(state.command_retry_count, 1);
        assert_eq!(state.last_error.as_ref().unwrap(), "Command syntax error");
        assert_eq!(state.recovery_actions_taken.len(), 1);
        assert_eq!(
            state.recovery_actions_taken[0],
            crate::communication::RecoveryAction::RetryCommand
        );
    }

    #[test]
    fn test_recovery_max_attempts_exceeded() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Error;

        // Exhaust reconnect attempts
        for _ in 0..3 {
            let _ = comm.attempt_recovery("connection lost");
        }

        // Next attempt should abort job
        let result = comm.attempt_recovery("connection lost");
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::AbortJob);

        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 3);
    }

    #[test]
    fn test_recovery_command_max_retries_exceeded() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Exhaust command retries
        for _ in 0..3 {
            let _ = comm.attempt_recovery("Command syntax error");
        }

        // Next attempt should skip command
        let result = comm.attempt_recovery("Command syntax error");
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(action, crate::communication::RecoveryAction::SkipCommand);

        let state = comm.get_recovery_state();
        assert_eq!(state.command_retry_count, 3);
    }

    #[test]
    fn test_recovery_critical_error() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test recovery for critical error
        let result = comm.attempt_recovery("alarm: hard limit triggered");
        assert!(result.is_ok());
        let action = result.unwrap();
        assert_eq!(
            action,
            crate::communication::RecoveryAction::ResetController
        );

        let state = comm.get_recovery_state();
        assert_eq!(
            state.last_error.as_ref().unwrap(),
            "alarm: hard limit triggered"
        );
        assert_eq!(state.recovery_actions_taken.len(), 1);
        assert_eq!(
            state.recovery_actions_taken[0],
            crate::communication::RecoveryAction::ResetController
        );
    }

    #[test]
    fn test_recovery_disabled() {
        let mut comm = FluidNCCommunication::new();
        let mut config = comm.get_recovery_config().clone();
        config.auto_recovery_enabled = false;
        comm.set_recovery_config(config);

        // Test that recovery is disabled
        let result = comm.attempt_recovery("connection lost");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Auto recovery disabled");
    }

    #[test]
    fn test_recovery_state_reset() {
        let mut comm = FluidNCCommunication::new();
        comm.connection_state = ConnectionState::Error;

        // Perform some recovery actions
        let _ = comm.attempt_recovery("connection lost");
        let _ = comm.attempt_recovery("command timeout");

        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 2);
        assert_eq!(state.command_retry_count, 0);
        assert_eq!(state.recovery_actions_taken.len(), 2);

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
    fn test_recovery_is_recovering() {
        let mut comm = FluidNCCommunication::new();

        // Initially not recovering
        assert!(!comm.is_recovering());

        // After recovery attempt, should be recovering
        comm.connection_state = ConnectionState::Error;
        let _ = comm.attempt_recovery("connection lost");
        assert!(comm.is_recovering());

        // After reset, should not be recovering
        comm.reset_recovery_state();
        assert!(!comm.is_recovering());
    }
}
