use chrono::Utc;
use serialport::{SerialPort, available_ports};
use std::collections::VecDeque;
use std::io::{Read, Write};

#[derive(Default, PartialEq, Debug, Clone)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
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
    Unknown,
}

impl Default for MachineState {
    fn default() -> Self {
        MachineState::Unknown
    }
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

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Default)]
pub struct GrblStatus {
    pub machine_state: MachineState,
    pub machine_position: Position, // MPos
    pub work_position: Position,    // WPos
    pub feed_rate: Option<f32>,
    pub spindle_speed: Option<f32>,
    pub line_number: Option<u32>,
    pub input_pin_state: Option<String>,
    pub override_values: Option<String>,
}

#[derive(Debug, Clone)]
pub enum GrblResponse {
    Ok,
    Error(String),
    Status(GrblStatus),
    Feedback(String),
    Alarm(String),
    Version(String),
    Settings(String),
    Other(String),
}

pub struct GrblCommunication {
    pub connection_state: ConnectionState,
    pub selected_port: String,
    pub available_ports: Vec<String>,
    pub status_message: String,
    pub grbl_version: String,
    pub current_status: GrblStatus,
    pub gcode_queue: VecDeque<String>,
    pub last_response: Option<GrblResponse>,
    serial_port: Option<Box<dyn SerialPort>>,
}

impl Default for GrblCommunication {
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
            serial_port: None,
        }
    }
}

