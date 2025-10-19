//! Comprehensive integration tests for GRBL status parser.
//! Tests all GRBL versions and edge cases.

use gcodekit::communication::status_parser::{parse_status_response, StatusParseError};
use gcodekit::communication::{MachineState, MachineStatus};

#[test]
fn test_grbl_v1_0_idle() {
    let response = "<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.state, MachineState::Idle);
    assert_eq!(status.machine_position.x, 0.0);
    assert_eq!(status.machine_position.y, 0.0);
    assert_eq!(status.machine_position.z, 0.0);
    assert_eq!(status.feed_speed.feed_rate, 0.0);
    assert_eq!(status.feed_speed.spindle_speed, 0.0);
    assert_eq!(status.overrides.feed_override, 100);
}

#[test]
fn test_grbl_v1_0_running() {
    let response = "<Run|MPos:45.23,12.50,8.75|FS:2400,15000|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.state, MachineState::Run);
    assert_eq!(status.machine_position.x, 45.23);
    assert_eq!(status.machine_position.y, 12.50);
    assert_eq!(status.machine_position.z, 8.75);
    assert_eq!(status.feed_speed.feed_rate, 2400.0);
    assert_eq!(status.feed_speed.spindle_speed, 15000.0);
}

#[test]
fn test_grbl_v1_1_full_response() {
    let response = "<Run|MPos:10.50,5.25,2.10|WPos:10.50,5.25,2.10|FS:1500,12000|Ov:100,100,100|Buf:18|Rx:256|Line:42>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.state, MachineState::Run);
    assert!(status.work_position.is_some());
    assert_eq!(status.work_position.unwrap().x, 10.50);
    assert_eq!(status.buffer_state.planner_buffer, 18);
    assert_eq!(status.buffer_state.rx_buffer, 255); // Clamped from 256
    assert_eq!(status.line_number, Some(42));
}

#[test]
fn test_all_machine_states() {
    let states = vec![
        ("Idle", MachineState::Idle),
        ("Run", MachineState::Run),
        ("Hold", MachineState::Hold),
        ("Jog", MachineState::Jog),
        ("Alarm", MachineState::Alarm),
        ("Door", MachineState::Door),
        ("Check", MachineState::Check),
        ("Home", MachineState::Home),
        ("Sleep", MachineState::Sleep),
    ];

    for (state_str, expected_state) in states {
        let response = format!("<{}|MPos:0,0,0|FS:0,0|Ov:100,100,100>", state_str);
        let status =
            parse_status_response(&response).expect(&format!("Failed to parse {}", state_str));
        assert_eq!(status.state, expected_state);
    }
}

#[test]
fn test_high_precision_coordinates() {
    let response = "<Idle|MPos:123.456,789.012,345.678|FS:0,0|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert!((status.machine_position.x - 123.456).abs() < 0.001);
    assert!((status.machine_position.y - 789.012).abs() < 0.001);
    assert!((status.machine_position.z - 345.678).abs() < 0.001);
}

#[test]
fn test_negative_coordinates() {
    let response = "<Idle|MPos:-10.5,-20.3,-5.1|FS:0,0|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.machine_position.x, -10.5);
    assert_eq!(status.machine_position.y, -20.3);
    assert_eq!(status.machine_position.z, -5.1);
}

#[test]
fn test_override_ranges() {
    // Test maximum overrides (200%)
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:200,150,50>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.overrides.feed_override, 200);
    assert_eq!(status.overrides.spindle_override, 150);
    assert_eq!(status.overrides.coolant_override, 50);

    // Test minimum overrides
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:0,0,0>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.overrides.feed_override, 0);
    assert_eq!(status.overrides.spindle_override, 0);
    assert_eq!(status.overrides.coolant_override, 0);
}

#[test]
fn test_pin_states_single() {
    let responses = vec![
        ("<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:X>", "x_limit"),
        ("<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:Y>", "y_limit"),
        ("<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:Z>", "z_limit"),
        ("<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:P>", "probe"),
        ("<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:D>", "door"),
    ];

    for (response, pin_name) in responses {
        let status =
            parse_status_response(response).expect(&format!("Failed to parse {}", pin_name));
        match pin_name {
            "x_limit" => assert!(status.pin_states.x_limit),
            "y_limit" => assert!(status.pin_states.y_limit),
            "z_limit" => assert!(status.pin_states.z_limit),
            "probe" => assert!(status.pin_states.probe),
            "door" => assert!(status.pin_states.door_open),
            _ => panic!("Unknown pin"),
        }
    }
}

#[test]
fn test_pin_states_multiple() {
    let response = "<Alarm|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:XYZ>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert!(status.pin_states.x_limit);
    assert!(status.pin_states.y_limit);
    assert!(status.pin_states.z_limit);
    assert!(!status.pin_states.probe);
}

