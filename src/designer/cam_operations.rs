use crate::cam::types::*;
use crate::types::{MachinePosition, MoveType, PathSegment};
use std::f32::consts::PI;

/// Generate toolpath for a CAM operation
pub fn generate_cam_toolpath(operation: &CAMOperation, params: &CAMParameters) -> Vec<PathSegment> {
    match operation {
        CAMOperation::None => Vec::new(),
        CAMOperation::Contour2D {
            depth,
            stepover,
            direction,
        } => generate_2d_contour(*depth, *stepover, direction, params),
        CAMOperation::SideProfile {
            depth,
            stepover,
            direction,
            wall_angle,
        } => generate_side_profile(*depth, *stepover, direction, *wall_angle, params),
        CAMOperation::Waterline {
            min_z,
            max_z,
            stepdown,
            stepover,
        } => generate_waterline(*min_z, *max_z, *stepdown, *stepover, params),
        CAMOperation::Scanline {
            min_z,
            max_z,
            stepdown,
            stepover,
            angle,
        } => generate_scanline(*min_z, *max_z, *stepdown, *stepover, *angle, params),
        CAMOperation::Turning {
            diameter,
            length,
            finish_pass,
            roughing_feed,
            finishing_feed,
        } => generate_turning(
            *diameter,
            *length,
            *finish_pass,
            *roughing_feed,
            *finishing_feed,
            params,
        ),
        CAMOperation::Facing {
            diameter,
            width,
            depth,
            stepover,
        } => generate_facing(*diameter, *width, *depth, *stepover, params),
        CAMOperation::Threading {
            major_diameter,
            minor_diameter,
            pitch,
            length,
        } => generate_threading(*major_diameter, *minor_diameter, *pitch, *length, params),
    }
}

/// Generate 2D contour toolpath
fn generate_2d_contour(
    _depth: f32,
    _stepover: f32,
    direction: &ContourDirection,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_z = params.stock_surface;

    // Move to safe Z
    segments.push(PathSegment {
        start: MachinePosition::new(0.0, 0.0, current_z),
        end: MachinePosition::new(0.0, 0.0, params.safe_z),
        move_type: MoveType::Rapid,
        line_number: 0,
    });

    // Generate depth passes
    while current_z > params.final_depth {
        current_z = (current_z - params.stepdown).max(params.final_depth);

        // Rapid to start position
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, params.safe_z),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Plunge to cutting depth
        segments.push(PathSegment {
            start: MachinePosition::new(
                -params.tool_diameter,
                -params.tool_diameter,
                params.safe_z,
            ),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, current_z),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Generate rectangular contour with tabs and lead moves
        let width = 100.0; // Example dimensions
        let height = 50.0;

        let base_points = match direction {
            ContourDirection::Clockwise | ContourDirection::Climb => {
                vec![
                    (-params.tool_diameter, -params.tool_diameter),
                    (width + params.tool_diameter, -params.tool_diameter),
                    (width + params.tool_diameter, height + params.tool_diameter),
                    (-params.tool_diameter, height + params.tool_diameter),
                    (-params.tool_diameter, -params.tool_diameter),
                ]
            }
            ContourDirection::CounterClockwise | ContourDirection::Conventional => {
                vec![
                    (-params.tool_diameter, -params.tool_diameter),
                    (-params.tool_diameter, height + params.tool_diameter),
                    (width + params.tool_diameter, height + params.tool_diameter),
                    (width + params.tool_diameter, -params.tool_diameter),
                    (-params.tool_diameter, -params.tool_diameter),
                ]
            }
        };

        // Add tabs if enabled and this is the final depth pass
        let points = if params.tabs_enabled && (current_z - params.final_depth).abs() < 0.01 {
            add_tabs_to_contour(&base_points, params.tab_width, params.tab_height, current_z)
        } else {
            base_points
        };

        // If there are no points, skip this pass
        if points.len() < 2 {
            continue;
        }

        // Generate contour with lead moves
        for i in 0..points.len() - 1 {
            let start_point = points[i];
            let end_point = points[i + 1];

            // Add lead-in move if enabled and this is the first segment
            if params.lead_in_enabled && i == 0 {
                let lead_in_point =
                    calculate_lead_point(start_point, end_point, params.lead_in_length, true);
                segments.push(PathSegment {
                    start: MachinePosition::new(start_point.0, start_point.1, current_z),
                    end: MachinePosition::new(lead_in_point.0, lead_in_point.1, current_z),
                    move_type: MoveType::Feed,
                    line_number: segments.len(),
                });
            }

            // Main cutting move
            segments.push(PathSegment {
                start: MachinePosition::new(start_point.0, start_point.1, current_z),
                end: MachinePosition::new(end_point.0, end_point.1, current_z),
                move_type: MoveType::Feed,
                line_number: segments.len(),
            });

            // Add lead-out move if enabled and this is the last segment
            if params.lead_out_enabled && i == points.len() - 2 {
                let lead_out_point =
                    calculate_lead_point(end_point, start_point, params.lead_out_length, false);
                segments.push(PathSegment {
                    start: MachinePosition::new(end_point.0, end_point.1, current_z),
                    end: MachinePosition::new(lead_out_point.0, lead_out_point.1, current_z),
                    move_type: MoveType::Feed,
                    line_number: segments.len(),
                });
            }
        }

        // Return to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(
                points.last().copied().unwrap_or((0.0,0.0)).0,
                points.last().copied().unwrap_or((0.0,0.0)).1,
                current_z,
            ),
            end: MachinePosition::new(
                points.last().copied().unwrap_or((0.0,0.0)).0,
                points.last().copied().unwrap_or((0.0,0.0)).1,
                params.safe_z,
            ),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });
    }

    segments
}

