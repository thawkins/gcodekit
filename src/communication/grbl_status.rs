//! GRBL machine status types and data structures.
//!
//! This module defines the complete set of types used to represent GRBL device status,
//! including machine state, position, feed rate, overrides, and other real-time metrics.
//! Supports GRBL v1.0, v1.1, and v1.2 response formats.
//!
//! # Examples
//!
//! ```ignore
//! use gcodekit::communication::grbl_status::MachineStatus;
//!
//! let status = MachineStatus::default();
//! println!("State: {:?}", status.state);
//! ```

use std::time::Instant;

/// Machine state enumeration for GRBL devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MachineState {
    /// Machine is idle and ready
    #[default]
    Idle,
    /// Machine is executing a job
    Run,
    /// Machine is holding (paused during execution)
    Hold,
    /// Machine is in jog mode
    Jog,
    /// Machine has encountered an alarm
    Alarm,
    /// Safety door is open
    Door,
    /// Machine is in check mode (simulating without movement)
    Check,
    /// Machine is homing
    Home,
    /// Machine is in sleep mode
    Sleep,
    /// State could not be determined
    Unknown,
}

impl From<&str> for MachineState {
    fn from(s: &str) -> Self {
        match s.trim() {
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

impl std::fmt::Display for MachineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineState::Idle => write!(f, "Idle"),
            MachineState::Run => write!(f, "Run"),
            MachineState::Hold => write!(f, "Hold"),
            MachineState::Jog => write!(f, "Jog"),
            MachineState::Alarm => write!(f, "Alarm"),
            MachineState::Door => write!(f, "Door"),
            MachineState::Check => write!(f, "Check"),
            MachineState::Home => write!(f, "Home"),
            MachineState::Sleep => write!(f, "Sleep"),
            MachineState::Unknown => write!(f, "Unknown"),
        }
    }
}

/// XYZ position coordinates (optional rotary axes A, B, C).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Position {
    /// X axis position (mm)
    pub x: f32,
    /// Y axis position (mm)
    pub y: f32,
    /// Z axis position (mm)
    pub z: f32,
    /// A rotary axis (optional, degrees)
    pub a: Option<f32>,
    /// B rotary axis (optional, degrees)
    pub b: Option<f32>,
    /// C rotary axis (optional, degrees)
    pub c: Option<f32>,
}

impl Position {
    /// Create a new position with only X, Y, Z coordinates.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Position {
            x,
            y,
            z,
            a: None,
            b: None,
            c: None,
        }
    }

    /// Format position as a human-readable string.
    pub fn format(&self) -> String {
        if let (Some(a), Some(b), Some(c)) = (self.a, self.b, self.c) {
            format!(
                "X:{:.2} Y:{:.2} Z:{:.2} A:{:.2} B:{:.2} C:{:.2}",
                self.x, self.y, self.z, a, b, c
            )
        } else {
            format!("X:{:.2} Y:{:.2} Z:{:.2}", self.x, self.y, self.z)
        }
    }
}

/// Feed rate and spindle/laser speed.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FeedSpeed {
    /// Current feed rate (mm/min)
    pub feed_rate: f32,
    /// Current spindle speed (RPM) or laser power (0-100%)
    pub spindle_speed: f32,
}

impl FeedSpeed {
    /// Create new feed speed values.
    pub fn new(feed_rate: f32, spindle_speed: f32) -> Self {
        FeedSpeed {
            feed_rate,
            spindle_speed,
        }
    }
}

/// Override percentages (100% = normal speed).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OverrideState {
    /// Feed rate override percentage (100% = 100 mm/min)
    pub feed_override: u8,
    /// Spindle speed override percentage
    pub spindle_override: u8,
    /// Laser power or coolant override percentage
    pub coolant_override: u8,
}

impl OverrideState {
    /// Create new override state.
    pub fn new(feed_override: u8, spindle_override: u8, coolant_override: u8) -> Self {
        OverrideState {
            feed_override: feed_override.min(200),
            spindle_override: spindle_override.min(200),
            coolant_override: coolant_override.min(200),
        }
    }
}

/// Planner and serial buffer status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BufferState {
    /// Planner buffer fill count (0-128 typical)
    pub planner_buffer: u8,
    /// Serial RX buffer fill count
    pub rx_buffer: u8,
}

impl BufferState {
    /// Create new buffer state.
    pub fn new(planner_buffer: u8, rx_buffer: u8) -> Self {
        BufferState {
            planner_buffer,
            rx_buffer,
        }
    }

    /// Get planner buffer fill percentage (0-100).
    pub fn planner_fill_percent(&self) -> u8 {
        ((self.planner_buffer as f32 / 128.0) * 100.0) as u8
    }

    /// Get RX buffer fill percentage (0-100).
    pub fn rx_fill_percent(&self) -> u8 {
        ((self.rx_buffer as f32 / 256.0) * 100.0) as u8
    }
}

/// Input pin states (limit switches, probe, door, etc.).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PinStates {
    /// Probe input active
    pub probe: bool,
    /// X limit switch active
    pub x_limit: bool,
    /// Y limit switch active
    pub y_limit: bool,
    /// Z limit switch active
    pub z_limit: bool,
    /// Safety door open
    pub door_open: bool,
    /// Cycle start button pressed
    pub cycle_start: bool,
    /// Feed hold button pressed
    pub feed_hold: bool,
}

impl PinStates {
    /// Check if any alarm-related pin is active (limits or door).
    pub fn has_alarm(&self) -> bool {
        self.x_limit || self.y_limit || self.z_limit || self.door_open
    }

