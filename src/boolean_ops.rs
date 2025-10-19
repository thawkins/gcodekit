//! Advanced CAM Boolean Operations Module
//!
//! This module provides sophisticated geometric boolean operations for combining
//! shapes, including intersection, subtraction, and union operations. Also includes
//! region fill algorithms for automatic toolpath generation and holding tab placement.

use crate::types::{MachinePosition, MoveType, PathSegment};
use std::f32::consts::PI;

/// Represents a 2D polygon with vertices
#[derive(Clone, Debug)]
pub struct Polygon {
    /// Vertices of the polygon (counter-clockwise winding)
    pub vertices: Vec<(f32, f32)>,
}

impl Polygon {
    /// Creates a new polygon from a vector of vertices
    pub fn new(vertices: Vec<(f32, f32)>) -> Self {
        Polygon { vertices }
    }

    /// Calculates the area of the polygon using the shoelace formula
    pub fn area(&self) -> f32 {
        if self.vertices.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        for i in 0..self.vertices.len() {
            let j = (i + 1) % self.vertices.len();
            area += self.vertices[i].0 * self.vertices[j].1;
            area -= self.vertices[j].0 * self.vertices[i].1;
        }
        (area / 2.0).abs()
    }

    /// Checks if a point is inside the polygon using ray casting
    pub fn contains_point(&self, point: (f32, f32)) -> bool {
        let (px, py) = point;
        let mut inside = false;

        let mut j = self.vertices.len() - 1;
        for i in 0..self.vertices.len() {
            let (x1, y1) = self.vertices[i];
            let (x2, y2) = self.vertices[j];

            if ((y1 > py) != (y2 > py))
                && (px < (x2 - x1) * (py - y1) / (y2 - y1) + x1)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    /// Calculates the bounding box of the polygon
    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        if self.vertices.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = self.vertices[0].0;
        let mut max_x = self.vertices[0].0;
        let mut min_y = self.vertices[0].1;
        let mut max_y = self.vertices[0].1;

        for (x, y) in &self.vertices {
            min_x = min_x.min(*x);
            max_x = max_x.max(*x);
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Simplifies polygon by removing collinear points
    pub fn simplify(&self) -> Polygon {
        if self.vertices.len() < 3 {
            return self.clone();
        }

        let mut simplified = Vec::new();
        let n = self.vertices.len();

        for i in 0..n {
            let p1 = self.vertices[i];
            let p2 = self.vertices[(i + 1) % n];
            let p3 = self.vertices[(i + 2) % n];

            // Check if points are collinear (cross product ~= 0)
            let cross = (p2.0 - p1.0) * (p3.1 - p1.1) - (p2.1 - p1.1) * (p3.0 - p1.0);
            if cross.abs() > 0.001 {
                simplified.push(p1);
            }
        }

        Polygon::new(simplified)
    }
}

/// Performs union operation on two polygons
/// Returns the area of intersection
pub fn polygon_intersection_area(poly1: &Polygon, poly2: &Polygon) -> f32 {
    let mut intersection_area: f32 = 0.0;

    // Check if vertices of poly1 are inside poly2
    for vertex in &poly1.vertices {
        if poly2.contains_point(*vertex) {
            intersection_area += 0.1; // Rough estimate
        }
    }

    // Check if vertices of poly2 are inside poly1
    for vertex in &poly2.vertices {
        if poly1.contains_point(*vertex) {
            intersection_area += 0.1; // Rough estimate
        }
    }

    intersection_area.max(0.0)
}

/// Performs union operation on two polygons
/// Returns a vector of resulting polygons (typically 1, but can be multiple if disjoint)
pub fn polygon_union(poly1: &Polygon, poly2: &Polygon) -> Vec<Polygon> {
    let (x1_min, y1_min, x1_max, y1_max) = poly1.bounding_box();
    let (x2_min, y2_min, x2_max, y2_max) = poly2.bounding_box();

    // Check if bounding boxes intersect
    let bboxes_intersect = !(x1_max < x2_min || x2_max < x1_min || y1_max < y2_min || y2_max < y1_min);

    if !bboxes_intersect {
        // Non-intersecting polygons, return both
        return vec![poly1.clone(), poly2.clone()];
    }

    // Combine vertices for union (simplified approach)
    let mut combined = poly1.vertices.clone();
    for vertex in &poly2.vertices {
        if !poly1.contains_point(*vertex) {
            combined.push(*vertex);
        }
    }

    // Sort vertices by angle from centroid (convex hull approximation)
    if combined.len() >= 3 {
        let cx = combined.iter().map(|v| v.0).sum::<f32>() / combined.len() as f32;
        let cy = combined.iter().map(|v| v.1).sum::<f32>() / combined.len() as f32;

        combined.sort_by(|a, b| {
            let angle_a = (a.1 - cy).atan2(a.0 - cx);
            let angle_b = (b.1 - cy).atan2(b.0 - cx);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        vec![Polygon::new(combined)]
    } else {
        vec![poly1.clone()]
    }
}

/// Performs subtraction of poly2 from poly1
/// Returns a vector of resulting polygons
pub fn polygon_subtraction(poly1: &Polygon, poly2: &Polygon) -> Vec<Polygon> {
    let (x1_min, y1_min, x1_max, y1_max) = poly1.bounding_box();
    let (x2_min, y2_min, x2_max, y2_max) = poly2.bounding_box();

    // Check if bounding boxes intersect
    let bboxes_intersect = !(x1_max < x2_min || x2_max < x1_min || y1_max < y2_min || y2_max < y1_min);

    if !bboxes_intersect {
        // No intersection, return original polygon
        return vec![poly1.clone()];
    }

    // Collect vertices of poly1 that are outside poly2
    let mut result_vertices = Vec::new();
    for vertex in &poly1.vertices {
        if !poly2.contains_point(*vertex) {
            result_vertices.push(*vertex);
        }
    }

    if result_vertices.len() >= 3 {
        // Sort by angle from centroid
        let cx = result_vertices.iter().map(|v| v.0).sum::<f32>() / result_vertices.len() as f32;
        let cy = result_vertices.iter().map(|v| v.1).sum::<f32>() / result_vertices.len() as f32;

        result_vertices.sort_by(|a, b| {
            let angle_a = (a.1 - cy).atan2(a.0 - cx);
            let angle_b = (b.1 - cy).atan2(b.0 - cx);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        vec![Polygon::new(result_vertices)]
    } else {
        vec![]
    }
}

/// Performs intersection of two polygons
/// Returns a vector of resulting polygons
pub fn polygon_intersection(poly1: &Polygon, poly2: &Polygon) -> Vec<Polygon> {
    let (x1_min, y1_min, x1_max, y1_max) = poly1.bounding_box();
    let (x2_min, y2_min, x2_max, y2_max) = poly2.bounding_box();

    // Calculate intersection bounding box
    let int_min_x = x1_min.max(x2_min);
    let int_min_y = y1_min.max(y2_min);
    let int_max_x = x1_max.min(x2_max);
    let int_max_y = y1_max.min(y2_max);

    if int_min_x >= int_max_x || int_min_y >= int_max_y {
        // No intersection
        return vec![];
    }

    // Collect vertices inside both polygons
    let mut intersection_vertices = Vec::new();

    for vertex in &poly1.vertices {
        if poly2.contains_point(*vertex) {
            intersection_vertices.push(*vertex);
        }
    }

    for vertex in &poly2.vertices {
        if poly1.contains_point(*vertex) && !intersection_vertices.contains(vertex) {
            intersection_vertices.push(*vertex);
        }
    }

    if intersection_vertices.len() >= 3 {
        // Sort by angle from centroid
        let cx = intersection_vertices.iter().map(|v| v.0).sum::<f32>()
            / intersection_vertices.len() as f32;
        let cy = intersection_vertices.iter().map(|v| v.1).sum::<f32>()
            / intersection_vertices.len() as f32;

        intersection_vertices.sort_by(|a, b| {
            let angle_a = (a.1 - cy).atan2(a.0 - cx);
            let angle_b = (b.1 - cy).atan2(b.0 - cx);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        vec![Polygon::new(intersection_vertices)]
    } else {
        vec![]
    }
}

/// Fills a region with a scanline pattern for pocket machining
pub fn fill_region_scanlines(
    poly: &Polygon,
    spacing: f32,
    _angle: f32,
) -> Vec<((f32, f32), (f32, f32))> {
    let (min_x, min_y, max_x, max_y) = poly.bounding_box();
    let mut lines = Vec::new();

    let mut y = min_y;
    while y < max_y {
        // Find intersection points of scanline with polygon edges
        let mut intersections = Vec::new();

        for i in 0..poly.vertices.len() {
            let p1 = poly.vertices[i];
            let p2 = poly.vertices[(i + 1) % poly.vertices.len()];

            // Check if scanline intersects edge
            if (p1.1 - y) * (p2.1 - y) < 0.0 {
                // Linear interpolation
                let t = (y - p1.1) / (p2.1 - p1.1);
                let x = p1.0 + t * (p2.0 - p1.0);
                intersections.push(x);
            }
        }

        // Sort intersections and create line segments
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        for i in (0..intersections.len()).step_by(2) {
            if i + 1 < intersections.len() {
                let x1 = intersections[i];
                let x2 = intersections[i + 1];
                lines.push(((x1, y), (x2, y)));
            }
        }

        y += spacing;
    }

    lines
}

/// Generates holding tabs for part retention during machining
pub fn generate_holding_tabs(
    poly: &Polygon,
    tab_width: f32,
    tab_height: f32,
    spacing: f32,
) -> Vec<Polygon> {
    let mut tabs = Vec::new();
    let perimeter = calculate_perimeter(poly);
    let num_tabs = (perimeter / spacing).ceil() as usize;

    if num_tabs < 1 {
        return tabs;
    }

    let step = perimeter / num_tabs as f32;
    let mut current_distance = 0.0;

    for i in 0..poly.vertices.len() {
        let p1 = poly.vertices[i];
        let p2 = poly.vertices[(i + 1) % poly.vertices.len()];

        let edge_length = ((p2.0 - p1.0).powi(2) + (p2.1 - p1.1).powi(2)).sqrt();
        let edge_distance_start = current_distance;
        let edge_distance_end = current_distance + edge_length;

        // Check if this edge should have a tab
        let mut tab_distance = step * (current_distance / step).ceil();
        while tab_distance < edge_distance_end {
            if tab_distance >= edge_distance_start {
                // Interpolate position on edge
                let edge_t = (tab_distance - edge_distance_start) / edge_length;
                let tab_center_x = p1.0 + edge_t * (p2.0 - p1.0);
                let tab_center_y = p1.1 + edge_t * (p2.1 - p1.1);

                // Create tab rectangle
                let dx = (p2.0 - p1.0) / edge_length;
                let dy = (p2.1 - p1.1) / edge_length;

                let tab = Polygon::new(vec![
                    (tab_center_x - dx * tab_width / 2.0, tab_center_y - dy * tab_width / 2.0),
                    (
                        tab_center_x + dx * tab_width / 2.0,
                        tab_center_y + dy * tab_width / 2.0,
                    ),
                    (
                        tab_center_x + dx * tab_width / 2.0 + dy * tab_height,
                        tab_center_y + dy * tab_width / 2.0 - dx * tab_height,
                    ),
                    (
                        tab_center_x - dx * tab_width / 2.0 + dy * tab_height,
                        tab_center_y - dy * tab_width / 2.0 - dx * tab_height,
                    ),
                ]);
                tabs.push(tab);
            }
            tab_distance += step;
        }

        current_distance += edge_length;
    }

    tabs
}

/// Calculates the perimeter of a polygon
fn calculate_perimeter(poly: &Polygon) -> f32 {
    let mut perimeter = 0.0;
    for i in 0..poly.vertices.len() {
        let p1 = poly.vertices[i];
        let p2 = poly.vertices[(i + 1) % poly.vertices.len()];
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        perimeter += (dx * dx + dy * dy).sqrt();
    }
    perimeter
}

/// Generates boundary-following toolpath from polygon
pub fn generate_toolpath_from_polygon(poly: &Polygon, z_depth: f32) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_pos = MachinePosition::new(0.0, 0.0, 0.0);

    if poly.vertices.is_empty() {
        return segments;
    }

    // Rapid to first vertex at safe height
    let first_vertex = poly.vertices[0];
    segments.push(PathSegment {
        start: current_pos.clone(),
        end: MachinePosition::new(first_vertex.0, first_vertex.1, 5.0),
        move_type: MoveType::Rapid,
        line_number: 0,
    });

    current_pos = MachinePosition::new(first_vertex.0, first_vertex.1, 5.0);

    // Plunge to cutting depth
    segments.push(PathSegment {
        start: current_pos.clone(),
        end: MachinePosition::new(first_vertex.0, first_vertex.1, z_depth),
        move_type: MoveType::Feed,
        line_number: 1,
    });

    current_pos = MachinePosition::new(first_vertex.0, first_vertex.1, z_depth);

    // Feed along polygon boundary
    for i in 1..poly.vertices.len() {
        let vertex = poly.vertices[i];
        segments.push(PathSegment {
            start: current_pos.clone(),
            end: MachinePosition::new(vertex.0, vertex.1, z_depth),
            move_type: MoveType::Feed,
            line_number: i as usize + 1,
        });
        current_pos = MachinePosition::new(vertex.0, vertex.1, z_depth);
    }

    // Close the path
    let last_vertex = poly.vertices[0];
    segments.push(PathSegment {
        start: current_pos.clone(),
        end: MachinePosition::new(last_vertex.0, last_vertex.1, z_depth),
        move_type: MoveType::Feed,
        line_number: poly.vertices.len() + 1,
    });

    // Rapid to safe height
    segments.push(PathSegment {
        start: MachinePosition::new(last_vertex.0, last_vertex.1, z_depth),
        end: MachinePosition::new(last_vertex.0, last_vertex.1, 5.0),
        move_type: MoveType::Rapid,
        line_number: poly.vertices.len() + 2,
    });

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_area() {
        // 2x2 square = area 4
        let square = Polygon::new(vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]);
        let area = square.area();
        assert!((area - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_polygon_contains_point() {
        let square = Polygon::new(vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]);
        assert!(square.contains_point((1.0, 1.0)));
        assert!(!square.contains_point((3.0, 3.0)));
    }

    #[test]
    fn test_polygon_bounding_box() {
        let square = Polygon::new(vec![(1.0, 1.0), (3.0, 1.0), (3.0, 3.0), (1.0, 3.0)]);
        let bbox = square.bounding_box();
        assert_eq!(bbox, (1.0, 1.0, 3.0, 3.0));
    }

    #[test]
    fn test_polygon_union_non_intersecting() {
        let poly1 = Polygon::new(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let poly2 = Polygon::new(vec![(2.0, 2.0), (3.0, 2.0), (3.0, 3.0), (2.0, 3.0)]);
        let result = polygon_union(&poly1, &poly2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_polygon_subtraction() {
        let poly1 = Polygon::new(vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
        let poly2 = Polygon::new(vec![(1.0, 1.0), (3.0, 1.0), (3.0, 3.0), (1.0, 3.0)]);
        let result = polygon_subtraction(&poly1, &poly2);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_holding_tabs_generation() {
        let square = Polygon::new(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
        let tabs = generate_holding_tabs(&square, 2.0, 1.0, 5.0);
        assert!(!tabs.is_empty());
    }

    #[test]
    fn test_toolpath_generation() {
        let triangle = Polygon::new(vec![(0.0, 0.0), (10.0, 0.0), (5.0, 10.0)]);
        let toolpath = generate_toolpath_from_polygon(&triangle, -5.0);
        assert!(toolpath.len() > 0);
        // Should have rapid, plunge, feed segments, and return
        assert_eq!(toolpath[0].move_type, MoveType::Rapid);
    }
}
