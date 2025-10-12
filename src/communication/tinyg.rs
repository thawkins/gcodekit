use crate::communication::{CncController, ErrorRecoveryConfig};
use serialport::SerialPort;
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Clone, Debug, PartialEq)]
pub enum TinyGResponse {
    Ok,
    Error(String),
    Status {
        pos: (f32, f32, f32, f32),
        vel: f32,
        stat: u8,
    },
    Feedback(String),
    Version(String),
    Settings(String),
    Other(String),
}

#[derive(Clone, Debug)]
pub enum WcsCoordinate {
    G54,
    G55,
    G56,
    G57,
    G58,
    G59,
}

#[derive(Clone, Debug)]
pub struct TinyGCommunication {
    pub port: Option<Box<dyn SerialPort>>,
    pub connection_state: ConnectionState,
    pub status_message: String,
    pub current_wcs: WcsCoordinate,
    last_response: Option<TinyGResponse>,
    response_sender: Sender<String>,
    response_receiver: Receiver<String>,
    error_recovery_config: ErrorRecoveryConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl Default for TinyGCommunication {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            port: None,
            connection_state: ConnectionState::Disconnected,
            status_message: "Disconnected".to_string(),
            current_wcs: WcsCoordinate::G54,
            last_response: None,
            response_sender: tx,
            response_receiver: rx,
            error_recovery_config: ErrorRecoveryConfig::default(),
        }
    }
}

impl TinyGCommunication {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connect(&mut self, port_name: &str, baud_rate: u32) -> Result<(), String> {
        self.connection_state = ConnectionState::Connecting;
        self.status_message = "Connecting...".to_string();

        match serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()
        {
            Ok(port) => {
                self.port = Some(port);
                self.connection_state = ConnectionState::Connected;
                self.status_message = format!("Connected to {} at {} baud", port_name, baud_rate);
                info!("Connected to TinyG on {}", port_name);

                // Start response reader thread
                self.start_response_reader();

                Ok(())
            }
            Err(e) => {
                self.connection_state = ConnectionState::Error(e.to_string());
                self.status_message = format!("Connection failed: {}", e);
                Err(format!("Failed to connect: {}", e))
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.port = None;
        self.connection_state = ConnectionState::Disconnected;
        self.status_message = "Disconnected".to_string();
        info!("Disconnected from TinyG");
    }

    fn start_response_reader(&self) {
        let port = self.port.as_ref().unwrap().try_clone().unwrap();
        let sender = self.response_sender.clone();

        thread::spawn(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match port.read(&mut buffer) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                        let lines: Vec<&str> = response.split('\n').collect();
                        for line in lines {
                            let line = line.trim();
                            if !line.is_empty() {
                                if let Err(_) = sender.send(line.to_string()) {
                                    break; // Channel closed
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        // Timeout is expected, continue
                    }
                    Err(e) => {
                        warn!("Error reading from TinyG: {}", e);
                        break;
                    }
                    _ => {}
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn send_gcode_line(&mut self, gcode: &str) -> Result<(), String> {
        if self.connection_state != ConnectionState::Connected {
            return Err("Not connected".to_string());
        }

        let command = format!("{}\n", gcode.trim());
        match self.port.as_mut().unwrap().write_all(command.as_bytes()) {
            Ok(()) => {
                info!("Sent to TinyG: {}", gcode);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to send command: {}", e);
                warn!("{}", err_msg);
                Err(err_msg)
            }
        }
    }

    pub fn read_response(&mut self) -> Option<TinyGResponse> {
        if let Ok(response) = self.response_receiver.try_recv() {
            let parsed = self.parse_tinyg_response(&response);
            self.last_response = Some(parsed.clone());
            Some(parsed)
        } else {
            None
        }
    }

    fn parse_tinyg_response(&self, response: &str) -> TinyGResponse {
        // Basic TinyG JSON response parsing
        // TinyG uses JSON format for responses
        if response.contains("\"r\":") {
            // Status response
            // This is a simplified parser - real implementation would use serde_json
            if response.contains("\"posx\":") {
                // Extract position (simplified)
                TinyGResponse::Status {
                    pos: (0.0, 0.0, 0.0, 0.0), // Parse actual values
                    vel: 0.0,
                    stat: 0,
                }
            } else {
                TinyGResponse::Ok
            }
        } else if response.starts_with("{\"er\":") {
            TinyGResponse::Error("TinyG error".to_string())
        } else if response.starts_with("{\"fb\":") {
            TinyGResponse::Feedback(response.to_string())
        } else {
            TinyGResponse::Other(response.to_string())
        }
    }

    pub fn query_realtime_status(&mut self) {
        let _ = self.send_gcode_line("{\"sr\":\"\"}"); // Status report request
    }

    pub fn get_tinyg_settings(&mut self) {
        let _ = self.send_gcode_line("{\"sys\":\"\"}"); // System settings
    }

    pub fn set_tinyg_setting(&mut self, group: &str, key: &str, value: f32) {
        let cmd = format!("{{\"{}\":{{\"{}\":{}}}}}", group, key, value);
        let _ = self.send_gcode_line(&cmd);
    }

    pub fn feed_hold(&mut self) {
        let _ = self.send_gcode_line("!"); // Feed hold
    }

    pub fn resume(&mut self) {
        let _ = self.send_gcode_line("~"); // Resume
    }

    pub fn reset_tinyg(&mut self) {
        let _ = self.send_gcode_line("\x18"); // Ctrl+X reset
    }
}

impl CncController for TinyGCommunication {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn connect(&mut self, port: &str, baud: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.connect(port, baud).map_err(|e| e.into())
    }

    fn disconnect(&mut self) {
        self.disconnect();
    }

    fn send_gcode(&mut self, gcode: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.send_gcode_line(gcode).map_err(|e| e.into())
    }

    fn get_status(&self) -> String {
        self.status_message.clone()
    }

    fn set_recovery_config(&mut self, config: ErrorRecoveryConfig) {
        self.error_recovery_config = config;
    }
}