/// Generate side profile machining toolpath
fn generate_side_profile(
    _depth: f32,
    _stepover: f32,
    direction: &ContourDirection,
    wall_angle: f32,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_z = params.stock_surface;

    // Move to safe Z
    segments.push(PathSegment {
        start: MachinePosition::new(0.0, 0.0, current_z),
        end: MachinePosition::new(0.0, 0.0, params.safe_z),
        move_type: MoveType::Rapid,
        line_number: 0,
    });

    // Generate depth passes
    while current_z > params.final_depth {
        current_z = (current_z - params.stepdown).max(params.final_depth);

        // Rapid to start position
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, params.safe_z),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Plunge to cutting depth
        segments.push(PathSegment {
            start: MachinePosition::new(
                -params.tool_diameter,
                -params.tool_diameter,
                params.safe_z,
            ),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, current_z),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Generate side profile with draft angle
        let width = 100.0;
        let height = 50.0;
        let angle_rad = wall_angle * PI / 180.0;
        let draft_offset = current_z * angle_rad.tan(); // Horizontal offset due to draft

        let points = match direction {
            ContourDirection::Clockwise | ContourDirection::Climb => {
                vec![
                    (
                        -params.tool_diameter - draft_offset,
                        -params.tool_diameter - draft_offset,
                    ),
                    (
                        width + params.tool_diameter + draft_offset,
                        -params.tool_diameter - draft_offset,
                    ),
                    (
                        width + params.tool_diameter + draft_offset,
                        height + params.tool_diameter + draft_offset,
                    ),
                    (
                        -params.tool_diameter - draft_offset,
                        height + params.tool_diameter + draft_offset,
                    ),
                    (
                        -params.tool_diameter - draft_offset,
                        -params.tool_diameter - draft_offset,
                    ),
                ]
            }
            ContourDirection::CounterClockwise | ContourDirection::Conventional => {
                vec![
                    (
                        -params.tool_diameter - draft_offset,
                        -params.tool_diameter - draft_offset,
                    ),
                    (
                        -params.tool_diameter - draft_offset,
                        height + params.tool_diameter + draft_offset,
                    ),
                    (
                        width + params.tool_diameter + draft_offset,
                        height + params.tool_diameter + draft_offset,
                    ),
                    (
                        width + params.tool_diameter + draft_offset,
                        -params.tool_diameter - draft_offset,
                    ),
                    (
                        -params.tool_diameter - draft_offset,
                        -params.tool_diameter - draft_offset,
                    ),
                ]
            }
        };

        for i in 0..points.len() - 1 {
            segments.push(PathSegment {
                start: MachinePosition::new(points[i].0, points[i].1, current_z),
                end: MachinePosition::new(points[i + 1].0, points[i + 1].1, current_z),
                move_type: MoveType::Feed,
                line_number: segments.len(),
            });
        }

        // Return to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(
                -params.tool_diameter - draft_offset,
                -params.tool_diameter - draft_offset,
                current_z,
            ),
            end: MachinePosition::new(
                -params.tool_diameter - draft_offset,
                -params.tool_diameter - draft_offset,
                params.safe_z,
            ),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });
    }

    segments
}

