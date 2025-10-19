//! G-code parsing and processing utilities.
//!
//! This module provides functions for parsing G-code content, extracting
//! path segments for visualization, performing G-code analysis, and
//! optimizing G-code for production with advanced techniques.

use crate::types::{MachinePosition, MoveType, PathSegment};
use std::f32::consts::PI;

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
                let axis = match part.chars().next() {
                    Some(c) => c,
                    None => continue,
                };
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
                || new_pos.z != current_pos.z;

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

/// Optimizes G-code by truncating decimal precision to specified decimal places.
/// Reduces file size while maintaining precision for most machining operations.
///
/// # Arguments
/// * `gcode_content` - The G-code content as a string
/// * `decimal_places` - Number of decimal places to retain (typically 3-4)
///
/// # Returns
/// Optimized G-code string with truncated decimal values
pub fn truncate_decimal_precision(gcode_content: &str, decimal_places: u32) -> String {
    let multiplier = 10_f32.powi(decimal_places as i32);
    let mut result = String::new();

    for line in gcode_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let mut optimized_line = String::new();
        let mut chars = trimmed.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_alphabetic() {
                optimized_line.push(ch);
                // Collect the numeric part
                let mut num_str = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_numeric() || next_ch == '.' || next_ch == '-' {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !num_str.is_empty() {
                    if let Ok(val) = num_str.parse::<f32>() {
                        let truncated = (val * multiplier).trunc() / multiplier;
                        optimized_line.push_str(&format!("{}", truncated));
                    } else {
                        optimized_line.push_str(&num_str);
                    }
                }
            } else {
                optimized_line.push(ch);
            }
        }

        result.push_str(&optimized_line);
        result.push('\n');
    }

    result
}

/// Converts arc commands (G2/G3) to line segments with specified tolerance.
/// Useful for controllers with limited arc support or for path analysis.
///
/// # Arguments
/// * `gcode_content` - The G-code content as a string
/// * `tolerance` - Maximum deviation in mm (typically 0.01-0.1)
///
/// # Returns
/// G-code with arcs converted to G1 line segments
pub fn convert_arcs_to_lines(gcode_content: &str, tolerance: f32) -> String {
    let mut result = Vec::new();
    let mut current_pos = MachinePosition::new(0.0, 0.0, 0.0);
    let mut absolute_mode = true;

    for line in gcode_content.lines() {
        let trimmed = line.trim();

        // Handle mode commands
        if trimmed.contains("G90") {
            absolute_mode = true;
            result.push(line.to_string());
            continue;
        }
        if trimmed.contains("G91") {
            absolute_mode = false;
            result.push(line.to_string());
            continue;
        }

        // Check if this is an arc command
        if trimmed.starts_with(';') || trimmed.is_empty() {
            result.push(line.to_string());
            continue;
        }

        let is_arc = trimmed.contains('G') && (trimmed.contains("G2") || trimmed.contains("G3"));
        if !is_arc {
            result.push(line.to_string());
            // Update current position for non-arc moves
            update_position(trimmed, &mut current_pos, absolute_mode);
            continue;
        }

        // Parse arc parameters
        let (end_pos, center_pos, is_cw, feed_rate) =
            parse_arc_command(trimmed, current_pos.clone(), absolute_mode);

        if let (Some(end), Some(center)) = (end_pos, center_pos) {
            // Convert arc to line segments
            let segments = approximate_arc(current_pos.clone(), end.clone(), center, is_cw, tolerance);
            for (i, seg_end) in segments.iter().enumerate() {
                if i == 0 {
                    continue; // Skip first point (current position)
                }
                let feed_str = feed_rate.map(|f| format!(" F{}", f)).unwrap_or_default();
                result.push(format!(
                    "G1 X{:.4} Y{:.4} Z{:.4}{}",
                    seg_end.x, seg_end.y, seg_end.z, feed_str
                ));
            }
            current_pos = end;
        }
    }

    result.join("\n")
}

