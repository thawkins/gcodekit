//! Custom error types for gcodekit application.
//!
//! This module defines a centralized error type using `thiserror` for better
//! error handling and specificity throughout the application.

use thiserror::Error;

/// Main application error type.
///
/// This enum covers all possible errors that can occur in the gcodekit application,
/// providing specific variants for different subsystems.
#[derive(Debug, Error)]
pub enum GcodeKitError {
    /// Errors related to file I/O operations
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors related to JSON serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Errors related to serial port communication
    #[error("Serial port error: {0}")]
    SerialPort(#[from] serialport::Error),

    /// Errors related to parsing or processing GCODE
    #[error("GCODE error: {0}")]
    Gcode(String),

    /// Errors related to communication with CNC controllers
    #[error("Communication error: {0}")]
    Communication(String),

    /// Errors related to job processing and management
    #[error("Job error: {0}")]
    Job(String),

    /// Errors related to scripting (Rhai)
    #[error("Script error: {0}")]
    Script(String),

    /// Errors related to image processing
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    /// Errors related to SVG processing
    #[error("SVG processing error: {0}")]
    Svg(String),

    /// Errors related to DXF processing
    #[error("DXF processing error: {0}")]
    Dxf(String),

    /// Errors related to STL processing
    #[error("STL processing error: {0}")]
    Stl(String),

    /// Errors related to OBJ processing
    #[error("OBJ processing error: {0}")]
    Obj(String),

    /// Errors related to OBJ file loading
    #[error("OBJ loading error: {0}")]
    ObjLoad(String),

    /// Errors related to GLTF processing
    #[error("GLTF processing error: {0}")]
    Gltf(String),

    /// Generic application errors
    #[error("Application error: {0}")]
    App(String),

    /// Errors from external crates that don't have specific variants
    #[error("External error: {0}")]
    External(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, GcodeKitError>;