/// Generate waterline machining toolpath
fn generate_waterline(
    min_z: f32,
    max_z: f32,
    stepdown: f32,
    stepover: f32,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_z = max_z;

    // Waterline machining generates horizontal slices at constant Z heights
    while current_z >= min_z {
        // Move to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, current_z),
            end: MachinePosition::new(0.0, 0.0, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Rapid to start of waterline
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, params.safe_z),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Plunge to waterline depth
        segments.push(PathSegment {
            start: MachinePosition::new(
                -params.tool_diameter,
                -params.tool_diameter,
                params.safe_z,
            ),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, current_z),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Generate waterline pattern (simplified rectangular)
        let width = 100.0;
        let height = 50.0;
        let mut y = -params.tool_diameter;

        while y <= height + params.tool_diameter {
            // Move to start of line
            segments.push(PathSegment {
                start: MachinePosition::new(-params.tool_diameter, y, current_z),
                end: MachinePosition::new(width + params.tool_diameter, y, current_z),
                move_type: MoveType::Feed,
                line_number: segments.len(),
            });

            y += stepover;
        }

        // Return to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(width + params.tool_diameter, y - stepover, current_z),
            end: MachinePosition::new(width + params.tool_diameter, y - stepover, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        current_z -= stepdown;
    }

    segments
}

/// Generate scanline machining toolpath
fn generate_scanline(
    min_z: f32,
    max_z: f32,
    stepdown: f32,
    stepover: f32,
    angle: f32,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_z = max_z;
    let angle_rad = angle * PI / 180.0;

    // Scanline machining generates parallel lines at an angle
    while current_z >= min_z {
        // Move to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, current_z),
            end: MachinePosition::new(0.0, 0.0, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Rapid to start position
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, params.safe_z),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Plunge to cutting depth
        segments.push(PathSegment {
            start: MachinePosition::new(
                -params.tool_diameter,
                -params.tool_diameter,
                params.safe_z,
            ),
            end: MachinePosition::new(-params.tool_diameter, -params.tool_diameter, current_z),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Generate scan lines at specified angle
        let width = 100.0;
        let height = 50.0;
        let mut x = -params.tool_diameter;
        let direction = if ((current_z / stepdown) as i32) % 2 == 0 {
            1.0
        } else {
            -1.0
        };

        while x <= width + params.tool_diameter {
            let start_y = -params.tool_diameter;
            let end_y = height + params.tool_diameter;

            // Calculate rotated coordinates
            let start_x_rotated = x * angle_rad.cos() - start_y * angle_rad.sin();
            let start_y_rotated = x * angle_rad.sin() + start_y * angle_rad.cos();
            let end_x_rotated = x * angle_rad.cos() - end_y * angle_rad.sin();
            let end_y_rotated = x * angle_rad.sin() + end_y * angle_rad.cos();

            segments.push(PathSegment {
                start: MachinePosition::new(start_x_rotated, start_y_rotated, current_z),
                end: MachinePosition::new(end_x_rotated, end_y_rotated, current_z),
                move_type: MoveType::Feed,
                line_number: segments.len(),
            });

            x += stepover * direction;
        }

        // Return to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(
                x - stepover * direction,
                height + params.tool_diameter,
                current_z,
            ),
            end: MachinePosition::new(
                x - stepover * direction,
                height + params.tool_diameter,
                params.safe_z,
            ),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        current_z -= stepdown;
    }

    segments
}

/// Generate lathe turning toolpath
fn generate_turning(
    diameter: f32,
    length: f32,
    finish_pass: f32,
    _roughing_feed: f32,
    _finishing_feed: f32,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let radius = diameter / 2.0;

    // Lathe operations use X for diameter/radius and Z for length
    // Move to start position
    segments.push(PathSegment {
        start: MachinePosition::new(0.0, 0.0, params.safe_z),
        end: MachinePosition::new(radius + 2.0, 0.0, params.safe_z),
        move_type: MoveType::Rapid,
        line_number: segments.len(),
    });

    // Roughing passes
    let mut current_radius = radius + 2.0; // Start from outside
    let roughing_step = params.stepover;

    while current_radius > radius + finish_pass {
        current_radius = (current_radius - roughing_step).max(radius + finish_pass);

        // Move to start of pass
        segments.push(PathSegment {
            start: MachinePosition::new(radius + 2.0, 0.0, params.safe_z),
            end: MachinePosition::new(current_radius, 0.0, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Plunge to cutting depth (Z)
        segments.push(PathSegment {
            start: MachinePosition::new(current_radius, 0.0, params.safe_z),
            end: MachinePosition::new(current_radius, 0.0, -length),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Return to start
        segments.push(PathSegment {
            start: MachinePosition::new(current_radius, 0.0, -length),
            end: MachinePosition::new(current_radius, 0.0, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });
    }

    // Finishing pass
    segments.push(PathSegment {
        start: MachinePosition::new(radius + 2.0, 0.0, params.safe_z),
        end: MachinePosition::new(radius, 0.0, params.safe_z),
        move_type: MoveType::Rapid,
        line_number: segments.len(),
    });

    segments.push(PathSegment {
        start: MachinePosition::new(radius, 0.0, params.safe_z),
        end: MachinePosition::new(radius, 0.0, -length),
        move_type: MoveType::Feed,
        line_number: segments.len(),
    });

    segments
}

/// Generate lathe facing toolpath
fn generate_facing(
    diameter: f32,
    width: f32,
    depth: f32,
    stepover: f32,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let radius = diameter / 2.0;

    // Facing cuts across the face of the workpiece
    let mut current_z = 0.0;

    while current_z > -depth {
        current_z = (current_z - stepover).max(-depth);

        // Move to start position
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, params.safe_z),
            end: MachinePosition::new(radius + 2.0, current_z, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Plunge to cutting radius
        segments.push(PathSegment {
            start: MachinePosition::new(radius + 2.0, current_z, params.safe_z),
            end: MachinePosition::new(radius + 2.0, current_z, -width),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Face cut (move in Z direction)
        segments.push(PathSegment {
            start: MachinePosition::new(radius + 2.0, current_z, -width),
            end: MachinePosition::new(radius, current_z, -width),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Return
        segments.push(PathSegment {
            start: MachinePosition::new(radius, current_z, -width),
            end: MachinePosition::new(radius, current_z, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });
    }

    segments
}

/// Generate lathe threading toolpath
fn generate_threading(
    major_diameter: f32,
    minor_diameter: f32,
    _pitch: f32,
    length: f32,
    params: &CAMParameters,
) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let major_radius = major_diameter / 2.0;
    let minor_radius = minor_diameter / 2.0;

    // Threading requires helical interpolation
    // This is a simplified representation
    let _current_z = 0.0;
    let thread_depth = (major_radius - minor_radius) / 2.0; // Total thread depth

    // Multiple threading passes
    for pass in 1..=4 {
        let current_depth = (thread_depth / 4.0) * pass as f32;
        let current_radius = major_radius - current_depth;

        // Move to start
        segments.push(PathSegment {
            start: MachinePosition::new(0.0, 0.0, params.safe_z),
            end: MachinePosition::new(current_radius, 0.0, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });

        // Helical threading pass (simplified as linear for now)
        segments.push(PathSegment {
            start: MachinePosition::new(current_radius, 0.0, params.safe_z),
            end: MachinePosition::new(current_radius, 0.0, -length),
            move_type: MoveType::Feed,
            line_number: segments.len(),
        });

        // Return to safe Z
        segments.push(PathSegment {
            start: MachinePosition::new(current_radius, 0.0, -length),
            end: MachinePosition::new(current_radius, 0.0, params.safe_z),
            move_type: MoveType::Rapid,
            line_number: segments.len(),
        });
    }

    segments
}

/// Add tabs to contour points
fn add_tabs_to_contour(
    points: &[(f32, f32)],
    tab_width: f32,
    _tab_height: f32,
    _z: f32,
) -> Vec<(f32, f32)> {
    let mut result = Vec::new();
    for i in 0..points.len() - 1 {
        let start = points[i];
        let end = points[i + 1];
        result.push(start);

        // Add tab at midpoint of segment if it's long enough
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        let length = (dx * dx + dy * dy).sqrt();

        if length > tab_width * 2.0 {
            // Calculate midpoint
            let mid_x = (start.0 + end.0) / 2.0;
            let mid_y = (start.1 + end.1) / 2.0;

            // Calculate perpendicular direction for tab
            let perp_x = -dy / length * tab_width / 2.0;
            let perp_y = dx / length * tab_width / 2.0;

            // Tab points (leave uncut material)
            let tab_start = (mid_x - perp_x, mid_y - perp_y);
            let tab_end = (mid_x + perp_x, mid_y + perp_y);

            result.push(tab_start);
            result.push(tab_end);
        }
    }
    if let Some(&pt) = points.last() { result.push(pt); };
    result
}

/// Calculate lead point for smooth entry/exit
fn calculate_lead_point(
    start: (f32, f32),
    end: (f32, f32),
    lead_length: f32,
    is_lead_in: bool,
) -> (f32, f32) {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length = (dx * dx + dy * dy).sqrt();

    if length == 0.0 {
        return start;
    }

    let unit_x = dx / length;
    let unit_y = dy / length;

    if is_lead_in {
        (
            start.0 - unit_x * lead_length,
            start.1 - unit_y * lead_length,
        )
    } else {
        (end.0 + unit_x * lead_length, end.1 + unit_y * lead_length)
    }
}

/// Perform part nesting to arrange parts efficiently on a sheet
pub fn perform_part_nesting(
    parts: &[(f32, f32)], // Vec of (width, height) for each part
    config: &PartNestingConfig,
) -> Vec<NestedPart> {
    let mut nested_parts = Vec::new();
    let mut placed_parts: Vec<PlacedPart> = Vec::new();

    // Simple bottom-left fill algorithm
    for (part_index, &(part_width, part_height)) in parts.iter().enumerate() {
        let mut best_position = None;
        let mut best_rotation = 0.0;
        let mut min_waste = f32::INFINITY;

        // Try different rotations if enabled
        let rotations = if config.rotation_enabled {
            config.rotation_angles.clone()
        } else {
            vec![0.0]
        };

        for &rotation in &rotations {
            let (rotated_width, rotated_height) = if rotation == 90.0 || rotation == 270.0 {
                (part_height, part_width)
            } else {
                (part_width, part_height)
            };

            // Try to place at bottom-left of each placed part
            for placed in &placed_parts {
                let test_positions = vec![
                    // Bottom of placed part
                    (placed.x, placed.y + placed.height + config.spacing),
                    // Right of placed part
                    (placed.x + placed.width + config.spacing, placed.y),
                ];

                for (test_x, test_y) in test_positions {
                    if test_x + rotated_width <= config.sheet_width
                        && test_y + rotated_height <= config.sheet_height
                    {
                        // Check for overlap with existing parts
                        let mut overlaps = false;
                        for other in &placed_parts {
                            if rectangles_overlap(
                                test_x,
                                test_y,
                                rotated_width,
                                rotated_height,
                                other.x,
                                other.y,
                                other.width,
                                other.height,
                            ) {
                                overlaps = true;
                                break;
                            }
                        }

                        if !overlaps {
                            // Calculate waste (distance from origin)
                            let waste = test_x + test_y;
                            if waste < min_waste {
                                min_waste = waste;
                                best_position = Some((test_x, test_y));
                                best_rotation = rotation;
                            }
                        }
                    }
                }
            }

            // Also try placing at origin if no good position found
            if best_position.is_none()
                && rotated_width <= config.sheet_width
                && rotated_height <= config.sheet_height
            {
                let mut overlaps = false;
                for other in &placed_parts {
                    if rectangles_overlap(
                        0.0,
                        0.0,
                        rotated_width,
                        rotated_height,
                        other.x,
                        other.y,
                        other.width,
                        other.height,
                    ) {
                        overlaps = true;
                        break;
                    }
                }

                if !overlaps {
                    best_position = Some((0.0, 0.0));
                    best_rotation = rotation;
                }
            }
        }

        // Place the part if a position was found
        if let Some((x, y)) = best_position {
            let (final_width, final_height) = if best_rotation == 90.0 || best_rotation == 270.0 {
                (part_height, part_width)
            } else {
                (part_width, part_height)
            };

            placed_parts.push(PlacedPart {
                x,
                y,
                width: final_width,
                height: final_height,
            });

            nested_parts.push(NestedPart {
                x,
                y,
                rotation: best_rotation,
                part_index,
            });
        }
    }

    nested_parts
}

/// Helper struct for placed parts
#[derive(Clone, Debug)]
struct PlacedPart {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Check if two rectangles overlap
#[allow(clippy::too_many_arguments)]
fn rectangles_overlap(
    x1: f32,
    y1: f32,
    w1: f32,
    h1: f32,
    x2: f32,
    y2: f32,
    w2: f32,
    h2: f32,
) -> bool {
    !(x1 + w1 <= x2 || x2 + w2 <= x1 || y1 + h1 <= y2 || y2 + h2 <= y1)
}