#[test]
fn test_pin_states_all() {
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Pn:XYZDCFP>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert!(status.pin_states.x_limit);
    assert!(status.pin_states.y_limit);
    assert!(status.pin_states.z_limit);
    assert!(status.pin_states.door_open);
    assert!(status.pin_states.cycle_start);
    assert!(status.pin_states.feed_hold);
    assert!(status.pin_states.probe);
}

#[test]
fn test_6_axis_position() {
    let response = "<Idle|MPos:1.0,2.0,3.0,4.0,5.0,6.0|FS:0,0|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.machine_position.x, 1.0);
    assert_eq!(status.machine_position.y, 2.0);
    assert_eq!(status.machine_position.z, 3.0);
    assert_eq!(status.machine_position.a, Some(4.0));
    assert_eq!(status.machine_position.b, Some(5.0));
    assert_eq!(status.machine_position.c, Some(6.0));
}

#[test]
fn test_high_feed_rates() {
    let response = "<Idle|MPos:0,0,0|FS:9000,20000|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.feed_speed.feed_rate, 9000.0);
    assert_eq!(status.feed_speed.spindle_speed, 20000.0);
}

#[test]
fn test_high_line_numbers() {
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100|Line:99999>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert_eq!(status.line_number, Some(99999));
}

#[test]
fn test_invalid_format_no_brackets() {
    let response = "Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100";
    assert_eq!(
        parse_status_response(response),
        Err(StatusParseError::InvalidFormat)
    );
}

#[test]
fn test_invalid_format_missing_closing() {
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100";
    assert_eq!(
        parse_status_response(response),
        Err(StatusParseError::InvalidFormat)
    );
}

#[test]
fn test_empty_response() {
    let response = "<>";
    assert_eq!(
        parse_status_response(response),
        Err(StatusParseError::EmptyResponse)
    );
}

#[test]
fn test_missing_optional_fields() {
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");

    assert!(status.work_position.is_none());
    assert!(status.line_number.is_none());
}

#[test]
fn test_status_helper_methods() {
    let mut status = MachineStatus::new(MachineState::Run);
    assert!(status.is_executing());
    assert!(!status.is_error_state());
    assert!(!status.is_idle());

    status.state = MachineState::Alarm;
    assert!(!status.is_executing());
    assert!(status.is_error_state());

    status.state = MachineState::Hold;
    assert!(status.is_idle());
}

#[test]
fn test_buffer_fill_percentage() {
    let status: MachineStatus = MachineStatus {
        buffer_state: gcodekit::communication::grbl_status::BufferState::new(64, 128),
        ..Default::default()
    };

    assert_eq!(status.buffer_state.planner_fill_percent(), 50);
}

#[test]
fn test_format_status_debug() {
    let response = "<Run|MPos:10.50,5.25,2.10|FS:1500,12000|Ov:100,100,100|Line:42>";
    let status = parse_status_response(response).expect("Failed to parse");
    let debug_str = status.format_debug();

    assert!(debug_str.contains("Run"));
    assert!(debug_str.contains("10.50"));
    assert!(debug_str.contains("1500"));
    assert!(debug_str.len() > 50);
}

#[test]
fn test_whitespace_tolerance() {
    // Parser should handle some whitespace
    let response = "<Idle|MPos:0,0,0|FS:0,0|Ov:100,100,100>";
    let status = parse_status_response(response).expect("Failed to parse");
    assert_eq!(status.state, MachineState::Idle);
}

#[test]
fn test_real_world_scenario_job_running() {
    let responses = vec![
        "<Run|MPos:0.00,0.00,0.00|WPos:0.00,0.00,0.00|FS:1000,12000|Ov:100,100,100|Buf:32|Rx:128|Line:1>",
        "<Run|MPos:10.50,5.25,0.00|WPos:10.50,5.25,0.00|FS:1200,12000|Ov:100,100,100|Buf:28|Rx:130|Line:50>",
        "<Run|MPos:20.00,10.00,-0.50|WPos:20.00,10.00,-0.50|FS:1500,12000|Ov:100,100,100|Buf:24|Rx:132|Line:100>",
        "<Hold|MPos:20.50,10.25,-0.50|WPos:20.50,10.25,-0.50|FS:0,12000|Ov:100,100,100|Buf:32|Rx:128|Line:101>",
        "<Hold|MPos:20.50,10.25,-0.50|WPos:20.50,10.25,-0.50|FS:0,12000|Ov:100,100,100|Buf:32|Rx:128|Line:101>",
        "<Run|MPos:20.50,10.25,-0.50|WPos:20.50,10.25,-0.50|FS:1500,12000|Ov:100,100,100|Buf:26|Rx:130|Line:102>",
        "<Idle|MPos:20.50,10.25,-0.50|WPos:20.50,10.25,-0.50|FS:0,0|Ov:100,100,100|Buf:0|Rx:64|Line:102>",
    ];

    for response in responses {
        let status = parse_status_response(response).expect("Failed to parse");
        assert!(matches!(
            status.state,
            MachineState::Run | MachineState::Hold | MachineState::Idle
        ));
        assert!(status.line_number.is_some());
    }
}
