use chrono::Utc;
use serialport::{available_ports, DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::any::Any;
use std::collections::VecDeque;
use std::error::Error;
use std::io::{Read, Write};
use tracing::{debug, info};

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

#[derive(Debug, Clone, PartialEq)]
pub enum QueueState {
    Idle,
    WaitingForAck,
    Paused,
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

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: Option<f32>,
    pub b: Option<f32>,
    pub c: Option<f32>,
    pub d: Option<f32>,
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
    pub is_sending_job: bool,
    pub last_response: Option<GrblResponse>,
    pub current_wcs: WcsCoordinate,
    pub recovery_config: crate::communication::ErrorRecoveryConfig,
    pub recovery_state: crate::communication::RecoveryState,
    pub health_metrics: crate::communication::HealthMetrics,
    pub debug_enabled: bool,
    feed_hold_sent: bool,
    serial_port: Option<Box<dyn SerialPort>>,
    queue_state: QueueState,
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
            is_sending_job: false,
            last_response: None,
            current_wcs: WcsCoordinate::G54,
            recovery_config: crate::communication::ErrorRecoveryConfig::default(),
            recovery_state: crate::communication::RecoveryState::default(),
            health_metrics: crate::communication::HealthMetrics::default(),
            debug_enabled: false,
            feed_hold_sent: false,
            serial_port: None,
            queue_state: QueueState::Idle,
        }
    }
}

