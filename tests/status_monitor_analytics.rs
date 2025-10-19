//! Comprehensive integration tests for status monitor and analytics.
//! Tests the full monitoring pipeline including status tracking and analysis.

use gcodekit::communication::{
    grbl_status::MachineStatus, status_analytics::*, MachineState, StatusMonitor,
    StatusMonitorConfig,
};

fn create_test_status(
    state: MachineState,
    x: f32,
    y: f32,
    z: f32,
    feedrate: f32,
    spindle: f32,
) -> MachineStatus {
    let mut status = MachineStatus::new(state);
    status.machine_position.x = x;
    status.machine_position.y = y;
    status.machine_position.z = z;
    status.feed_speed.feed_rate = feedrate;
    status.feed_speed.spindle_speed = spindle;
    status
}

#[test]
fn test_status_monitor_creation() {
    let config = StatusMonitorConfig::default();
    let monitor = StatusMonitor::new(config);
    assert!(!monitor.is_running());
}

#[test]
fn test_status_monitor_config_defaults() {
    let config = StatusMonitorConfig::default();
    assert_eq!(config.query_interval_ms, 250);
    assert_eq!(config.history_buffer_size, 300);
    assert_eq!(config.max_parse_retries, 3);
    assert!(config.adaptive_timing);
    assert!(config.circular_buffer);
}

#[test]
fn test_status_monitor_config_custom() {
    let config = StatusMonitorConfig {
        query_interval_ms: 500,
        max_parse_retries: 5,
        adaptive_timing: false,
        history_buffer_size: 100,
        circular_buffer: false,
        track_errors: true,
        max_error_patterns: 5,
    };

    assert_eq!(config.query_interval_ms, 500);
    assert_eq!(config.history_buffer_size, 100);
}

#[test]
fn test_analytics_empty_history() {
    let statuses: Vec<MachineStatus> = vec![];
    let analytics = analyze_status_history(&statuses);

    assert_eq!(analytics.sample_count, 0);
    assert_eq!(analytics.avg_feedrate, 0.0);
    assert_eq!(analytics.peak_feedrate, 0.0);
    assert_eq!(analytics.state_transitions.len(), 0);
}

#[test]
fn test_analytics_single_status() {
    let statuses = vec![create_test_status(
        MachineState::Run,
        0.0,
        0.0,
        0.0,
        1500.0,
        12000.0,
    )];

    let analytics = analyze_status_history(&statuses);

    assert_eq!(analytics.sample_count, 1);
    assert_eq!(analytics.avg_feedrate, 1500.0);
    assert_eq!(analytics.peak_feedrate, 1500.0);
    assert_eq!(analytics.avg_spindle_speed, 12000.0);
    assert_eq!(analytics.state_transitions.len(), 0);
}

#[test]
fn test_analytics_multiple_states() {
    let statuses = vec![
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1200.0, 5000.0),
        create_test_status(MachineState::Hold, 2.5, 2.5, 0.0, 0.0, 5000.0),
        create_test_status(MachineState::Idle, 2.5, 2.5, 0.0, 0.0, 0.0),
    ];

    let analytics = analyze_status_history(&statuses);

    assert_eq!(analytics.sample_count, 6);
    assert_eq!(analytics.state_transitions.len(), 3); // Idle->Run, Run->Hold, Hold->Idle
    assert!(analytics.avg_feedrate > 0.0);
    assert!(analytics.peak_feedrate >= 1200.0);
}

#[test]
fn test_analytics_feedrate_statistics() {
    let statuses = vec![
        create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 100.0, 1000.0),
        create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 200.0, 5000.0),
        create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 300.0, 10000.0),
        create_test_status(MachineState::Run, 3.0, 3.0, 0.0, 150.0, 7000.0),
    ];

    let analytics = analyze_status_history(&statuses);

    assert_eq!(analytics.peak_feedrate, 300.0);
    assert_eq!(analytics.min_feedrate, 100.0);
    assert!((analytics.avg_feedrate - 187.5).abs() < 0.1); // (100+200+300+150)/4
}

#[test]
fn test_analytics_spindle_speed() {
    let statuses = vec![
        create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0, 12000.0),
        create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1000.0, 10000.0),
    ];

    let analytics = analyze_status_history(&statuses);

    assert_eq!(analytics.peak_spindle_speed, 12000.0);
    assert!((analytics.avg_spindle_speed - 9000.0).abs() < 1.0); // (5000+12000+10000)/3
}

#[test]
fn test_detect_alarms() {
    let statuses = vec![
        create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Alarm, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
    ];

    let alarms = detect_alarms(&statuses);
    assert_eq!(alarms.len(), 1);
    assert_eq!(alarms[0], 1);
}

#[test]
fn test_detect_multiple_alarms() {
    let statuses = vec![
        create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Alarm, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Door, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
    ];

    let alarms = detect_alarms(&statuses);
    assert_eq!(alarms.len(), 2);
    assert_eq!(alarms[0], 1); // Alarm at index 1
    assert_eq!(alarms[1], 2); // Door at index 2
}

