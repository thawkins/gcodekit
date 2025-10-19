//! Web pendant interface tests

use gcodekit::web_pendant::{WebPendant, WebPendantMessage};

#[tokio::test]
async fn test_web_pendant_new() {
    let (pendant, _receiver) = WebPendant::new();
    // Verify pendant was created successfully
    drop(pendant);
}

#[tokio::test]
async fn test_web_pendant_default() {
    let _pendant = WebPendant::default();
    // Verify default pendant was created successfully
}

#[test]
fn test_web_pendant_message_serialize() {
    let msg = WebPendantMessage::StatusUpdate {
        connected: true,
        position: Some((10.0, 20.0, 30.0)),
        status: "Running".to_string(),
    };

    let json = serde_json::to_string(&msg);
    assert!(json.is_ok(), "Should serialize status update");

    let serialized = json.unwrap();
    assert!(serialized.contains("StatusUpdate") || serialized.contains("connected"));
}

#[test]
fn test_web_pendant_message_command() {
    let msg = WebPendantMessage::Command {
        command_type: "JOG".to_string(),
        data: serde_json::json!({"axis": "X", "distance": 10.0}),
    };

    let json = serde_json::to_string(&msg);
    assert!(json.is_ok(), "Should serialize command message");
}

#[test]
fn test_web_pendant_message_response() {
    let msg = WebPendantMessage::Response {
        success: true,
        message: "Command executed".to_string(),
        data: Some(serde_json::json!({"result": "ok"})),
    };

    let json = serde_json::to_string(&msg);
    assert!(json.is_ok(), "Should serialize response message");
}

#[test]
fn test_web_pendant_message_status_update_no_position() {
    let msg = WebPendantMessage::StatusUpdate {
        connected: false,
        position: None,
        status: "Disconnected".to_string(),
    };

    let json = serde_json::to_string(&msg);
    assert!(json.is_ok(), "Should handle status without position");
}

#[test]
fn test_web_pendant_clone_message() {
    let msg = WebPendantMessage::StatusUpdate {
        connected: true,
        position: Some((1.0, 2.0, 3.0)),
        status: "Idle".to_string(),
    };

    let cloned = msg.clone();
    let json1 = serde_json::to_string(&msg).unwrap();
    let json2 = serde_json::to_string(&cloned).unwrap();
    assert_eq!(json1, json2, "Cloned message should serialize identically");
}

#[test]
fn test_web_pendant_message_debug() {
    let msg = WebPendantMessage::StatusUpdate {
        connected: true,
        position: Some((5.0, 10.0, 15.0)),
        status: "Running".to_string(),
    };

    let debug_str = format!("{:?}", msg);
    assert!(debug_str.contains("StatusUpdate"));
}

#[tokio::test]
async fn test_web_pendant_async_new() {
    let (_pendant, _receiver) = WebPendant::new();
    // Test that web pendant can be created in async context
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
}
