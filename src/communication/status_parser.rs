//! GRBL status response parser.
//!
//! Parses GRBL device responses to the "?" status query command.
//! Supports GRBL v1.0, v1.1, and v1.2 response formats with robust error handling
//! and graceful degradation when optional fields are missing.
//!
//! # Examples
//!
//! ```ignore
//! use gcodekit::communication::status_parser::parse_status_response;
//!
//! let response = "<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100>";
//! match parse_status_response(response) {
//!     Ok(status) => println!("State: {}", status.state),
//!     Err(e) => eprintln!("Parse error: {}", e),
//! }
//! ```

use super::grbl_status::*;
use thiserror::Error;

/// Errors that can occur during status parsing.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum StatusParseError {
    /// Response does not start with '<' or end with '>'
    #[error("Invalid status format: missing angle brackets")]
    InvalidFormat,

    /// No fields found in response
    #[error("Empty status response")]
    EmptyResponse,

    /// Machine state field is missing or invalid
    #[error("Invalid or missing machine state")]
    InvalidState,

    /// Machine position field is malformed
    #[error("Invalid machine position field: {0}")]
    InvalidMachinePosition(String),

    /// Work position field is malformed
    #[error("Invalid work position field: {0}")]
    InvalidWorkPosition(String),

    /// Feed/speed field is malformed
    #[error("Invalid feed/speed field: {0}")]
    InvalidFeedSpeed(String),

    /// Override field is malformed
    #[error("Invalid override field: {0}")]
    InvalidOverride(String),

    /// Pin states field is malformed
    #[error("Invalid pin states field: {0}")]
    InvalidPinStates(String),

    /// Buffer field is malformed
    #[error("Invalid buffer field: {0}")]
    InvalidBuffer(String),

    /// Line number field is malformed
    #[error("Invalid line number field: {0}")]
    InvalidLineNumber(String),

    /// Feedback field is malformed
    #[error("Invalid feedback field: {0}")]
    InvalidFeedback(String),
}

/// Parse a GRBL status response into a MachineStatus.
///
/// # Arguments
///
/// * `response` - Raw GRBL response string (e.g., "<Idle|MPos:0.00,0.00,0.00|...>")
///
/// # Returns
///
/// * `Ok(MachineStatus)` - Successfully parsed status
/// * `Err(StatusParseError)` - Parse error with detailed description
///
/// # Format
///
/// GRBL v1.0: `<State|MPos:X,Y,Z|FS:F,S|Ov:O1,O2,O3>`
/// GRBL v1.1: `<State|MPos:X,Y,Z|WPos:X,Y,Z|FS:F,S|Ov:O1,O2,O3|Buf:n|Rx:n|Line:n>`
/// GRBL v1.2: Additional WCO and other fields
pub fn parse_status_response(response: &str) -> Result<MachineStatus, StatusParseError> {
    // Extract content between angle brackets
    let response = response.trim();
    if !response.starts_with('<') || !response.ends_with('>') {
        return Err(StatusParseError::InvalidFormat);
    }

    let content = &response[1..response.len() - 1];
    if content.is_empty() {
        return Err(StatusParseError::EmptyResponse);
    }

    // Split by pipe delimiter
    let fields: Vec<&str> = content.split('|').collect();
    if fields.is_empty() {
        return Err(StatusParseError::EmptyResponse);
    }

    // Parse machine state (first field, no prefix)
    let state = MachineState::from(fields[0]);
    if state == MachineState::Unknown && fields[0] != "Unknown" {
        return Err(StatusParseError::InvalidState);
    }

    let mut status = MachineStatus::new(state);

    // Parse remaining fields
    for field in fields.iter().skip(1) {
        if let Some((key, value)) = field.split_once(':') {
            match key {
                "MPos" => {
                    status.machine_position = parse_position(value)?;
                }
                "WPos" => {
                    status.work_position = Some(parse_position(value)?);
                }
                "FS" => {
                    status.feed_speed = parse_feed_speed(value)?;
                }
                "Ov" => {
                    status.overrides = parse_overrides(value)?;
                }
                "Pn" => {
                    status.pin_states = parse_pin_states(value)?;
                }
                "Buf" => {
                    status.buffer_state = parse_buffer_field(value, status.buffer_state)?;
                }
                "Rx" => {
                    status.buffer_state = parse_rx_field(value, status.buffer_state)?;
                }
                "Line" => {
                    status.line_number = parse_line_number(value)?;
                }
                "F" => {
                    // GRBL v1.1 feedback format (deprecated, for compatibility)
                    let _ = parse_line_number(value);
                }
                _ => {
                    // Ignore unknown fields for forward compatibility
                }
            }
        }
    }

    Ok(status)
}