/// Removes redundant whitespace and optimizes G-code formatting.
/// Removes extra spaces, consolidates commands on single lines where appropriate.
///
/// # Arguments
/// * `gcode_content` - The G-code content as a string
///
/// # Returns
/// Compact G-code with minimal whitespace
pub fn remove_redundant_whitespace(gcode_content: &str) -> String {
    let mut result = Vec::new();

    for line in gcode_content.lines() {
        let trimmed = line.trim();

        // Preserve empty lines sparingly and comments
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with(';') {
            result.push(trimmed.to_string());
            continue;
        }

        // Collapse multiple spaces into single space, remove leading/trailing
        let optimized = trimmed
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        result.push(optimized);
    }

    result.join("\n")
}

/// Parses an arc command and extracts end position, center, direction, and feed rate
fn parse_arc_command(
    line: &str,
    current_pos: MachinePosition,
    absolute_mode: bool,
) -> (Option<MachinePosition>, Option<MachinePosition>, bool, Option<f32>) {
    let mut end_pos = current_pos.clone();
    let mut center_pos = MachinePosition::new(0.0, 0.0, 0.0);
    let mut feed_rate = None;
    let is_cw = line.contains("G2") && !line.contains("G20") && !line.contains("G28");

    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        if let Some(val_str) = part.strip_prefix('X') {
            if let Ok(val) = val_str.parse::<f32>() {
                if absolute_mode {
                    end_pos.x = val;
                } else {
                    end_pos.x = current_pos.x + val;
                }
            }
        } else if let Some(val_str) = part.strip_prefix('Y') {
            if let Ok(val) = val_str.parse::<f32>() {
                if absolute_mode {
                    end_pos.y = val;
                } else {
                    end_pos.y = current_pos.y + val;
                }
            }
        } else if let Some(val_str) = part.strip_prefix('Z') {
            if let Ok(val) = val_str.parse::<f32>() {
                if absolute_mode {
                    end_pos.z = val;
                } else {
                    end_pos.z = current_pos.z + val;
                }
            }
        } else if let Some(val_str) = part.strip_prefix('I') {
            if let Ok(val) = val_str.parse::<f32>() {
                center_pos.x = current_pos.x + val;
            }
        } else if let Some(val_str) = part.strip_prefix('J') {
            if let Ok(val) = val_str.parse::<f32>() {
                center_pos.y = current_pos.y + val;
            }
        } else if let Some(val_str) = part.strip_prefix('F') {
            if let Ok(val) = val_str.parse::<f32>() {
                feed_rate = Some(val);
            }
        }
    }

    (Some(end_pos), Some(center_pos), is_cw, feed_rate)
}

/// Updates current position based on a G-code line
fn update_position(line: &str, pos: &mut MachinePosition, absolute_mode: bool) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        if let Some(val_str) = part.strip_prefix('X') {
            if let Ok(val) = val_str.parse::<f32>() {
                if absolute_mode {
                    pos.x = val;
                } else {
                    pos.x += val;
                }
            }
        } else if let Some(val_str) = part.strip_prefix('Y') {
            if let Ok(val) = val_str.parse::<f32>() {
                if absolute_mode {
                    pos.y = val;
                } else {
                    pos.y += val;
                }
            }
        } else if let Some(val_str) = part.strip_prefix('Z') {
            if let Ok(val) = val_str.parse::<f32>() {
                if absolute_mode {
                    pos.z = val;
                } else {
                    pos.z += val;
                }
            }
        }
    }
}

/// Approximates an arc with line segments using the chord error method
fn approximate_arc(
    start: MachinePosition,
    end: MachinePosition,
    center: MachinePosition,
    is_cw: bool,
    tolerance: f32,
) -> Vec<MachinePosition> {
    let mut segments = vec![start.clone()];

    let radius = ((start.x - center.x).powi(2) + (start.y - center.y).powi(2)).sqrt();
    if radius < 0.001 {
        segments.push(end);
        return segments;
    }

    let start_angle = (start.y - center.y).atan2(start.x - center.x);
    let end_angle = (end.y - center.y).atan2(end.x - center.x);

    let mut sweep = end_angle - start_angle;
    if is_cw && sweep > 0.0 {
        sweep -= 2.0 * PI;
    } else if !is_cw && sweep < 0.0 {
        sweep += 2.0 * PI;
    }

    let steps = ((sweep.abs() * radius / (2.0 * tolerance.max(0.001))).ceil()) as u32;
    let steps = steps.clamp(1, 1000);

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + sweep * t;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        let z = start.z + (end.z - start.z) * t;
        segments.push(MachinePosition::new(x, y, z));
    }

    segments
}