impl GrblCommunication {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            selected_port: String::new(),
            available_ports: Vec::new(),
            status_message: String::new(),
            grbl_version: String::new(),
            current_status: GrblStatus::default(),
            gcode_queue: VecDeque::new(),
            is_sending_job: false,
            last_response: None,
            current_wcs: WcsCoordinate::G54,
            recovery_config: crate::communication::ErrorRecoveryConfig::default(),
            recovery_state: crate::communication::RecoveryState {
                reconnect_attempts: 0,
                last_reconnect_attempt: None,
                command_retry_count: 0,
                last_error: None,
                recovery_actions_taken: Vec::new(),
            },
            health_metrics: crate::communication::HealthMetrics::default(),
            debug_enabled: false,
            feed_hold_sent: false,
            serial_port: None,
            queue_state: QueueState::Idle,
        }
    }

    pub fn set_wcs(&mut self, wcs: WcsCoordinate) -> Result<(), String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected to device".to_string());
        }

        let gcode = match wcs {
            WcsCoordinate::G54 => "G54",
            WcsCoordinate::G55 => "G55",
            WcsCoordinate::G56 => "G56",
            WcsCoordinate::G57 => "G57",
            WcsCoordinate::G58 => "G58",
            WcsCoordinate::G59 => "G59",
        };

        self.send_grbl_command(gcode);
        self.current_wcs = wcs.clone();
        self.status_message = format!("Switched to {:?}", wcs);
        Ok(())
    }

    /// Check if a port name matches patterns commonly used by GRBL devices
    fn is_grbl_port(&self, port_name: &str) -> bool {
        // Check for common GRBL port patterns
        if port_name.starts_with("/dev/ttyACM") {
            // "/dev/ttyACM" is 11 chars, check if next char is digit
            port_name
                .chars()
                .nth(11)
                .is_some_and(|c| c.is_ascii_digit())
        } else if port_name.starts_with("/dev/ttyUSB") {
            // "/dev/ttyUSB" is 11 chars, check if next char is digit
            port_name
                .chars()
                .nth(11)
                .is_some_and(|c| c.is_ascii_digit())
        } else if port_name.starts_with("COM") {
            // "COM" is 3 chars, check if next char is digit
            port_name.chars().nth(3).is_some_and(|c| c.is_ascii_digit())
        } else if port_name.starts_with("/dev/tty.usbserial") {
            // "/dev/tty.usbserial" is 18 chars, check if next char is digit
            port_name
                .chars()
                .nth(18)
                .is_some_and(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    pub fn refresh_ports(&mut self) {
        self.available_ports.clear();
        match available_ports() {
            Ok(ports) => {
                for port in ports {
                    let port_name = port.port_name;
                    // Filter ports to only include those likely to have GRBL devices
                    if self.is_grbl_port(&port_name) {
                        self.available_ports.push(port_name);
                    }
                }
                if self.available_ports.is_empty() {
                    self.status_message = "No compatible serial ports found".to_string();
                } else {
                    self.status_message =
                        format!("Found {} compatible ports", self.available_ports.len());
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
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
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
                if self.debug_enabled {
                    debug!("DEBUG: Sending wake-up commands to GRBL");
                }
                self.send_grbl_command("\n\n");
                std::thread::sleep(std::time::Duration::from_millis(2000));

                // Send initialization commands that require proper acknowledgment
                // Use send_gcode_line for commands that need "ok" responses
                if self.debug_enabled {
                    debug!("DEBUG: Sending $X command to unlock GRBL if in alarm state");
                }
                if let Err(e) = self.send_gcode_line("$X") {
                    // Unlock GRBL
                    if self.debug_enabled {
                        debug!("DEBUG: Failed to send unlock command: {}", e);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));

                if self.debug_enabled {
                    debug!("DEBUG: Sending G21 command to set units to mm");
                }
                if let Err(e) = self.send_gcode_line("G21") {
                    // Set units to mm
                    if self.debug_enabled {
                        debug!("DEBUG: Failed to send G21 command: {}", e);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(200));

                if self.debug_enabled {
                    debug!("DEBUG: Sending G90 command to set absolute positioning");
                }
                if let Err(e) = self.send_gcode_line("G90") {
                    // Set absolute positioning
                    if self.debug_enabled {
                        debug!("DEBUG: Failed to send G90 command: {}", e);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(200));

                // Send query commands to get GRBL state (these don't need queuing)
                if self.debug_enabled {
                    debug!("DEBUG: Sending $$ command to get GRBL settings");
                }
                self.send_grbl_command("$$\n"); // Get settings
                std::thread::sleep(std::time::Duration::from_millis(500));

                if self.debug_enabled {
                    debug!("DEBUG: Sending $G command to get GRBL parser state");
                }
                self.send_grbl_command("$G\n"); // Get parser state
                std::thread::sleep(std::time::Duration::from_millis(500));

                if self.debug_enabled {
                    debug!("DEBUG: Sending ? command to get GRBL status");
                }
                self.send_grbl_command("?\n"); // Get status
                std::thread::sleep(std::time::Duration::from_millis(500));
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
        // Reset queue state on disconnect
        self.queue_state = QueueState::Idle;
        self.gcode_queue.clear();
    }

    pub fn send_grbl_command(&mut self, command: &str) {
        let port_available = self.serial_port.is_some();
        let command_bytes = command.as_bytes();
        if self.debug_enabled {
            debug!(
                "DEBUG: send_grbl_command: Attempting to send: '{}' (port available: {})",
                command.trim(),
                port_available
            );
        }

        if port_available {
            if self.debug_enabled {
                debug!(
                    "DEBUG: send_grbl_command: Port is available, calling write_all with {} bytes",
                    command_bytes.len()
                );
            }

            // Perform the write operation
            let write_result = if let Some(ref mut port) = self.serial_port {
                let result = port.write_all(command_bytes);
                if result.is_ok() {
                    // Flush to ensure data is sent immediately
                    let _ = port.flush();
                }
                result
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Port not available",
                ))
            };

            if self.debug_enabled {
                match write_result {
                    Ok(_) => {
                        debug!(
                            "DEBUG: send_grbl_command: write_all succeeded for: '{}'",
                            command.trim()
                        );
                        debug!("DEBUG: Sent: {}", command.trim());
                    }
                    Err(e) => {
                        debug!(
                            "DEBUG: send_grbl_command: write_all failed with error: {}",
                            e
                        );
                        debug!("DEBUG: Send error: {}", e);
                    }
                }
            }
        } else if self.debug_enabled {
            debug!("DEBUG: send_grbl_command: No serial port available!");
        }
    }

    pub fn read_grbl_responses(&mut self) -> Vec<String> {
        let mut messages = Vec::new();

        if self.serial_port.is_some() {
            self.log_console("read_grbl_responses: Attempting to read from serial port");
            let mut buffer = [0; 1024];
            let read_result = if let Some(ref mut port) = self.serial_port {
                port.read(&mut buffer)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Port not available",
                ))
            };

            match read_result {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        self.log_console(&format!(
                            "read_grbl_responses: Read {} bytes",
                            bytes_read
                        ));
                        if let Ok(response) = std::str::from_utf8(&buffer[..bytes_read]) {
                            let clean_response = response.trim();
                            self.log_console(&format!(
                                "read_grbl_responses: Decoded response: '{}'",
                                clean_response
                            ));
                            if !clean_response.is_empty() {
                                messages.push(clean_response.to_string());

                                // Check if this is a version response
                                if clean_response.contains("Grbl") {
                                    self.parse_grbl_version(clean_response);
                                }
                            }
                        } else {
                            self.log_console("read_grbl_responses: Failed to decode UTF-8");
                        }
                    } else {
                        self.log_console("read_grbl_responses: No bytes read (EOF or timeout)");
                    }
                }
                Err(e) => {
                    self.log_console(&format!("read_grbl_responses: Read error: {}", e));
                }
            }
        } else {
            self.log_console("read_grbl_responses: No serial port available");
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
        let command = format!("$J=G91 {} {:.3} F1000\n", axis, distance);
        self.send_grbl_command(&command);
        self.status_message = format!("Jogging {} axis by {:.3}mm", axis, distance);
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

    pub fn parse_grbl_status(&mut self, response: &str) -> Result<GrblStatus, String> {
        // GRBL status response format: <State|MPos:X,Y,Z|WPos:X,Y,Z|F:feed|S:speed>
        if !response.starts_with('<') || !response.ends_with('>') {
            return Err("Invalid status format".to_string());
        }

        let content = &response[1..response.len() - 1];
        let parts: Vec<&str> = content.split('|').collect();

        if parts.is_empty() {
            return Err("Empty status content".to_string());
        }

        let machine_state = MachineState::from(parts[0]);
        if machine_state == MachineState::Unknown {
            return Err(format!("Unknown machine state: {}", parts[0]));
        }

        let mut status = GrblStatus {
            machine_state,
            ..Default::default()
        };

        for part in &parts[1..] {
            if let Some(colon_pos) = part.find(':') {
                let key = &part[..colon_pos];
                let value = &part[colon_pos + 1..];

                match key {
                    "MPos" => match self.parse_position(value) {
                        Some(pos) => status.machine_position = pos,
                        None => return Err(format!("Failed to parse MPos: {}", value)),
                    },
                    "WPos" => match self.parse_position(value) {
                        Some(pos) => status.work_position = pos,
                        None => return Err(format!("Failed to parse WPos: {}", value)),
                    },
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
                    "FS" => {
                        // Handle combined feed and spindle: FS:feed,spindle
                        if let Some(comma_pos) = value.find(',') {
                            let feed_str = &value[..comma_pos];
                            let speed_str = &value[comma_pos + 1..];
                            if let Ok(feed) = feed_str.parse::<f32>() {
                                status.feed_rate = Some(feed);
                            }
                            if let Ok(speed) = speed_str.parse::<f32>() {
                                status.spindle_speed = Some(speed);
                            }
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

        Ok(status)
    }

    fn parse_position(&self, pos_str: &str) -> Option<Position> {
        let coords: Vec<&str> = pos_str.split(',').collect();
        if coords.len() >= 3 {
            if let (Ok(x), Ok(y), Ok(z)) = (
                coords[0].parse::<f32>(),
                coords[1].parse::<f32>(),
                coords[2].parse::<f32>(),
            ) {
                let mut pos = Position {
                    x,
                    y,
                    z,
                    a: None,
                    b: None,
                    c: None,
                    d: None,
                };

                // Parse additional axes if available
                if coords.len() >= 4 {
                    if let Ok(a) = coords[3].parse::<f32>() {
                        pos.a = Some(a);
                    }
                }
                if coords.len() >= 5 {
                    if let Ok(b) = coords[4].parse::<f32>() {
                        pos.b = Some(b);
                    }
                }
                if coords.len() >= 6 {
                    if let Ok(c) = coords[5].parse::<f32>() {
                        pos.c = Some(c);
                    }
                }
                if coords.len() >= 7 {
                    if let Ok(d) = coords[6].parse::<f32>() {
                        pos.d = Some(d);
                    }
                }

                return Some(pos);
            }
        }
        None
    }

    pub fn parse_grbl_response(&mut self, response: &str) -> GrblResponse {
        let trimmed = response.trim();

        if trimmed == "ok" {
            GrblResponse::Ok
        } else if let Some(stripped) = trimmed.strip_prefix("error:") {
            GrblResponse::Error(stripped.to_string())
        } else if let Some(stripped) = trimmed.strip_prefix("ALARM:") {
            GrblResponse::Alarm(stripped.to_string())
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
            match self.parse_grbl_status(trimmed) {
                Ok(status) => {
                    self.current_status = status.clone();
                    GrblResponse::Status(status)
                }
                Err(_) => GrblResponse::Other(trimmed.to_string()),
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
        let was_empty = self.gcode_queue.is_empty();
        if self.debug_enabled {
            debug!(
                "DEBUG: send_gcode_line: Command '{}' - queue was_empty: {}, queue len: {}, queue_state: {:?}",
                trimmed,
                was_empty,
                self.gcode_queue.len(),
                self.queue_state
            );
        }
        self.gcode_queue.push_back(trimmed.to_string());
        if self.debug_enabled {
            debug!(
                "DEBUG: send_gcode_line: After adding, queue len: {}",
                self.gcode_queue.len()
            );
        }

        // If queue was empty and we're idle, send immediately
        if was_empty && self.queue_state == QueueState::Idle {
            if self.debug_enabled {
                debug!("DEBUG: send_gcode_line: Sending '{}' immediately", trimmed);
            }
            self.send_grbl_command(&format!("{}\r\n", trimmed));
            self.queue_state = QueueState::WaitingForAck;
        } else if self.debug_enabled {
            debug!(
                "DEBUG: send_gcode_line: Adding '{}' to queue (state: {:?})",
                trimmed, self.queue_state
            );
        }

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

    pub fn probe_axis(&mut self, axis: char, distance: f32, feed: f32) -> Result<(), String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }
        let command = format!("G38.2 {}{:.3} F{:.0}", axis, distance, feed);
        self.send_gcode_line(&command)
    }

    pub fn probe_z_down(&mut self, distance: f32, feed: f32) -> Result<(), String> {
        self.probe_axis('Z', -distance.abs(), feed)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn auto_level_grid(
        &mut self,
        x_start: f32,
        y_start: f32,
        x_end: f32,
        y_end: f32,
        grid_size: usize,
        probe_depth: f32,
        feed: f32,
    ) -> Result<Vec<(f32, f32, f32)>, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let mut probe_points = Vec::new();

        let x_step = (x_end - x_start) / (grid_size - 1) as f32;
        let y_step = (y_end - y_start) / (grid_size - 1) as f32;

        for i in 0..grid_size {
            for j in 0..grid_size {
                let x = x_start + i as f32 * x_step;
                let y = y_start + j as f32 * y_step;

                // Move to probe position
                self.send_gcode_line(&format!("G0 X{:.3} Y{:.3}", x, y))?;

                // Probe Z
                self.probe_z_down(probe_depth, feed)?;

                // Get probe result (this would need to parse the response)
                // For now, assume we get the Z position
                // In real implementation, parse the probe result from status
                let z = 0.0; // Placeholder

                probe_points.push((x, y, z));
            }
        }

        Ok(probe_points)
    }

    pub fn measure_workpiece(
        &mut self,
        axis: char,
        direction: f32,
        feed: f32,
    ) -> Result<f32, String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let command = format!("G38.2 {}{:.3} F{:.0}", axis, direction, feed);
        self.send_gcode_line(&command)?;

        // Parse the probe result
        // This would need to read the status and extract the probed position
        // For now, return a placeholder
        Ok(0.0)
    }

    fn log_console(&mut self, message: &str) {
        let timestamp = Utc::now().format("%H:%M:%S");
        // Note: Console logging is handled by the main app now
        // This is just for internal communication logging
        info!("[{}] {}", timestamp, message);
    }
}

impl CncController for GrblCommunication {
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

    fn handle_response(&mut self, response: &str) -> Option<crate::MachinePosition> {
        if self.debug_enabled {
            debug!(
                "DEBUG: handle_response: Received response: '{}'",
                response.trim()
            );
        }
        let parsed = self.parse_grbl_response(response);
        if self.debug_enabled {
            debug!("DEBUG: handle_response: Parsed as: {:?}", parsed);
        }
        // Handle the response as needed, e.g., update status
        match parsed {
            GrblResponse::Ok => {
                if self.debug_enabled {
                    debug!(
                        "DEBUG: handle_response: Got OK, queue length: {}, queue_state: {:?}",
                        self.gcode_queue.len(),
                        self.queue_state
                    );
                }
                // Mark current command as completed
                self.queue_state = QueueState::Idle;

                // Send next line from queue if available
                if let Some(next_line) = self.gcode_queue.pop_front() {
                    if self.debug_enabled {
                        debug!(
                            "DEBUG: Queue: Sending next command from queue: '{}'",
                            next_line
                        );
                        let command_with_ending = format!("{}\r\n", next_line);
                        debug!(
                            "DEBUG: Queue: Full command being sent: {:?}",
                            command_with_ending.as_bytes()
                        );
                    }
                    let command_with_ending = format!("{}\r\n", next_line);
                    self.send_grbl_command(&command_with_ending);
                    self.queue_state = QueueState::WaitingForAck;
                } else if self.debug_enabled {
                    debug!("DEBUG: Queue: No more commands in queue");
                }
                None
            }
            GrblResponse::Status(status) => {
                if self.debug_enabled {
                    debug!(
                        "DEBUG: handle_response: GRBL status - State: {:?}, Position: ({:.3}, {:.3}, {:.3})",
                        status.machine_state,
                        status.work_position.x,
                        status.work_position.y,
                        status.work_position.z
                    );
                }
                self.current_status = status.clone();
                let pos = status.work_position;
                Some(crate::MachinePosition {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    a: pos.a,
                    b: pos.b,
                    c: pos.c,
                    d: pos.d,
                })
            }
            GrblResponse::Error(error_msg) => {
                if self.debug_enabled {
                    debug!(
                        "DEBUG: handle_response: Got ERROR: {}, resetting queue state",
                        error_msg
                    );
                }
                self.queue_state = QueueState::Idle; // Allow next command to be sent
                None
            }
            GrblResponse::Alarm(alarm_msg) => {
                if self.debug_enabled {
                    debug!(
                        "DEBUG: handle_response: Got ALARM: {}, pausing queue",
                        alarm_msg
                    );
                }
                self.queue_state = QueueState::Paused; // Pause queue on alarm
                None
            }
            _ => None,
        }
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
        if self.connection_state == ConnectionState::Recovering {
            self.connection_state = ConnectionState::Connected;
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

        // Update health metrics
        self.health_metrics.update_health_scores();

        // Check connection stability
        if self.health_metrics.connection_stability < 0.8 {
            warnings.push(format!(
                "Connection stability: {:.1}% - Consider checking connections",
                self.health_metrics.connection_stability * 100.0
            ));
        }

        // Check command success rate
        if self.health_metrics.command_success_rate < 0.9 {
            warnings.push(format!(
                "Command success rate: {:.1}% - System may need attention",
                self.health_metrics.command_success_rate * 100.0
            ));
        }

        // Check for error patterns
        warnings.extend(self.health_metrics.predict_potential_issues());

        // Check GRBL status
        if self.current_status.machine_state == MachineState::Alarm {
            warnings.push("GRBL is in alarm state - Clear alarm before proceeding".to_string());
        }

        warnings
    }

    fn optimize_settings_based_on_health(&mut self) -> Vec<String> {
        let mut optimizations = Vec::new();

        // If connection is unstable, suggest reducing baud rate
        if self.health_metrics.connection_stability < 0.7 {
            optimizations
                .push("Consider reducing serial baud rate for better stability".to_string());
        }

        // If many command errors, suggest checking G-code
        if self.health_metrics.command_success_rate < 0.8 {
            optimizations
                .push("Frequent command errors detected - validate G-code syntax".to_string());
        }

        // Check for timeout patterns and suggest timeout adjustments
        if let Some(timeout_pattern) = self
            .health_metrics
            .error_patterns
            .iter()
            .find(|p| p.error_type.contains("timeout"))
        {
            if timeout_pattern.frequency > 3 {
                optimizations.push(
                    "Frequent timeouts detected - consider increasing command timeout".to_string(),
                );
            }
        }

        optimizations
    }

    fn emergency_stop(&mut self) {
        // Send feed hold command to GRBL
        let _ = self.send_gcode_line("!");
    }

    fn send_raw_command(&mut self, command: &str) {
        self.send_grbl_command(command);
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
        assert_eq!(comm.status_message, "Jogging X axis by 10.000mm");

        // Test Y axis jog with decimal
        comm.jog_axis('Y', -5.5);
        assert_eq!(comm.status_message, "Jogging Y axis by -5.500mm");

        // Test Z axis jog
        comm.jog_axis('Z', 2.25);
        assert_eq!(comm.status_message, "Jogging Z axis by 2.250mm");
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
        let status = comm
            .parse_grbl_status(status_str)
            .expect("failed to parse status string");

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
        let status = comm
            .parse_grbl_status(status_str)
            .expect("failed to parse status string");

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
        let response = comm.parse_grbl_response("error:1");
        assert!(matches!(response, GrblResponse::Error(_)));

        // Test status response
        let response = comm.parse_grbl_response("<Idle|MPos:0.000,0.000,0.000|FS:0,0>");
        assert!(matches!(response, GrblResponse::Status { .. }));

        // Test feedback response
        let response = comm.parse_grbl_response("[MSG:Test message]");
        assert!(matches!(response, GrblResponse::Feedback(_)));

        // Test version response
        let response = comm.parse_grbl_response("Grbl 1.1f ['$' for help]");
        assert!(matches!(response, GrblResponse::Version(_)));

        // Test settings response
        let response = comm.parse_grbl_response("$0=10");
        assert!(matches!(response, GrblResponse::Settings(_)));

        // Test alarm response
        let response = comm.parse_grbl_response("ALARM:1");
        assert!(matches!(response, GrblResponse::Alarm(_)));

        // Test other response
        let response = comm.parse_grbl_response("Some other response");
        assert!(matches!(response, GrblResponse::Other(_)));
    }

    #[test]
    fn test_parse_grbl_response_edge_cases() {
        let mut comm = GrblCommunication::new();

        // Test empty response
        let response = comm.parse_grbl_response("");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test whitespace only
        let response = comm.parse_grbl_response("   ");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test malformed status response
        let response = comm.parse_grbl_response("<>");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test status response with missing fields
        let response = comm.parse_grbl_response("<Idle>");
        assert!(matches!(response, GrblResponse::Status { .. }));

        // Test error response without colon
        let response = comm.parse_grbl_response("error");
        assert!(matches!(response, GrblResponse::Other(_)));

        // Test alarm response without colon
        let response = comm.parse_grbl_response("ALARM");
        assert!(matches!(response, GrblResponse::Other(_)));
    }

    #[test]
    fn test_parse_grbl_status_complex() {
        let mut comm = GrblCommunication::new();

        // Test complex status response with all fields
        let status = comm.parse_grbl_status("<Run|MPos:10.500,20.750,5.250|WPos:0.000,0.000,0.000|FS:1500,200|WCO:0.000,0.000,0.000>").expect("failed to parse example status");

        assert_eq!(status.machine_state, MachineState::Run);
        assert_eq!(status.machine_position.x, 10.5);
        assert_eq!(status.machine_position.y, 20.75);
        assert_eq!(status.machine_position.z, 5.25);
        assert_eq!(status.work_position.x, 0.0);
        assert_eq!(status.work_position.y, 0.0);
        assert_eq!(status.work_position.z, 0.0);
        assert_eq!(status.feed_rate, Some(1500.0));
        assert_eq!(status.spindle_speed, Some(200.0));
    }

    #[test]
    fn test_parse_grbl_status_invalid() {
        let mut comm = GrblCommunication::new();

        // Test invalid status responses
        assert!(comm.parse_grbl_status("").is_err());
        assert!(comm.parse_grbl_status("not a status").is_err());
        assert!(comm.parse_grbl_status("<>").is_err());
        assert!(comm.parse_grbl_status("<InvalidState>").is_err());
    }

    #[test]
    fn test_machine_state_from_string() {
        // Test all valid machine states
        assert_eq!(MachineState::from("Idle"), MachineState::Idle);
        assert_eq!(MachineState::from("Run"), MachineState::Run);
        assert_eq!(MachineState::from("Hold"), MachineState::Hold);
        assert_eq!(MachineState::from("Jog"), MachineState::Jog);
        assert_eq!(MachineState::from("Alarm"), MachineState::Alarm);
        assert_eq!(MachineState::from("Door"), MachineState::Door);
        assert_eq!(MachineState::from("Check"), MachineState::Check);
        assert_eq!(MachineState::from("Home"), MachineState::Home);
        assert_eq!(MachineState::from("Sleep"), MachineState::Sleep);

        // Test unknown state defaults to Unknown
        assert_eq!(MachineState::from("Unknown"), MachineState::Unknown);
        assert_eq!(MachineState::from(""), MachineState::Unknown);
    }

    #[test]
    fn test_jog_command_edge_cases() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test very small jog distance
        comm.jog_axis('X', 0.001);
        assert_eq!(comm.status_message, "Jogging X axis by 0.001mm");

        // Test negative jog distance
        comm.jog_axis('Y', -5.0);
        assert_eq!(comm.status_message, "Jogging Y axis by -5.000mm");

        // Test large jog distance
        comm.jog_axis('Z', 1000.0);
        assert_eq!(comm.status_message, "Jogging Z axis by 1000.000mm");

        // Test different axes
        comm.jog_axis('A', 45.0);
        assert_eq!(comm.status_message, "Jogging A axis by 45.000mm");

        comm.jog_axis('B', -30.0);
        assert_eq!(comm.status_message, "Jogging B axis by -30.000mm");
    }

    #[test]
    fn test_override_commands_edge_cases() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test zero override
        comm.send_spindle_override(0.0);
        assert_eq!(comm.status_message, "Spindle override: 0%");

        // Test maximum override
        comm.send_feed_override(200.0);
        assert_eq!(comm.status_message, "Feed override: 200%");

        // Test negative override (should still work)
        comm.send_spindle_override(-10.0);
        assert_eq!(comm.status_message, "Spindle override: -10%");

        // Test very high override
        comm.send_feed_override(500.0);
        assert_eq!(comm.status_message, "Feed override: 500%");
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

    #[test]
    fn test_error_recovery_config() {
        let mut comm = GrblCommunication::new();
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
        let comm = GrblCommunication::new();
        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 0);
        assert!(state.last_reconnect_attempt.is_none());
        assert_eq!(state.command_retry_count, 0);
        assert!(state.last_error.is_none());
        assert!(state.recovery_actions_taken.is_empty());
    }

    #[test]
    fn test_recovery_connection_error() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Error;

        // Test recovery for connection error
        let result = comm.attempt_recovery("connection lost");
        assert!(result.is_ok());
        let action = result.expect("attempt_recovery failed");
        assert_eq!(action, crate::communication::RecoveryAction::Reconnect);

        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 1);
        assert!(state.last_reconnect_attempt.is_some());
        assert_eq!(
            state.last_error.as_ref().expect("expected last_error set"),
            "connection lost"
        );
        assert_eq!(state.recovery_actions_taken.len(), 1);
        assert_eq!(
            state.recovery_actions_taken[0],
            crate::communication::RecoveryAction::Reconnect
        );
    }

    #[test]
    fn test_recovery_command_error() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test recovery for command error
        let result = comm.attempt_recovery("Command syntax error");
        assert!(result.is_ok());
        let action = result.expect("attempt_recovery failed");
        assert_eq!(action, crate::communication::RecoveryAction::RetryCommand);

        let state = comm.get_recovery_state();
        assert_eq!(state.command_retry_count, 1);
        assert_eq!(
            state.last_error.as_ref().expect("expected last_error set"),
            "Command syntax error"
        );
        assert_eq!(state.recovery_actions_taken.len(), 1);
        assert_eq!(
            state.recovery_actions_taken[0],
            crate::communication::RecoveryAction::RetryCommand
        );
    }

    #[test]
    fn test_recovery_max_attempts_exceeded() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Error;

        // Exhaust reconnect attempts
        for _ in 0..3 {
            assert!(comm.attempt_recovery("connection lost").is_ok());
        }

        // Next attempt should abort job
        let result = comm.attempt_recovery("connection lost");
        assert!(result.is_ok());
        let action = result.expect("attempt_recovery failed");
        assert_eq!(action, crate::communication::RecoveryAction::AbortJob);

        let state = comm.get_recovery_state();
        assert_eq!(state.reconnect_attempts, 3);
    }

    #[test]
    fn test_recovery_command_max_retries_exceeded() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Exhaust command retries
        for _ in 0..3 {
            assert!(comm.attempt_recovery("Command syntax error").is_ok());
        }

        // Next attempt should skip command
        let result = comm.attempt_recovery("Command syntax error");
        assert!(result.is_ok());
        let action = result.expect("attempt_recovery failed");
        assert_eq!(action, crate::communication::RecoveryAction::SkipCommand);

        let state = comm.get_recovery_state();
        assert_eq!(state.command_retry_count, 3);
    }

    #[test]
    fn test_recovery_critical_error() {
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Connected;

        // Test recovery for critical error
        let result = comm.attempt_recovery("alarm: hard limit triggered");
        assert!(result.is_ok());
        let action = result.expect("attempt_recovery failed");
        assert_eq!(
            action,
            crate::communication::RecoveryAction::ResetController
        );

        let state = comm.get_recovery_state();
        assert_eq!(
            state.last_error.as_ref().expect("expected last_error set"),
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
        let mut comm = GrblCommunication::new();
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
        let mut comm = GrblCommunication::new();
        comm.connection_state = ConnectionState::Error;

        // Perform some recovery actions
        assert!(comm.attempt_recovery("connection lost").is_ok());
        assert!(comm.attempt_recovery("command timeout").is_ok());

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
        let mut comm = GrblCommunication::new();

        // Initially not recovering
        assert!(!comm.is_recovering());

        // After recovery attempt, should be recovering
        comm.connection_state = ConnectionState::Error;
        assert!(comm.attempt_recovery("connection lost").is_ok());
        assert!(comm.is_recovering());

        // After reset, should not be recovering
        comm.reset_recovery_state();
        assert!(!comm.is_recovering());
    }

    #[test]
    fn test_is_grbl_port_filtering() {
        let comm = GrblCommunication::new();

        // Test valid GRBL ports
        assert!(comm.is_grbl_port("/dev/ttyACM0"));
        assert!(comm.is_grbl_port("/dev/ttyACM1"));
        assert!(comm.is_grbl_port("/dev/ttyUSB0"));
        assert!(comm.is_grbl_port("/dev/ttyUSB5"));
        assert!(comm.is_grbl_port("COM1"));
        assert!(comm.is_grbl_port("COM9"));
        assert!(comm.is_grbl_port("/dev/tty.usbserial0"));
        assert!(comm.is_grbl_port("/dev/tty.usbserial3"));

        // Test invalid ports (no digit after prefix)
        assert!(!comm.is_grbl_port("/dev/ttyACM"));
        assert!(!comm.is_grbl_port("/dev/ttyUSB"));
        assert!(!comm.is_grbl_port("COM"));
        assert!(!comm.is_grbl_port("/dev/tty.usbserial"));

        // Test ports with non-digit characters
        assert!(!comm.is_grbl_port("/dev/ttyACMa"));
        assert!(!comm.is_grbl_port("/dev/ttyUSBx"));
        assert!(!comm.is_grbl_port("COMa"));
        assert!(!comm.is_grbl_port("/dev/tty.usbserialx"));

        // Test completely different port names
        assert!(!comm.is_grbl_port("/dev/ttyS0"));
        assert!(!comm.is_grbl_port("/dev/tty0"));
        assert!(!comm.is_grbl_port("ttyACM0"));
        assert!(!comm.is_grbl_port("USB0"));
    }
}