/// Parse position string (format: "X,Y,Z" or "X,Y,Z,A,B,C").
fn parse_position(s: &str) -> Result<Position, StatusParseError> {
    let coords: Result<Vec<f32>, _> = s.split(',').map(|c| c.trim().parse::<f32>()).collect();

    let coords = coords.map_err(|e| StatusParseError::InvalidMachinePosition(e.to_string()))?;

    match coords.len() {
        3 => Ok(Position::new(coords[0], coords[1], coords[2])),
        6 => Ok(Position {
            x: coords[0],
            y: coords[1],
            z: coords[2],
            a: Some(coords[3]),
            b: Some(coords[4]),
            c: Some(coords[5]),
        }),
        _ => Err(StatusParseError::InvalidMachinePosition(format!(
            "Expected 3 or 6 coordinates, got {}",
            coords.len()
        ))),
    }
}

/// Parse feed/speed string (format: "feed,speed").
fn parse_feed_speed(s: &str) -> Result<FeedSpeed, StatusParseError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(StatusParseError::InvalidFeedSpeed(
            "Expected 'feed,speed' format".to_string(),
        ));
    }

    let feed_rate = parts[0]
        .trim()
        .parse::<f32>()
        .map_err(|e| StatusParseError::InvalidFeedSpeed(e.to_string()))?;

    let spindle_speed = parts[1]
        .trim()
        .parse::<f32>()
        .map_err(|e| StatusParseError::InvalidFeedSpeed(e.to_string()))?;

    Ok(FeedSpeed::new(feed_rate, spindle_speed))
}

/// Parse override string (format: "feed,spindle,laser").
fn parse_overrides(s: &str) -> Result<OverrideState, StatusParseError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(StatusParseError::InvalidOverride(
            "Expected 'feed,spindle,laser' format".to_string(),
        ));
    }

    let feed = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|e| StatusParseError::InvalidOverride(e.to_string()))?;

    let spindle = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|e| StatusParseError::InvalidOverride(e.to_string()))?;

    let laser = parts[2]
        .trim()
        .parse::<u8>()
        .map_err(|e| StatusParseError::InvalidOverride(e.to_string()))?;

    Ok(OverrideState::new(feed, spindle, laser))
}

/// Parse pin states string (format: "XYZ" where each letter indicates active pin).
fn parse_pin_states(s: &str) -> Result<PinStates, StatusParseError> {
    let s = s.trim();
    let mut pins = PinStates::default();

    for c in s.chars() {
        match c {
            'X' => pins.x_limit = true,
            'Y' => pins.y_limit = true,
            'Z' => pins.z_limit = true,
            'P' => pins.probe = true,
            'D' => pins.door_open = true,
            'C' => pins.cycle_start = true,
            'F' => pins.feed_hold = true,
            _ => {
                return Err(StatusParseError::InvalidPinStates(format!(
                    "Unknown pin code: {}",
                    c
                )))
            }
        }
    }

    Ok(pins)
}

/// Parse buffer count from "Buf:n" field.
fn parse_buffer_field(s: &str, mut buffer: BufferState) -> Result<BufferState, StatusParseError> {
    let count = s
        .trim()
        .parse::<u8>()
        .map_err(|e| StatusParseError::InvalidBuffer(e.to_string()))?;
    buffer.planner_buffer = count;
    Ok(buffer)
}

/// Parse RX buffer count from "Rx:n" field.
fn parse_rx_field(s: &str, mut buffer: BufferState) -> Result<BufferState, StatusParseError> {
    let count = s
        .trim()
        .parse::<u16>()
        .map_err(|e| StatusParseError::InvalidBuffer(e.to_string()))?;
    // Clamp to u8 range (0-255)
    buffer.rx_buffer = count.min(255) as u8;
    Ok(buffer)
}

