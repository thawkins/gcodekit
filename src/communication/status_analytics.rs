//! Status monitoring analytics and trend analysis.
//!
//! Provides analysis functions for status history including trend calculation,
//! state transition tracking, and performance metrics.

use crate::communication::grbl_status::{MachineState, MachineStatus};
use std::time::Duration;

/// State transition event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransition {
    /// Previous state
    pub from: MachineState,
    /// New state
    pub to: MachineState,
    /// Duration spent in previous state
    pub duration: Duration,
}

/// Status analytics and metrics.
#[derive(Debug, Clone)]
pub struct StatusAnalytics {
    /// Average feed rate (mm/min) over history
    pub avg_feedrate: f32,
    /// Peak feed rate reached
    pub peak_feedrate: f32,
    /// Minimum feed rate
    pub min_feedrate: f32,
    /// Average spindle speed (RPM)
    pub avg_spindle_speed: f32,
    /// Peak spindle speed
    pub peak_spindle_speed: f32,
    /// Average buffer fill percentage
    pub avg_buffer_fill: f32,
    /// Peak buffer fill
    pub peak_buffer_fill: u8,
    /// State transitions in history
    pub state_transitions: Vec<StateTransition>,
    /// Time spent in each state
    pub state_durations: Vec<(MachineState, u32)>,
    /// Error count during period
    pub error_count: u32,
    /// Sample count analyzed
    pub sample_count: u32,
}

impl Default for StatusAnalytics {
    fn default() -> Self {
        StatusAnalytics {
            avg_feedrate: 0.0,
            peak_feedrate: 0.0,
            min_feedrate: 0.0,
            avg_spindle_speed: 0.0,
            peak_spindle_speed: 0.0,
            avg_buffer_fill: 0.0,
            peak_buffer_fill: 0,
            state_transitions: Vec::new(),
            state_durations: Vec::new(),
            error_count: 0,
            sample_count: 0,
        }
    }
}

/// Calculate analytics from status history.
pub fn analyze_status_history(statuses: &[MachineStatus]) -> StatusAnalytics {
    if statuses.is_empty() {
        return StatusAnalytics::default();
    }

    let mut analytics = StatusAnalytics {
        sample_count: statuses.len() as u32,
        ..Default::default()
    };

    // Calculate feed rate statistics
    let feedrates: Vec<f32> = statuses.iter().map(|s| s.feed_speed.feed_rate).collect();
    if !feedrates.is_empty() {
        analytics.avg_feedrate = feedrates.iter().sum::<f32>() / feedrates.len() as f32;
        analytics.peak_feedrate = feedrates
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        analytics.min_feedrate = feedrates
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
    }

    // Calculate spindle speed statistics
    let spindle_speeds: Vec<f32> = statuses
        .iter()
        .map(|s| s.feed_speed.spindle_speed)
        .collect();
    if !spindle_speeds.is_empty() {
        analytics.avg_spindle_speed =
            spindle_speeds.iter().sum::<f32>() / spindle_speeds.len() as f32;
        analytics.peak_spindle_speed = spindle_speeds
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
    }

    // Calculate buffer statistics
    let buffer_fills: Vec<f32> = statuses
        .iter()
        .map(|s| s.buffer_state.planner_fill_percent() as f32)
        .collect();
    if !buffer_fills.is_empty() {
        analytics.avg_buffer_fill = buffer_fills.iter().sum::<f32>() / buffer_fills.len() as f32;
        analytics.peak_buffer_fill = statuses
            .iter()
            .map(|s| s.buffer_state.planner_buffer)
            .max()
            .unwrap_or(0);
    }

    // Track state transitions
    let mut current_state = statuses[0].state;
    let mut state_count = 1u32;
    let mut transitions = Vec::new();

    for status in &statuses[1..] {
        if status.state != current_state {
            // Add duration for previous state
            analytics.state_durations.push((current_state, state_count));

            // Record transition
            transitions.push((current_state, status.state));

            current_state = status.state;
            state_count = 1;
        } else {
            state_count += 1;
        }
    }

    // Add final state duration
    analytics.state_durations.push((current_state, state_count));

    // Convert transition pairs to StateTransition objects
    // Assume each sample is ~250ms at default interval
    for (from, to) in transitions {
        analytics.state_transitions.push(StateTransition {
            from,
            to,
            duration: Duration::from_millis(state_count as u64 * 250),
        });
    }

    analytics
}

/// Get statistics about specific machine state.
#[derive(Debug, Clone)]
pub struct StateDuration {
    /// State
    pub state: MachineState,
    /// Number of samples in this state
    pub sample_count: u32,
    /// Approximate duration (samples × 250ms)
    pub duration_ms: u64,
}

/// Get state durations from history.
pub fn get_state_durations(statuses: &[MachineStatus]) -> Vec<StateDuration> {
    let analytics = analyze_status_history(statuses);

    analytics
        .state_durations
        .into_iter()
        .map(|(state, count)| StateDuration {
            state,
            sample_count: count,
            duration_ms: count as u64 * 250,
        })
        .collect()
}