#[test]
fn test_position_change() {
    let statuses = vec![
        create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Run, 3.0, 4.0, 0.0, 1000.0, 5000.0),
    ];

    let distance = calculate_position_change(&statuses).unwrap();
    assert!((distance - 5.0).abs() < 0.01); // 3-4-5 triangle
}

#[test]
fn test_position_change_3d() {
    let status1 = create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0);
    let status2 = create_test_status(MachineState::Run, 1.0, 1.0, 1.0, 1000.0, 5000.0);

    let statuses = vec![status1, status2];
    let distance = calculate_position_change(&statuses).unwrap();

    // sqrt(1^2 + 1^2 + 1^2) = sqrt(3)
    assert!((distance - 1.732).abs() < 0.01);
}

#[test]
fn test_state_changes() {
    let statuses = vec![
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Hold, 2.5, 2.5, 0.0, 0.0, 5000.0),
        create_test_status(MachineState::Idle, 2.5, 2.5, 0.0, 0.0, 0.0),
    ];

    let changes = find_state_changes(&statuses);

    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].0, 2); // First change at index 2
    assert_eq!(changes[0].1, MachineState::Idle);
    assert_eq!(changes[0].2, MachineState::Run);

    assert_eq!(changes[1].0, 4);
    assert_eq!(changes[1].1, MachineState::Run);
    assert_eq!(changes[1].2, MachineState::Hold);

    assert_eq!(changes[2].0, 5);
    assert_eq!(changes[2].1, MachineState::Hold);
    assert_eq!(changes[2].2, MachineState::Idle);
}

#[test]
fn test_state_durations() {
    let statuses = vec![
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Run, 3.0, 3.0, 0.0, 1000.0, 5000.0),
        create_test_status(MachineState::Hold, 3.5, 3.5, 0.0, 0.0, 5000.0),
    ];

    let durations = get_state_durations(&statuses);

    assert_eq!(durations.len(), 3); // Idle, Run, Hold
    assert_eq!(durations[0].state, MachineState::Idle);
    assert_eq!(durations[0].sample_count, 2);

    assert_eq!(durations[1].state, MachineState::Run);
    assert_eq!(durations[1].sample_count, 3);

    assert_eq!(durations[2].state, MachineState::Hold);
    assert_eq!(durations[2].sample_count, 1);
}

#[test]
fn test_job_progress() {
    let mut status = create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0);
    status.feedback.lines_completed = 50;
    status.feedback.lines_remaining = 50;

    let progress = calculate_progress(&[status]).unwrap();
    assert_eq!(progress, 50.0);
}

#[test]
fn test_job_progress_none_when_empty() {
    let progress = calculate_progress(&[]);
    assert!(progress.is_none());
}

#[test]
fn test_real_world_job_simulation() {
    // Simulate a real CNC job
    let statuses = vec![
        create_test_status(MachineState::Idle, 0.0, 0.0, 0.0, 0.0, 0.0),
        create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 500.0, 5000.0),
        create_test_status(MachineState::Run, 10.0, 5.0, -2.0, 1000.0, 10000.0),
        create_test_status(MachineState::Run, 20.0, 10.0, -2.0, 1200.0, 12000.0),
        create_test_status(MachineState::Run, 30.0, 15.0, -2.0, 1000.0, 10000.0),
        create_test_status(MachineState::Hold, 30.0, 15.0, -2.0, 0.0, 10000.0),
        create_test_status(MachineState::Hold, 30.0, 15.0, -2.0, 0.0, 10000.0),
        create_test_status(MachineState::Run, 30.0, 15.0, -2.0, 800.0, 10000.0),
        create_test_status(MachineState::Run, 40.0, 20.0, -2.0, 1000.0, 12000.0),
        create_test_status(MachineState::Idle, 40.0, 20.0, -2.0, 0.0, 0.0),
    ];

    let analytics = analyze_status_history(&statuses);

    // Verify feedrate tracking
    assert!(analytics.peak_feedrate >= 1200.0);
    assert!(analytics.avg_feedrate > 0.0);

    // Verify state tracking
    assert!(analytics.state_transitions.len() >= 3);

    // Verify position change
    let distance = calculate_position_change(&statuses).unwrap();
    assert!(distance > 40.0); // Traveled >40mm

    // Verify state changes detected
    let changes = find_state_changes(&statuses);
    assert!(changes.len() >= 2); // At least Idle->Run and Hold transitions
}

#[test]
fn test_buffer_statistics() {
    let mut status1 = create_test_status(MachineState::Run, 0.0, 0.0, 0.0, 1000.0, 5000.0);
    status1.buffer_state.planner_buffer = 64; // 50%

    let mut status2 = create_test_status(MachineState::Run, 1.0, 1.0, 0.0, 1000.0, 5000.0);
    status2.buffer_state.planner_buffer = 128; // 100%

    let mut status3 = create_test_status(MachineState::Run, 2.0, 2.0, 0.0, 1000.0, 5000.0);
    status3.buffer_state.planner_buffer = 32; // 25%

    let statuses = vec![status1, status2, status3];
    let analytics = analyze_status_history(&statuses);

    assert_eq!(analytics.peak_buffer_fill, 128);
}
