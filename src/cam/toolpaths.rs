//! Toolpath generation algorithms for CAM operations.
//!
//! This module provides functions for generating machining toolpaths from
//! 2D shapes and 3D surfaces, including contouring, pocketing, and 3D machining.

use crate::cam::types::*;

/// Generate toolpath for waterline machining
/// Slices the 3D mesh at regular Z intervals and generates contour paths
pub fn generate_waterline_toolpath(
    mesh: &Mesh,
    params: &CAMParameters,
    operation: &CAMOperation,
) -> Vec<String> {
    let mut gcode = Vec::new();

    if let CAMOperation::Waterline {
        min_z,
        max_z,
        stepdown,
        stepover,
    } = operation
    {
        // Initialize G-code
        gcode.push("G90".to_string()); // Absolute positioning
        gcode.push("G21".to_string()); // Metric units
        gcode.push(format!("G0 Z{}", params.safe_z)); // Move to safe Z

        let mut current_z = *max_z;
        while current_z >= *min_z {
            // Generate contour at this Z level
            let contours = slice_mesh_at_z(mesh, current_z);

            for contour in contours {
                // Generate G-code for this contour
                if !contour.is_empty() {
                    // Move to start of contour at safe Z
                    gcode.push(format!("G0 X{:.3} Y{:.3}", contour[0].x, contour[0].y));
                    gcode.push(format!("G0 Z{:.3}", current_z + 1.0)); // Approach height
                    gcode.push(format!("G1 Z{:.3} F{}", current_z, params.plunge_rate));

                    // Cut the contour
                    for point in contour.iter().skip(1) {
                        gcode.push(format!(
                            "G1 X{:.3} Y{:.3} F{}",
                            point.x, point.y, params.feed_rate
                        ));
                    }

                    // Close the contour if it's a loop
                    if contour.len() > 2 {
                        gcode.push(format!("G1 X{:.3} Y{:.3}", contour[0].x, contour[0].y));
                    }

                    // Retract
                    gcode.push(format!("G0 Z{}", params.safe_z));
                }
            }

            current_z -= stepdown;
        }
    }

    gcode
}

/// Generate toolpath for scanline machining
/// Generates parallel lines at each Z level and finds surface intersections
pub fn generate_scanline_toolpath(
    mesh: &Mesh,
    params: &CAMParameters,
    operation: &CAMOperation,
) -> Vec<String> {
    let mut gcode = Vec::new();

    if let CAMOperation::Scanline {
        min_z,
        max_z,
        stepdown,
        stepover,
        angle,
    } = operation
    {
        // Initialize G-code
        gcode.push("G90".to_string());
        gcode.push("G21".to_string());
        gcode.push(format!("G0 Z{}", params.safe_z));

        let angle_rad = angle.to_radians();
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        let mut current_z = *max_z;
        while current_z >= *min_z {
            // Generate scan lines at this Z level
            let scan_lines = generate_scan_lines(mesh, current_z, *stepover, angle_rad);

            for line in scan_lines {
                if !line.is_empty() {
                    // Sort points along the scan direction
                    let mut sorted_line = line;
                    sorted_line.sort_by(|a, b| {
                        let a_proj = a.x * cos_angle + a.y * sin_angle;
                        let b_proj = b.x * cos_angle + b.y * sin_angle;
                        a_proj.partial_cmp(&b_proj).unwrap()
                    });

                    // Move to start
                    gcode.push(format!(
                        "G0 X{:.3} Y{:.3}",
                        sorted_line[0].x, sorted_line[0].y
                    ));
                    gcode.push(format!("G0 Z{:.3}", current_z + 1.0));
                    gcode.push(format!("G1 Z{:.3} F{}", current_z, params.plunge_rate));

                    // Cut along the line
                    for point in sorted_line.iter().skip(1) {
                        gcode.push(format!(
                            "G1 X{:.3} Y{:.3} F{}",
                            point.x, point.y, params.feed_rate
                        ));
                    }

                    // Retract
                    gcode.push(format!("G0 Z{}", params.safe_z));
                }
            }

            current_z -= stepdown;
        }
    }

    gcode
}