/// Calculate job progress from feedback metrics.
pub fn calculate_progress(statuses: &[MachineStatus]) -> Option<f32> {
    if statuses.is_empty() {
        return None;
    }

    let latest = &statuses[statuses.len() - 1];
    let total = latest.feedback.total_lines();

    if total == 0 {
        return None;
    }

    Some(latest.feedback.progress_percent() as f32)
}

/// Detect alarms or error states in history.
pub fn detect_alarms(statuses: &[MachineStatus]) -> Vec<usize> {
    statuses
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_error_state())
        .map(|(i, _)| i)
        .collect()
}

/// Find state change indices.
pub fn find_state_changes(statuses: &[MachineStatus]) -> Vec<(usize, MachineState, MachineState)> {
    if statuses.len() < 2 {
        return Vec::new();
    }

    let mut changes = Vec::new();
    let mut prev_state = statuses[0].state;

    for (i, status) in statuses.iter().enumerate().skip(1) {
        if status.state != prev_state {
            changes.push((i, prev_state, status.state));
            prev_state = status.state;
        }
    }

    changes
}

/// Calculate position change (distance traveled).
pub fn calculate_position_change(statuses: &[MachineStatus]) -> Option<f32> {
    if statuses.len() < 2 {
        return None;
    }

    let first = &statuses[0].machine_position;
    let last = &statuses[statuses.len() - 1].machine_position;

    let dx = last.x - first.x;
    let dy = last.y - first.y;
    let dz = last.z - first.z;

    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

/// Estimate remaining time based on current feedrate and lines remaining.
pub fn estimate_job_time(status: &MachineStatus) -> Option<Duration> {
    if status.feedback.lines_remaining == 0 {
        return None;
    }

    if status.feed_speed.feed_rate == 0.0 {
        return None;
    }

    // Rough estimate: assume average 10 units per line
    let distance_remaining = status.feedback.lines_remaining as f32 * 10.0;
    let time_seconds = distance_remaining / status.feed_speed.feed_rate * 60.0;

    Some(Duration::from_secs_f32(time_seconds))
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_status(
        state: MachineState,
        x: f32,
        y: f32,
        z: f32,
        feedrate: f32,
    ) -> MachineStatus {
        let mut status = MachineStatus::new(state);
        status.machine_position.x = x;
        status.machine_position.y = y;
        status.machine_position.z = z;
        status.feed_speed.feed_rate = feedrate;
        status
    }

    #[test]
    fn test_analyze_empty_history() {
        let analytics = analyze_status_history(&[]);
        assert_eq!(analytics.sample_count, 0);
        assert_eq!(analytics.avg_feedrate, 0.0);
    }

    #[test]
    fn test_analyze_single_status() {
        let status = create_test_status(MachineState::Run, 10.0, 20.0, 5.0, 1000.0);
        let analytics = analyze_status_history(&[status]);

        assert_eq!(analytics.sample_count, 1);
        assert_eq!(analytics.avg_feedrate, 1000.0);
        assert_eq!(analytics.peak_feedrate, 1000.0);
    }

    #[test]
    fn test_feedrate_statistics() {
        let statuses = vec![
            create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 100.0),
            create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 200.0),
            create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 150.0),
        ];

        let analytics = analyze_status_history(&statuses);
        assert_eq!(analytics.avg_feedrate, 150.0);
        assert_eq!(analytics.peak_feedrate, 200.0);
        assert_eq!(analytics.min_feedrate, 100.0);
    }

    #[test]
    fn test_state_transitions() {
        let statuses = vec![
            create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0),
            create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0),
            create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1000.0),
            create_test_status(MachineState::Hold, 2.5, 2.5, 0.0, 0.0),
            create_test_status(MachineState::Idle, 2.5, 2.5, 0.0, 0.0),
        ];

        let analytics = analyze_status_history(&statuses);
        assert_eq!(analytics.state_transitions.len(), 3);
        assert_eq!(analytics.state_transitions[0].from, MachineState::Idle);
        assert_eq!(analytics.state_transitions[0].to, MachineState::Run);
    }

    #[test]
    fn test_detect_alarms() {
        let statuses = vec![
            create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0),
            create_test_status(MachineState::Alarm, 0.0, 0.0, 0.0, 0.0),
            create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0),
        ];

        let alarms = detect_alarms(&statuses);
        assert_eq!(alarms.len(), 1);
        assert_eq!(alarms[0], 1);
    }

    #[test]
    fn test_position_change() {
        let statuses = vec![
            create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0),
            create_test_status(MachineState::Run, 3.0, 4.0, 0.0, 1000.0),
        ];

        let distance = calculate_position_change(&statuses).unwrap();
        assert!((distance - 5.0).abs() < 0.01); // 3-4-5 triangle
    }

    #[test]
    fn test_state_changes() {
        let statuses = vec![
            create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0),
            create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0),
            create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1000.0),
            create_test_status(MachineState::Hold, 2.5, 2.5, 0.0, 0.0),
        ];

        let changes = find_state_changes(&statuses);
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].0, 1); // Index of first change
        assert_eq!(changes[0].1, MachineState::Idle);
        assert_eq!(changes[0].2, MachineState::Run);
    }


}