impl GrblCommunication {
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
                if self.available_ports.is_empty() {
                    self.status_message = "No serial ports found".to_string();
                } else {
                    self.status_message = format!("Found {} ports", self.available_ports.len());
                }
            }
            Err(e) => {
                self.status_message = format!("Error listing ports: {}", e);
                self.connection_state = ConnectionState::Error;
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

                // Read initial GRBL response to get version
                std::thread::sleep(std::time::Duration::from_millis(100));
                self.read_grbl_version();

                self.status_message = format!("Connected to {} at 115200 baud", self.selected_port);

                // Send initial commands to wake up GRBL
                self.send_grbl_command("\r\n\r\n");
                std::thread::sleep(std::time::Duration::from_millis(2000));
                self.send_grbl_command("$$\n"); // Get settings
            }
            Err(e) => {
                self.connection_state = ConnectionState::Error;
                self.status_message = format!("Failed to connect: {}", e);
            }
        }
    }

    pub fn disconnect_from_device(&mut self) {
        self.serial_port = None;
        self.connection_state = ConnectionState::Disconnected;
        self.grbl_version.clear();
        self.status_message = "Disconnected from device".to_string();
    }

    pub fn send_grbl_command(&mut self, command: &str) {
        if let Some(ref mut port) = self.serial_port {
            match port.write_all(command.as_bytes()) {
                Ok(_) => {
                    self.log_console(&format!("Sent: {}", command.trim()));
                }
                Err(e) => {
                    self.log_console(&format!("Send error: {}", e));
                }
            }
        }
    }

    pub fn read_grbl_responses(&mut self) -> Vec<String> {
        let mut messages = Vec::new();

        if let Some(ref mut port) = self.serial_port {
            let mut buffer = [0; 1024];
            match port.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    if let Ok(response) = std::str::from_utf8(&buffer[..bytes_read]) {
                        let clean_response = response.trim();
                        if !clean_response.is_empty() {
                            messages.push(format!("Recv: {}", clean_response));

                            // Check if this is a version response
                            if clean_response.contains("Grbl") {
                                self.parse_grbl_version(clean_response);
                            }
                        }
                    }
                }
                _ => {} // No data or error, ignore for now
            }
        }

        messages
    }

    pub fn read_grbl_version(&mut self) {
        let mut version_response = None;

        if let Some(ref mut port) = self.serial_port {
            let mut buffer = [0; 1024];
            // Try to read version info multiple times in case GRBL sends it slowly
            for _ in 0..5 {
                match port.read(&mut buffer) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        if let Ok(response) = std::str::from_utf8(&buffer[..bytes_read]) {
                            let clean_response = response.trim();
                            if !clean_response.is_empty() {
                                version_response = Some(clean_response.to_string());
                                break;
                            }
                        }
                    }
                    _ => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }

        // Handle version response outside the borrow scope
        if let Some(clean_response) = version_response {
            self.log_console(&format!("Recv: {}", clean_response));

            // Check if this is a version response
            if clean_response.contains("Grbl") {
                self.parse_grbl_version(&clean_response);
            }
        }
    }

    pub fn parse_grbl_version(&mut self, response: &str) {
        // GRBL typically responds with something like: "Grbl 1.1f ['$' for help]"
        if let Some(version_start) = response.find("Grbl ") {
            let after_grbl = &response[version_start + 5..]; // Skip "Grbl "
            // Find the end of the version (space, bracket, or end of string)
            let end_pos = after_grbl
                .find(' ')
                .or_else(|| after_grbl.find('['))
                .unwrap_or(after_grbl.len());

            let version = format!("Grbl {}", &after_grbl[..end_pos]);
            self.grbl_version = version;
            self.log_console(&format!("Detected GRBL version: {}", self.grbl_version));
        }
    }

    pub fn jog_axis(&mut self, axis: char, distance: f32) {
        if self.connection_state != ConnectionState::Connected {
            self.status_message = "Not connected to device".to_string();
            return;
        }

        // Send GRBL jog command ($J=G91 X10 F1000)
        let command = format!("$J=G91 {} {:.1} F1000\n", axis, distance);
        self.send_grbl_command(&command);
        self.status_message = format!("Jogging {} axis by {:.1}mm", axis, distance);
    }

    pub fn home_all_axes(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            self.status_message = "Not connected to device".to_string();
            return;
        }

        // Send GRBL home command ($H)
        self.send_grbl_command("$H\n");
        self.status_message = "Homing all axes".to_string();
    }

    pub fn send_feed_override(&mut self, value: f32) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // Send real-time feed override command (0x90 + percentage)
        let override_value = (value.clamp(10.0, 200.0) as u8).saturating_sub(100);
        let command = format!("{}{}", char::from(0x90), override_value as char);
        self.send_grbl_command(&command);
        self.status_message = format!("Feed override: {:.0}%", value);
    }

    pub fn send_spindle_override(&mut self, value: f32) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // Send real-time spindle override command (0x9A + percentage)
        let override_value = (value.clamp(10.0, 200.0) as u8).saturating_sub(100);
        let command = format!("{}{}", char::from(0x9A), override_value as char);
        self.send_grbl_command(&command);
        self.status_message = format!("Spindle override: {:.0}%", value);
    }

    pub fn query_realtime_status(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // Send real-time status query command (?)
        self.send_grbl_command("?");
    }

    pub fn parse_grbl_status(&mut self, response: &str) -> Option<GrblStatus> {
        // GRBL status response format: <State|MPos:X,Y,Z|WPos:X,Y,Z|F:feed|S:speed>
        if !response.starts_with('<') || !response.ends_with('>') {
            return None;
        }

        let content = &response[1..response.len() - 1];
        let parts: Vec<&str> = content.split('|').collect();

        if parts.is_empty() {
            return None;
        }

        let mut status = GrblStatus::default();
        status.machine_state = MachineState::from(parts[0]);

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

        Some(status)
    }

    fn parse_position(&self, pos_str: &str) -> Option<Position> {
        let coords: Vec<&str> = pos_str.split(',').collect();
        if coords.len() >= 3 {
            if let (Ok(x), Ok(y), Ok(z)) = (
                coords[0].parse::<f32>(),
                coords[1].parse::<f32>(),
                coords[2].parse::<f32>(),
            ) {
                return Some(Position { x, y, z });
            }
        }
        None
    }

    pub fn parse_grbl_response(&mut self, response: &str) -> GrblResponse {
        let trimmed = response.trim();

        if trimmed == "ok" {
            GrblResponse::Ok
        } else if trimmed.starts_with("error:") {
            GrblResponse::Error(trimmed[6..].to_string())
        } else if trimmed.starts_with("ALARM:") {
            GrblResponse::Alarm(trimmed[6..].to_string())
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Feedback message
            GrblResponse::Feedback(trimmed[1..trimmed.len() - 1].to_string())
        } else if trimmed.starts_with("Grbl ") {
            GrblResponse::Version(trimmed.to_string())
        } else if trimmed.starts_with('$') || trimmed.contains('=') {
            // Settings response
            GrblResponse::Settings(trimmed.to_string())
        } else if trimmed.starts_with('<') && trimmed.ends_with('>') {
            // Status response
            if let Some(status) = self.parse_grbl_status(trimmed) {
                self.current_status = status.clone();
                GrblResponse::Status(status)
            } else {
                GrblResponse::Other(trimmed.to_string())
            }
        } else {
            GrblResponse::Other(trimmed.to_string())
        }
    }

    pub fn send_gcode_line(&mut self, line: &str) -> Result<(), String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected to device".to_string());
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(());
        }

        // Add to queue for flow control
        self.gcode_queue.push_back(trimmed.to_string());

        // For now, send immediately (TODO: implement proper queuing)
        self.send_grbl_command(trimmed);

        Ok(())
    }

    pub fn get_grbl_settings(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        self.send_grbl_command("$$\n");
    }

    pub fn set_grbl_setting(&mut self, setting: u32, value: f32) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        let command = format!("${}={}\n", setting, value);
        self.send_grbl_command(&command);
    }

    pub fn feed_hold(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // Send feed hold command (!)
        self.send_grbl_command("!");
        self.status_message = "Feed hold activated".to_string();
    }

    pub fn resume(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // Send resume command (~)
        self.send_grbl_command("~");
        self.status_message = "Resumed operation".to_string();
    }

    pub fn reset_grbl(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // Send reset command (Ctrl+X = 0x18)
        self.send_grbl_command(&format!("{}", char::from(0x18)));
        self.status_message = "GRBL reset sent".to_string();
    }

    fn log_console(&mut self, message: &str) {
        let timestamp = Utc::now().format("%H:%M:%S");
        // Note: Console logging is handled by the main app now
        // This is just for internal communication logging
        println!("[{}] {}", timestamp, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grbl_communication_new() {
        let comm = GrblCommunication::new();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert!(comm.selected_port.is_empty());
        assert!(comm.available_ports.is_empty());
        assert!(comm.status_message.is_empty());
        assert!(comm.grbl_version.is_empty());
        assert!(comm.serial_port.is_none());
    }

    #[test]
    fn test_parse_grbl_version() {
        let mut comm = GrblCommunication::new();

        // Test standard GRBL version response
        comm.parse_grbl_version("Grbl 1.1f ['$' for help]");
        assert_eq!(comm.grbl_version, "Grbl 1.1f");

        // Test version without space
        comm.parse_grbl_version("Grbl 1.2");
        assert_eq!(comm.grbl_version, "Grbl 1.2");

        // Test no version found
        comm.parse_grbl_version("Some other response");
        assert_eq!(comm.grbl_version, "Grbl 1.2"); // Should remain unchanged
    }

    #[test]
    fn test_jog_command_formatting() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test X axis jog
        comm.jog_axis('X', 10.0);
        assert_eq!(comm.status_message, "Jogging X axis by 10.0mm");

        // Test Y axis jog with decimal
        comm.jog_axis('Y', -5.5);
        assert_eq!(comm.status_message, "Jogging Y axis by -5.5mm");

        // Test Z axis jog
        comm.jog_axis('Z', 2.25);
        assert_eq!(comm.status_message, "Jogging Z axis by 2.2mm");
    }

    #[test]
    fn test_connection_state_management() {
        let mut comm = GrblCommunication::new();

        // Initial state
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);

        // Test disconnect when already disconnected
        comm.disconnect_from_device();
        assert_eq!(comm.connection_state, ConnectionState::Disconnected);
        assert_eq!(comm.status_message, "Disconnected from device".to_string());
    }

    #[test]
    fn test_home_command() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        comm.home_all_axes();
        assert_eq!(comm.status_message, "Homing all axes".to_string());
    }

    #[test]
    fn test_override_commands() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test spindle override
        comm.send_spindle_override(75.0);
        assert_eq!(comm.status_message, "Spindle override: 75%");

        // Test feed override
        comm.send_feed_override(120.0);
        assert_eq!(comm.status_message, "Feed override: 120%");

        // Test decimal values
        comm.send_spindle_override(33.7);
        assert_eq!(comm.status_message, "Spindle override: 34%");
    }

    #[test]
    fn test_disconnected_operations() {
        let mut comm = GrblCommunication::new();
        // Ensure disconnected
        comm.connection_state = ConnectionState::Disconnected;

        // Test jog when disconnected
        comm.jog_axis('X', 10.0);
        assert_eq!(comm.status_message, "Not connected to device".to_string());

        // Test home when disconnected
        comm.home_all_axes();
        assert_eq!(comm.status_message, "Not connected to device".to_string());

        // Test overrides when disconnected (should not change status)
        let original_message = comm.status_message.clone();
        comm.send_spindle_override(50.0);
        assert_eq!(comm.status_message, original_message); // Should remain unchanged
    }

    #[test]
    fn test_parse_grbl_status() {
        let mut comm = GrblCommunication::new();

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
        let mut comm = GrblCommunication::new();

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
    fn test_parse_grbl_response_types() {
        let mut comm = GrblCommunication::new();

        // Test OK response
        let response = comm.parse_grbl_response("ok");
        assert!(matches!(response, GrblResponse::Ok));

        // Test error response
        let response = comm.parse_grbl_response("error: Invalid command");
        assert!(matches!(response, GrblResponse::Error(_)));

        // Test alarm response
        let response = comm.parse_grbl_response("ALARM: Hard limit");
        assert!(matches!(response, GrblResponse::Alarm(_)));

        // Test feedback response
        let response = comm.parse_grbl_response("[MSG: Test message]");
        assert!(matches!(response, GrblResponse::Feedback(_)));

        // Test version response
        let response = comm.parse_grbl_response("Grbl 1.1f");
        assert!(matches!(response, GrblResponse::Version(_)));

        // Test settings response
        let response = comm.parse_grbl_response("$0=10");
        assert!(matches!(response, GrblResponse::Settings(_)));
    }

    #[test]
    fn test_machine_state_parsing() {
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
    fn test_realtime_override_commands() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test feed override 150%
        comm.send_feed_override(150.0);
        // Should send 0x91 (145) which is 150 - 100 = 50, but clamped and adjusted
        assert_eq!(comm.status_message, "Feed override: 150%");

        // Test spindle override 50%
        comm.send_spindle_override(50.0);
        assert_eq!(comm.status_message, "Spindle override: 50%");

        // Test boundary values
        comm.send_feed_override(10.0); // Minimum
        assert_eq!(comm.status_message, "Feed override: 10%");

        comm.send_feed_override(200.0); // Maximum
        assert_eq!(comm.status_message, "Feed override: 200%");
    }

    #[test]
    fn test_control_commands() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test feed hold
        comm.feed_hold();
        assert_eq!(comm.status_message, "Feed hold activated");

        // Test resume
        comm.resume();
        assert_eq!(comm.status_message, "Resumed operation");

        // Test reset
        comm.reset_grbl();
        assert_eq!(comm.status_message, "GRBL reset sent");
    }

    #[test]
    fn test_gcode_line_sending() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test successful sending
        let result = comm.send_gcode_line("G1 X10 Y20 F100");
        assert!(result.is_ok());

        // Test empty line
        let result = comm.send_gcode_line("");
        assert!(result.is_ok());

        // Test disconnected state
        comm.connection_state = ConnectionState::Disconnected;
        let result = comm.send_gcode_line("G1 X10");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not connected to device");
    }
}