/// Slice mesh at given Z height and return contour paths
fn slice_mesh_at_z(mesh: &Mesh, z: f32) -> Vec<Vec<Point3D>> {
    let mut contours = Vec::new();

    for triangle in &mesh.triangles {
        if let Some(segment) = intersect_triangle_with_plane(triangle, z) {
            contours.push(vec![segment.0, segment.1]);
        }
    }

    // TODO: Connect segments into continuous contours
    // For now, return individual segments
    contours
}

/// Generate scan lines at Z height
fn generate_scan_lines(mesh: &Mesh, z: f32, stepover: f32, angle: f32) -> Vec<Vec<Point3D>> {
    let mut scan_lines = Vec::new();

    // Simple implementation: generate horizontal lines across bounding box
    let bounds = &mesh.bounds;
    let width = bounds.max.x - bounds.min.x;
    let height = bounds.max.y - bounds.min.y;

    let num_lines = ((width.max(height)) / stepover) as usize;

    for i in 0..num_lines {
        let offset = i as f32 * stepover;
        let mut line_points = Vec::new();

        // Generate points along the line
        let steps = 100; // Resolution
        for j in 0..steps {
            let t = j as f32 / (steps - 1) as f32;
            let x = bounds.min.x + t * width;
            let y = bounds.min.y + offset;

            // Rotate by angle
            let rotated_x = x * angle.cos() - y * angle.sin();
            let rotated_y = x * angle.sin() + y * angle.cos();

            // Check if point is on surface
            if point_on_surface(mesh, rotated_x, rotated_y, z) {
                line_points.push(Point3D {
                    x: rotated_x,
                    y: rotated_y,
                    z,
                });
            }
        }

        if !line_points.is_empty() {
            scan_lines.push(line_points);
        }
    }

    scan_lines
}

/// Check if point is on the surface at given Z
fn point_on_surface(mesh: &Mesh, x: f32, y: f32, _z: f32) -> bool {
    // Simple bounding box check for now
    x >= mesh.bounds.min.x
        && x <= mesh.bounds.max.x
        && y >= mesh.bounds.min.y
        && y <= mesh.bounds.max.y
    // TODO: Implement proper ray casting or surface intersection
}

/// Intersect triangle with horizontal plane at Z
fn intersect_triangle_with_plane(triangle: &Triangle, z: f32) -> Option<(Point3D, Point3D)> {
    let vertices = &triangle.vertices;
    let mut above = Vec::new();
    let mut below = Vec::new();
    let mut on_plane = Vec::new();

    for vertex in vertices {
        if (vertex.z - z).abs() < 0.001 {
            on_plane.push(*vertex);
        } else if vertex.z > z {
            above.push(*vertex);
        } else {
            below.push(*vertex);
        }
    }

    match (above.len(), below.len(), on_plane.len()) {
        (1, 2, 0) | (2, 1, 0) => {
            // Intersect the edge between above and below points
            let (above_point, below_point) = if above.len() == 1 {
                (
                    above[0],
                    if below[0].z < below[1].z {
                        below[0]
                    } else {
                        below[1]
                    },
                )
            } else {
                (
                    below[0],
                    if above[0].z > above[1].z {
                        above[0]
                    } else {
                        above[1]
                    },
                )
            };

            let t = (z - below_point.z) / (above_point.z - below_point.z);
            let intersection = Point3D {
                x: below_point.x + t * (above_point.x - below_point.x),
                y: below_point.y + t * (above_point.y - below_point.y),
                z,
            };

            // Find second intersection
            let other_above = above
                .iter()
                .find(|p| *p != &above_point)
                .or_else(|| below.iter().find(|p| *p != &below_point))?;
            let t2 = (z - below_point.z) / (other_above.z - below_point.z);
            let intersection2 = Point3D {
                x: below_point.x + t2 * (other_above.x - below_point.x),
                y: below_point.y + t2 * (other_above.y - below_point.y),
                z,
            };

            Some((intersection, intersection2))
        }
        _ => None, // No intersection or degenerate cases
    }
}
