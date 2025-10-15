//! GRBL G/M code vocabulary for versions 1.0, 1.1 and 1.2
//! This file exposes simple code lists and metadata used by the editor for
//! syntax highlighting, completion and validation.

/// Code metadata
pub struct CodeInfo {
    pub code: &'static str,
    pub description: &'static str,
    pub supported_in: &'static [&'static str],
}

/// Common G-codes in GRBL 1.0 / 1.1 / 1.2
pub static G_CODES: &[CodeInfo] = &[
    CodeInfo { code: "G0", description: "Rapid positioning", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G1", description: "Linear interpolation (feed)", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G2", description: "Clockwise arc", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G3", description: "Counter-clockwise arc", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G4", description: "Dwell", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G10", description: "Coordinate data/probe (controller dependent)", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "G17", description: "XY plane selection", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G18", description: "XZ plane selection", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G19", description: "YZ plane selection", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G20", description: "Units: inches", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G21", description: "Units: millimeters", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G28", description: "Return to home position", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G30", description: "Return to secondary home", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G38.2", description: "Probe toward workpiece (stop on contact)", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "G40", description: "Tool radius compensation off (limited support)", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "G53", description: "Machine coordinate system (non-modal)", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "G54", description: "Work coordinate system 1", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G55", description: "Work coordinate system 2", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G56", description: "Work coordinate system 3", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G57", description: "Work coordinate system 4", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G58", description: "Work coordinate system 5", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G59", description: "Work coordinate system 6", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G90", description: "Absolute positioning", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G91", description: "Incremental positioning", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "G92", description: "Set position", supported_in: &["1.0","1.1","1.2"] },
];

/// Common M-codes in GRBL 1.0 / 1.1 / 1.2
pub static M_CODES: &[CodeInfo] = &[
    CodeInfo { code: "M0", description: "Program stop", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "M1", description: "Optional stop", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "M2", description: "Program end", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "M3", description: "Spindle CW", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "M4", description: "Spindle CCW", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "M5", description: "Spindle stop", supported_in: &["1.0","1.1","1.2"] },
    CodeInfo { code: "M7", description: "Mist coolant on (if supported)", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "M8", description: "Flood coolant on", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "M9", description: "Coolant off", supported_in: &["1.1","1.2"] },
    CodeInfo { code: "M30", description: "Program end and reset", supported_in: &["1.0","1.1","1.2"] },
];

/// Utility: check support
pub fn code_supported(code: &str, version: &str) -> bool {
    let all: Vec<&CodeInfo> = G_CODES.iter().chain(M_CODES.iter()).collect();
    all.iter().any(|c| c.code.eq_ignore_ascii_case(code) && c.supported_in.iter().any(|v| *v == version))
}