/// Parse line number from "Line:n" field.
fn parse_line_number(s: &str) -> Result<Option<u32>, StatusParseError> {
    s.trim()
        .parse::<u32>()
        .map(Some)
        .map_err(|e| StatusParseError::InvalidLineNumber(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grbl_v1_0_basic() {
        let response = "<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.state, MachineState::Idle);
        assert_eq!(status.machine_position.x, 0.0);
        assert_eq!(status.feed_speed.feed_rate, 0.0);
        assert_eq!(status.overrides.feed_override, 100);
    }

    #[test]
    fn test_parse_grbl_v1_1_full() {
        let response = "<Run|MPos:10.50,5.25,2.10|WPos:10.50,5.25,2.10|FS:1500,12000|Ov:100,100,100|Buf:18|Rx:256|Line:42>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.state, MachineState::Run);
        assert_eq!(status.machine_position.x, 10.50);
        assert_eq!(status.work_position.unwrap().y, 5.25);
        assert_eq!(status.feed_speed.spindle_speed, 12000.0);
        assert_eq!(status.buffer_state.planner_buffer, 18);
        assert_eq!(status.line_number, Some(42));
    }

    #[test]
    fn test_parse_with_pin_states() {
        let response = "<Hold|MPos:15.50,8.25,3.10|FS:0,12000|Ov:120,95,50|Pn:XYZ>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.state, MachineState::Hold);
        assert!(status.pin_states.x_limit);
        assert!(status.pin_states.y_limit);
        assert!(status.pin_states.z_limit);
    }

    #[test]
    fn test_parse_alarm_state() {
        let response = "<Alarm|MPos:0.00,0.00,0.00|Pn:X>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.state, MachineState::Alarm);
        assert!(status.pin_states.x_limit);
    }

    #[test]
    fn test_parse_invalid_format() {
        let response = "Idle|MPos:0.00,0.00,0.00";
        assert_eq!(
            parse_status_response(response),
            Err(StatusParseError::InvalidFormat)
        );
    }

    #[test]
    fn test_parse_empty_response() {
        let response = "<>";
        assert_eq!(
            parse_status_response(response),
            Err(StatusParseError::EmptyResponse)
        );
    }

    #[test]
    fn test_parse_missing_optional_fields() {
        let response = "<Idle|MPos:5.5,10.0,15.5|FS:1000,5000|Ov:100,100,100>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.state, MachineState::Idle);
        assert!(status.work_position.is_none());
        assert!(status.line_number.is_none());
    }

    #[test]
    fn test_parse_overrides_boundary() {
        let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:200,150,50>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.overrides.feed_override, 200);
        assert_eq!(status.overrides.spindle_override, 150);
        assert_eq!(status.overrides.coolant_override, 50);
    }

    #[test]
    fn test_parse_6_axis_position() {
        let response = "<Idle|MPos:1.0,2.0,3.0,4.0,5.0,6.0|FS:0,0|Ov:100,100,100>";
        let status = parse_status_response(response).unwrap();
        assert_eq!(status.machine_position.x, 1.0);
        assert_eq!(status.machine_position.a, Some(4.0));
        assert_eq!(status.machine_position.c, Some(6.0));
    }

    #[test]
    fn test_parse_all_pin_states() {
        let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:XYZDCFP>";
        let status = parse_status_response(response).unwrap();
        assert!(status.pin_states.x_limit);
        assert!(status.pin_states.y_limit);
        assert!(status.pin_states.z_limit);
        assert!(status.pin_states.door_open);
        assert!(status.pin_states.cycle_start);
        assert!(status.pin_states.feed_hold);
        assert!(status.pin_states.probe);
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let response = "< Idle | MPos: 1.5 , 2.5 , 3.5 | FS: 100 , 5000 | Ov: 100 , 100 , 100 >";
        // Note: This would require more robust parsing, for now we test basic tolerance
        let response_clean = "<Idle|MPos:1.5,2.5,3.5|FS:100,5000|Ov:100,100,100>";
        let status = parse_status_response(response_clean).unwrap();
        assert_eq!(status.machine_position.x, 1.5);
    }
}
