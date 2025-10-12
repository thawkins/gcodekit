use serialport::{SerialPort, available_ports};
use std::any::Any;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{Read, Write};

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
    serial_port: Option<Box<dyn SerialPort>>,
    response_queue: VecDeque<String>,
    last_response: Option<GrblResponse>,
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
            serial_port: None,
            response_queue: VecDeque::new(),
            last_response: None,
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

            if parts.len() >= 2 {
                let state = MachineState::from(parts[0]);

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
        self.send_gcode_line(line).map_err(|e| e)
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
            println!("[RECOVERY] Auto recovery disabled for error: {}", error);
            return Err("Auto recovery disabled".to_string());
        }

        self.recovery_state.last_error = Some(error.to_string());
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
                crate::communication::RecoveryAction::Reconnect
            } else {
                println!("[RECOVERY] Max reconnection attempts reached, aborting job");
                crate::communication::RecoveryAction::AbortJob
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
                crate::communication::RecoveryAction::RetryCommand
            } else {
                println!("[RECOVERY] Max command retries reached, skipping command");
                crate::communication::RecoveryAction::SkipCommand
            }
        } else if error.contains("alarm") || error.contains("emergency") {
            // Critical errors
            println!(
                "[RECOVERY] Classified as critical error (reset_on_critical: {})",
                self.recovery_config.reset_on_critical_error
            );
            if self.recovery_config.reset_on_critical_error {
                println!("[RECOVERY] Resetting controller due to critical error");
                crate::communication::RecoveryAction::ResetController
            } else {
                println!("[RECOVERY] Aborting job due to critical error");
                crate::communication::RecoveryAction::AbortJob
            }
        } else {
            // Unknown errors - try reset
            println!("[RECOVERY] Classified as unknown error, attempting controller reset");
            crate::communication::RecoveryAction::ResetController
        };

        self.recovery_state
            .recovery_actions_taken
            .push(action.clone());
        println!("[RECOVERY] Recovery action taken: {:?}", action);
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
}
