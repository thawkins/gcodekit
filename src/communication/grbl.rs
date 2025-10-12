use serialport::{available_ports, SerialPort};
use std::io::{Read, Write};
use chrono::Utc;

#[derive(Default, PartialEq, Debug, Clone)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

pub struct GrblCommunication {
    pub connection_state: ConnectionState,
    pub selected_port: String,
    pub available_ports: Vec<String>,
    pub status_message: String,
    pub grbl_version: String,
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
            serial_port: None,
        }
    }
}

impl GrblCommunication {
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
            let version_part = &response[version_start..];
            if let Some(end_pos) = version_part.find(" ") {
                let version = version_part[..end_pos].to_string();
                self.grbl_version = version;
                self.log_console(&format!("Detected GRBL version: {}", self.grbl_version));
            } else {
                // If no space found, take the whole "Grbl X.X" part
                self.grbl_version = version_part.to_string();
                self.log_console(&format!("Detected GRBL version: {}", self.grbl_version));
            }
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

    pub fn send_spindle_override(&mut self, value: f32) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // TODO: Send spindle override command to GRBL
        self.status_message = format!("Spindle override: {:.0}%", value);
    }

    pub fn send_feed_override(&mut self, value: f32) {
        if self.connection_state != ConnectionState::Connected {
            return;
        }
        // TODO: Send feed override command to GRBL
        self.status_message = format!("Feed override: {:.0}%", value);
    }

    fn log_console(&mut self, message: &str) {
        let timestamp = Utc::now().format("%H:%M:%S");
        // Note: Console logging is handled by the main app now
        // This is just for internal communication logging
        println!("[{}] {}", timestamp, message);
    }
}