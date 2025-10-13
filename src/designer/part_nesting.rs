use super::Shape;

/// Represents a rectangular bounding box
#[derive(Clone, Debug)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Represents a placed part on the sheet
#[derive(Clone, Debug)]
pub struct PlacedPart {
    pub shape_index: usize,
    pub x: f32,
    pub y: f32,
    pub rotation: f32, // degrees
    pub bounding_box: BoundingBox,
}

/// Nesting result containing placed parts and utilization
#[derive(Clone, Debug)]
pub struct NestingResult {
    pub placed_parts: Vec<PlacedPart>,
    pub sheet_width: f32,
    pub sheet_height: f32,
    pub utilization: f32, // percentage 0-100
}

/// Calculate bounding box for a shape
pub fn calculate_bounding_box(shape: &Shape) -> BoundingBox {
    match shape {
        Shape::Rectangle {
            x,
            y,
            width,
            height,
        } => BoundingBox {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        },
        Shape::Circle { x, y, radius } => BoundingBox {
            x: *x - *radius,
            y: *y - *radius,
            width: *radius * 2.0,
            height: *radius * 2.0,
        },
        Shape::Line { x1, y1, x2, y2 } => {
            let min_x = x1.min(*x2);
            let max_x = x1.max(*x2);
            let min_y = y1.min(*y2);
            let max_y = y1.max(*y2);
            BoundingBox {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            }
        }
        Shape::Text {
            x,
            y,
            text,
            font_size,
        } => {
            // Approximate text bounding box
            let width = text.len() as f32 * *font_size * 0.6;
            let height = *font_size;
            BoundingBox {
                x: *x,
                y: *y - height,
                width,
                height,
            }
        }
        Shape::Drill { x, y, .. } => BoundingBox {
            x: *x - 1.0,
            y: *y - 1.0,
            width: 2.0,
            height: 2.0,
        },
        Shape::Pocket {
            x,
            y,
            width,
            height,
            ..
        } => BoundingBox {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        },
        Shape::Cylinder {
            x,
            y,
            radius,
            height,
            ..
        } => BoundingBox {
            x: *x - *radius,
            y: *y,
            width: *radius * 2.0,
            height: *height,
        },
        Shape::Sphere { x, y, radius, .. } => BoundingBox {
            x: *x - *radius,
            y: *y - *radius,
            width: *radius * 2.0,
            height: *radius * 2.0,
        },
        // Add more shape types as needed
        _ => BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        }, // Default
    }
}

/// Simple bottom-left-fill nesting algorithm
pub fn bottom_left_fill_nesting(
    shapes: &[Shape],
    sheet_width: f32,
    sheet_height: f32,
    spacing: f32,
) -> NestingResult {
    let mut placed_parts = Vec::new();
    let mut occupied_regions = Vec::new();

    for (i, shape) in shapes.iter().enumerate() {
        let bbox = calculate_bounding_box(shape);

        // Try to place the part
        let placement =
            find_bottom_left_position(&bbox, sheet_width, sheet_height, &occupied_regions, spacing);

        if let Some((x, y)) = placement {
            let placed_part = PlacedPart {
                shape_index: i,
                x,
                y,
                rotation: 0.0, // No rotation for now
                bounding_box: BoundingBox {
                    x,
                    y,
                    width: bbox.width,
                    height: bbox.height,
                },
            };

            placed_parts.push(placed_part);
            occupied_regions.push(BoundingBox {
                x: x - spacing,
                y: y - spacing,
                width: bbox.width + 2.0 * spacing,
                height: bbox.height + 2.0 * spacing,
            });
        }
        // If can't place, skip for now (could implement sheet expansion or rejection)
    }

    let total_area: f32 = placed_parts
        .iter()
        .map(|p| p.bounding_box.width * p.bounding_box.height)
        .sum();
    let sheet_area = sheet_width * sheet_height;
    let utilization = if sheet_area > 0.0 {
        (total_area / sheet_area) * 100.0
    } else {
        0.0
    };

    NestingResult {
        placed_parts,
        sheet_width,
        sheet_height,
        utilization,
    }
}

/// Find the bottom-left position for a bounding box
fn find_bottom_left_position(
    bbox: &BoundingBox,
    sheet_width: f32,
    sheet_height: f32,
    occupied: &[BoundingBox],
    spacing: f32,
) -> Option<(f32, f32)> {
    // Simple grid search - in practice, you'd want a more sophisticated approach
    let step = 1.0; // Grid resolution

    for y in (0..=((sheet_height - bbox.height) / step) as i32).map(|i| i as f32 * step) {
        for x in (0..=((sheet_width - bbox.width) / step) as i32).map(|i| i as f32 * step) {
            let test_box = BoundingBox {
                x: x - spacing,
                y: y - spacing,
                width: bbox.width + 2.0 * spacing,
                height: bbox.height + 2.0 * spacing,
            };

            if !overlaps_any(&test_box, occupied) {
                return Some((x, y));
            }
        }
    }

    None
}

/// Check if a bounding box overlaps with any in the list
fn overlaps_any(bbox: &BoundingBox, others: &[BoundingBox]) -> bool {
    for other in others {
        if bbox.x < other.x + other.width
            && bbox.x + bbox.width > other.x
            && bbox.y < other.y + other.height
            && bbox.y + bbox.height > other.y
        {
            return true;
        }
    }
    false
}

/// Optimize nesting by trying different arrangements
pub fn optimize_nesting(
    shapes: &[Shape],
    sheet_width: f32,
    sheet_height: f32,
    spacing: f32,
) -> NestingResult {
    // For now, just use bottom-left-fill
    // In the future, could try rotations, different algorithms, etc.
    bottom_left_fill_nesting(shapes, sheet_width, sheet_height, spacing)
}
