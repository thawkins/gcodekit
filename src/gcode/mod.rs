//! G-code parsing and processing utilities.
//!
//! This module provides functions for parsing G-code content, extracting
//! path segments for visualization, and performing G-code analysis.

use crate::types::{MachinePosition, MoveType, PathSegment};

/// Parses G-code content and extracts path segments for visualization and analysis.
/// Handles absolute/incremental positioning modes and identifies move commands.
///
/// # Arguments
/// * `gcode_content` - The G-code content as a string
///
/// # Returns
/// A vector of PathSegment objects representing the parsed moves
pub fn parse_gcode(gcode_content: &str) -> Vec<PathSegment> {
    let mut parsed_paths = Vec::new();
    let mut current_pos = MachinePosition::new(0.0, 0.0, 0.0);
    let mut current_move_type = MoveType::Rapid;
    let mut absolute_mode = true; // G90 = absolute, G91 = incremental

    for (line_idx, line) in gcode_content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut new_pos = current_pos.clone();
        let mut move_type = current_move_type.clone();
        let mut has_move_command = false;

        for part in parts {
            if let Some(stripped) = part.strip_prefix('G') {
                if let Ok(code) = stripped.parse::<u32>() {
                    match code {
                        0 => {
                            move_type = MoveType::Rapid;
                            has_move_command = true;
                        }
                        1 => {
                            move_type = MoveType::Feed;
                            has_move_command = true;
                        }
                        2 | 3 => {
                            move_type = MoveType::Arc;
                            has_move_command = true;
                        }
                        90 => absolute_mode = true,
                        91 => absolute_mode = false,
                        _ => {} // Other G-codes like G17-G19 (planes), G20/G21, G54-G59, etc.
                    }
                }
            } else if part.len() > 1 {
                let axis = match part.chars().next() { Some(c) => c, None => continue };
                if let Ok(value) = part[1..].parse::<f32>() {
                    if absolute_mode {
                        new_pos.set_axis(axis, value);
                    } else {
                        // Incremental mode: add to current position
                        let current_value = current_pos.get_axis(axis).unwrap_or(0.0);
                        new_pos.set_axis(axis, current_value + value);
                    }
                }
            }
        }

        // Only create path segments for move commands
        if has_move_command {
            // Check if position changed
            let position_changed = new_pos.x != current_pos.x
                || new_pos.y != current_pos.y
                || new_pos.z != current_pos.z
                || new_pos.a != current_pos.a
                || new_pos.b != current_pos.b
                || new_pos.c != current_pos.c
                || new_pos.d != current_pos.d;

            if position_changed {
                parsed_paths.push(PathSegment {
                    start: current_pos,
                    end: new_pos.clone(),
                    move_type: move_type.clone(),
                    line_number: line_idx,
                });
                current_pos = new_pos;
            }
        }
        current_move_type = move_type;
    }

    parsed_paths
}