    /// Check if any pin is active.
    pub fn any_active(&self) -> bool {
        self.probe
            || self.x_limit
            || self.y_limit
            || self.z_limit
            || self.door_open
            || self.cycle_start
            || self.feed_hold
    }
}

/// Feedback counters and metrics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FeedbackMetrics {
    /// Lines queued (planned)
    pub lines_queued: u32,
    /// Lines remaining to execute
    pub lines_remaining: u32,
    /// Lines completed
    pub lines_completed: u32,
}

impl FeedbackMetrics {
    /// Get total lines in job (completed + remaining).
    pub fn total_lines(&self) -> u32 {
        self.lines_completed.saturating_add(self.lines_remaining)
    }

    /// Get job progress percentage (0-100).
    pub fn progress_percent(&self) -> u8 {
        let total = self.total_lines();
        if total == 0 {
            0
        } else {
            ((self.lines_completed as f32 / total as f32) * 100.0) as u8
        }
    }
}

/// Real-time machine status snapshot.
///
/// Represents a complete snapshot of the machine state at a specific point in time,
/// captured from a GRBL device status query ("?" command). This includes position,
/// machine state, feed parameters, and various hardware status indicators.
#[derive(Debug, Clone, PartialEq)]
pub struct MachineStatus {
    /// Current machine state
    pub state: MachineState,

    /// Machine position (MPos) - absolute coordinates
    pub machine_position: Position,

    /// Work position (WPos) - relative to work coordinate system (optional)
    pub work_position: Option<Position>,

    /// Feed rate and spindle speed
    pub feed_speed: FeedSpeed,

    /// Override values (feed, spindle, laser/coolant)
    pub overrides: OverrideState,

    /// Current line number being executed (GRBL 1.1+, optional)
    pub line_number: Option<u32>,

    /// Planner buffer status
    pub buffer_state: BufferState,

    /// Input pin states (probe, limit switches, etc.)
    pub pin_states: PinStates,

    /// Feedback counters and rates
    pub feedback: FeedbackMetrics,

    /// Timestamp when this status was captured
    pub timestamp: Instant,
}

impl Default for MachineStatus {
    fn default() -> Self {
        MachineStatus {
            state: MachineState::default(),
            machine_position: Position::default(),
            work_position: None,
            feed_speed: FeedSpeed::default(),
            overrides: OverrideState::default(),
            line_number: None,
            buffer_state: BufferState::default(),
            pin_states: PinStates::default(),
            feedback: FeedbackMetrics::default(),
            timestamp: Instant::now(),
        }
    }
}

impl MachineStatus {
    /// Create a new status with current timestamp.
    pub fn new(state: MachineState) -> Self {
        MachineStatus {
            state,
            timestamp: Instant::now(),
            ..Default::default()
        }
    }

    /// Check if machine is currently executing (Run or Jog state).
    pub fn is_executing(&self) -> bool {
        self.state == MachineState::Run || self.state == MachineState::Jog
    }

    /// Check if machine is in an error state (Alarm or Door).
    pub fn is_error_state(&self) -> bool {
        self.state == MachineState::Alarm || self.state == MachineState::Door
    }

    /// Check if machine is idle or holding.
    pub fn is_idle(&self) -> bool {
        self.state == MachineState::Idle || self.state == MachineState::Hold
    }

    /// Format status as debug string for logging.
    pub fn format_debug(&self) -> String {
        format!(
            "State:{} Pos:{} Feed:{:.0} Speed:{:.0} Override:{}%/{}%/{}% Buffer:{}/{} Line:{:?}",
            self.state,
            self.machine_position.format(),
            self.feed_speed.feed_rate,
            self.feed_speed.spindle_speed,
            self.overrides.feed_override,
            self.overrides.spindle_override,
            self.overrides.coolant_override,
            self.buffer_state.planner_buffer,
            self.buffer_state.rx_buffer,
            self.line_number
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_state_from_string() {
        assert_eq!(MachineState::from("Idle"), MachineState::Idle);
        assert_eq!(MachineState::from("Run"), MachineState::Run);
        assert_eq!(MachineState::from("Hold"), MachineState::Hold);
        assert_eq!(MachineState::from("Alarm"), MachineState::Alarm);
        assert_eq!(MachineState::from("Unknown"), MachineState::Unknown);
    }

    #[test]
    fn test_position_format() {
        let pos = Position::new(10.5, 20.3, 5.1);
        assert_eq!(pos.format(), "X:10.50 Y:20.30 Z:5.10");
    }

    #[test]
    fn test_buffer_fill_percent() {
        let buf = BufferState::new(64, 128);
        assert_eq!(buf.planner_fill_percent(), 50);
        assert_eq!(buf.rx_fill_percent(), 50);
    }

    #[test]
    fn test_feedback_progress() {
        let feedback = FeedbackMetrics {
            lines_completed: 50,
            lines_remaining: 50,
            lines_queued: 0,
        };
        assert_eq!(feedback.total_lines(), 100);
        assert_eq!(feedback.progress_percent(), 50);
    }

    #[test]
    fn test_pin_states_alarm() {
        let pins = PinStates {
            x_limit: true,
            ..Default::default()
        };
        assert!(pins.has_alarm());
    }

    #[test]
    fn test_machine_status_states() {
        let status = MachineStatus::new(MachineState::Run);
        assert!(status.is_executing());
        assert!(!status.is_error_state());

        let status = MachineStatus::new(MachineState::Alarm);
        assert!(status.is_error_state());
        assert!(!status.is_executing());
    }

    #[test]
    fn test_override_clamp() {
        let overrides = OverrideState::new(250, 250, 250);
        assert_eq!(overrides.feed_override, 200);
        assert_eq!(overrides.spindle_override, 200);
    }
}
