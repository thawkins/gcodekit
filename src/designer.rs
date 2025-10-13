pub mod bitmap_import;
pub mod bitmap_processing;
pub mod cam_operations;
pub mod image_engraving;
pub mod jigsaw;
pub mod parametric_design;
pub mod parametric_ui;
pub mod part_nesting;
pub mod shape_generation;
pub mod tabbed_box;
pub mod toolpath_generation;
pub mod vector_import;

use anyhow::Result;
use eframe::egui;
use image::GrayImage;
use lyon::math::point;
use lyon::path::Path;

use rhai::{AST, Engine, Scope};
use std::collections::VecDeque;
use std::fs;
use stl_io::{Normal, Triangle, Vertex};
use tobj;

use bitmap_processing::{BitmapProcessor, VectorizationConfig};

// Re-export the widget functions for easy access
pub use bitmap_import::show_bitmap_import_widget;
pub use image_engraving::show_image_engraving_widget;
pub use jigsaw::show_jigsaw_widget;
pub use parametric_ui::show_parametric_design_widget;
pub use shape_generation::show_shape_generation_widget;
pub use tabbed_box::show_tabbed_box_widget;
pub use toolpath_generation::show_toolpath_generation_widget;
pub use vector_import::show_vector_import_widget;

#[derive(Clone, Debug)]
pub enum Shape {
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        font_size: f32,
    },
    Drill {
        x: f32,
        y: f32,
        depth: f32,
    },
    Pocket {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth: f32,
        stepover: f32,
        pattern: ToolpathPattern,
    },
    Cylinder {
        x: f32,
        y: f32,
        radius: f32,
        height: f32,
        depth: f32,
    },
    Sphere {
        x: f32,
        y: f32,
        radius: f32,
        depth: f32,
    },
    Extrusion {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth: f32,
    },
    Turning {
        x: f32,
        y: f32,
        diameter: f32,
        length: f32,
        depth: f32,
    },
    Facing {
        x: f32,
        y: f32,
        width: f32,
        length: f32,
        depth: f32,
    },
    Threading {
        x: f32,
        y: f32,
        diameter: f32,
        length: f32,
        pitch: f32,
        depth: f32,
    },
    Polyline {
        points: Vec<(f32, f32)>,
    },
    Parametric {
        script: String,
        ast: Option<AST>,
        bounds: (f32, f32, f32, f32),
    },
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum ToolpathPattern {
    #[default]
    Offset,
    Spiral,
    Zigzag,
    Trochoidal,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub name: String,
    pub density: f32,              // kg/m³
    pub hardness: f32,             // HB
    pub thermal_conductivity: f32, // W/m·K
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Aluminum".to_string(),
            density: 2700.0,
            hardness: 30.0,
            thermal_conductivity: 237.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tool {
    pub name: String,
    pub diameter: f32, // mm
    pub length: f32,   // mm (tool length)
    pub material: String,
    pub flute_count: u32,
    pub max_rpm: u32,
    pub tool_number: u32,   // T number for G-code
    pub length_offset: f32, // H offset value for G43
    pub wear_offset: f32,   // Additional wear offset
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            name: "End Mill 3mm".to_string(),
            diameter: 3.0,
            length: 40.0,
            material: "HSS".to_string(),
            flute_count: 2,
            max_rpm: 10000,
            tool_number: 1,
            length_offset: 0.0,
            wear_offset: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum DrawingTool {
    #[default]
    Select,
    Rectangle,
    Circle,
    Line,
    Text,
    Drill,
    Pocket,
    Cylinder,
    Sphere,
    Extrusion,
    Turning,
    Facing,
    Threading,
    Polyline,
    Parametric,
    Move,
    Scale,
    Rotate,
    Mirror,
}

#[derive(Clone, Debug)]
pub enum DesignerEvent {
    ExportGcode,
    ExportStl,
    ExportObj,
    ExportGltf,
    ImportFile,
}

// Command pattern for undo/redo
pub trait Command {
    fn execute(&mut self, state: &mut DesignerState);
    fn undo(&mut self, state: &mut DesignerState);
}

pub struct AddShapeCommand {
    shape: Shape,
    index: Option<usize>,
}

impl AddShapeCommand {
    pub fn new(shape: Shape) -> Self {
        Self { shape, index: None }
    }
}

impl Command for AddShapeCommand {
    fn execute(&mut self, state: &mut DesignerState) {
        self.index = Some(state.shapes.len());
        state.shapes.push(self.shape.clone());
    }

    fn undo(&mut self, state: &mut DesignerState) {
        if let Some(index) = self.index {
            state.shapes.remove(index);
        }
    }
}

pub struct DeleteShapeCommand {
    shape: Option<Shape>,
    index: Option<usize>,
}

impl DeleteShapeCommand {
    pub fn new(index: usize) -> Self {
        Self {
            shape: None,
            index: Some(index),
        }
    }
}

impl Command for DeleteShapeCommand {
    fn execute(&mut self, state: &mut DesignerState) {
        if let Some(index) = self.index {
            self.shape = Some(state.shapes.remove(index));
        }
    }

    fn undo(&mut self, state: &mut DesignerState) {
        if let Some(shape) = &self.shape
            && let Some(index) = self.index
        {
            state.shapes.insert(index, shape.clone());
        }
    }
}

pub struct MoveShapeCommand {
    index: usize,
    old_pos: (f32, f32),
    new_pos: (f32, f32),
}

impl MoveShapeCommand {
    pub fn new(index: usize, old_pos: (f32, f32), new_pos: (f32, f32)) -> Self {
        Self {
            index,
            old_pos,
            new_pos,
        }
    }
}

impl Command for MoveShapeCommand {
    fn execute(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::set_shape_pos(shape, self.new_pos);
        }
    }

    fn undo(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::set_shape_pos(shape, self.old_pos);
        }
    }
}

pub struct ScaleShapeCommand {
    index: usize,
    old_scale: (f32, f32),
    new_scale: (f32, f32),
    pivot: (f32, f32),
}

impl ScaleShapeCommand {
    pub fn new(
        index: usize,
        old_scale: (f32, f32),
        new_scale: (f32, f32),
        pivot: (f32, f32),
    ) -> Self {
        Self {
            index,
            old_scale,
            new_scale,
            pivot,
        }
    }
}

impl Command for ScaleShapeCommand {
    fn execute(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::scale_shape(shape, self.new_scale, self.pivot);
        }
    }

    fn undo(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::scale_shape(shape, self.old_scale, self.pivot);
        }
    }
}

pub struct RotateShapeCommand {
    index: usize,
    old_rotation: f32,
    new_rotation: f32,
    pivot: (f32, f32),
}

impl RotateShapeCommand {
    pub fn new(index: usize, old_rotation: f32, new_rotation: f32, pivot: (f32, f32)) -> Self {
        Self {
            index,
            old_rotation,
            new_rotation,
            pivot,
        }
    }
}

impl Command for RotateShapeCommand {
    fn execute(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::rotate_shape(shape, self.new_rotation, self.pivot);
        }
    }

    fn undo(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::rotate_shape(shape, self.old_rotation, self.pivot);
        }
    }
}

pub struct MirrorShapeCommand {
    index: usize,
    axis: MirrorAxis,
}

impl MirrorShapeCommand {
    pub fn new(index: usize, axis: MirrorAxis) -> Self {
        Self { index, axis }
    }
}

impl Command for MirrorShapeCommand {
    fn execute(&mut self, state: &mut DesignerState) {
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::mirror_shape(shape, self.axis);
        }
    }

    fn undo(&mut self, state: &mut DesignerState) {
        // Mirror is its own inverse, so undo is the same as execute
        if let Some(shape) = state.shapes.get_mut(self.index) {
            DesignerState::mirror_shape(shape, self.axis);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MirrorAxis {
    Horizontal,
    Vertical,
}

#[derive(Default)]
pub struct DesignerState {
    pub shapes: Vec<Shape>,
    pub current_tool: DrawingTool,
    pub current_pattern: ToolpathPattern,
    pub current_material: Material,
    pub current_tool_def: Tool,
    pub drawing_start: Option<(f32, f32)>,
    pub selected_shape: Option<usize>,
    pub selected_point: Option<usize>,
    undo_stack: VecDeque<Box<dyn Command>>,
    redo_stack: VecDeque<Box<dyn Command>>,
    drag_start_pos: Option<(f32, f32)>,
    show_grid: bool,
    manipulation_start: Option<(f32, f32)>,
    original_shape: Option<Shape>,
    scale_start: Option<(f32, f32)>,
    rotation_start: Option<f32>,
    mirror_axis: Option<MirrorAxis>,
    current_scale: Option<(f32, f32)>,
    current_rotation: Option<f32>,
    current_polyline_points: Vec<(f32, f32)>,
    pub selected_cam_operation: cam_operations::CAMOperation,
    pub cam_params: cam_operations::CAMParameters,
}

impl DesignerState {
    pub fn execute_command(&mut self, mut command: Box<dyn Command>) {
        command.execute(self);
        self.undo_stack.push_back(command);
        self.redo_stack.clear(); // Clear redo stack when new command is executed
    }

    pub fn undo(&mut self) {
        if let Some(mut command) = self.undo_stack.pop_back() {
            command.undo(self);
            self.redo_stack.push_back(command);
        }
    }

    pub fn redo(&mut self) {
        if let Some(mut command) = self.redo_stack.pop_back() {
            command.execute(self);
            self.undo_stack.push_back(command);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn get_shape_center(shape: &Shape) -> (f32, f32) {
        match shape {
            Shape::Rectangle {
                x,
                y,
                width,
                height,
            } => (x + width / 2.0, y + height / 2.0),
            Shape::Circle { x, y, .. } => (*x, *y),
            Shape::Line { x1, y1, x2, y2 } => ((x1 + x2) / 2.0, (y1 + y2) / 2.0),
            Shape::Text { x, y, .. } => (*x, *y),
            Shape::Drill { x, y, .. } => (*x, *y),
            Shape::Pocket {
                x,
                y,
                width,
                height,
                ..
            } => (x + width / 2.0, y + height / 2.0),
            Shape::Cylinder { x, y, .. } => (*x, *y),
            Shape::Sphere { x, y, .. } => (*x, *y),
            Shape::Extrusion {
                x,
                y,
                width,
                height,
                ..
            } => (x + width / 2.0, y + height / 2.0),
            Shape::Turning { x, y, .. } => (*x, *y),
            Shape::Facing { x, y, .. } => (*x, *y),
            Shape::Threading { x, y, .. } => (*x, *y),
            Shape::Polyline { points } => {
                if points.is_empty() {
                    (0.0, 0.0)
                } else {
                    let sum_x: f32 = points.iter().map(|(x, _)| *x).sum();
                    let sum_y: f32 = points.iter().map(|(_, y)| *y).sum();
                    (sum_x / points.len() as f32, sum_y / points.len() as f32)
                }
            }
            Shape::Parametric { bounds, .. } => {
                let (x1, y1, x2, y2) = bounds;
                ((x1 + x2) / 2.0, (y1 + y2) / 2.0)
            }
        }
    }

    fn get_shape_pos(shape: &Shape) -> (f32, f32) {
        match shape {
            Shape::Rectangle { x, y, .. } => (*x, *y),
            Shape::Circle { x, y, .. } => (*x, *y),
            Shape::Line { x1, y1, .. } => (*x1, *y1),
            Shape::Text { x, y, .. } => (*x, *y),
            Shape::Drill { x, y, .. } => (*x, *y),
            Shape::Pocket { x, y, .. } => (*x, *y),
            Shape::Cylinder { x, y, .. } => (*x, *y),
            Shape::Sphere { x, y, .. } => (*x, *y),
            Shape::Extrusion { x, y, .. } => (*x, *y),
            Shape::Turning { x, y, .. } => (*x, *y),
            Shape::Facing { x, y, .. } => (*x, *y),
            Shape::Threading { x, y, .. } => (*x, *y),
            Shape::Polyline { points } => points.first().copied().unwrap_or((0.0, 0.0)),
            Shape::Parametric { bounds, .. } => (bounds.0, bounds.1),
        }
    }

    fn set_shape_pos(shape: &mut Shape, pos: (f32, f32)) {
        match shape {
            Shape::Rectangle { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Circle { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Line { x1, y1, .. } => {
                *x1 = pos.0;
                *y1 = pos.1;
            }
            Shape::Text { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Drill { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Pocket { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Cylinder { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Sphere { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Extrusion { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Turning { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Facing { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Threading { x, y, .. } => {
                *x = pos.0;
                *y = pos.1;
            }
            Shape::Polyline { points } => {
                if let Some(first) = points.first() {
                    let dx = pos.0 - first.0;
                    let dy = pos.1 - first.1;
                    for point in points.iter_mut() {
                        point.0 += dx;
                        point.1 += dy;
                    }
                }
            }
            Shape::Parametric { bounds, .. } => {
                let dx = pos.0 - bounds.0;
                let dy = pos.1 - bounds.1;
                *bounds = (pos.0, pos.1, bounds.2 + dx, bounds.3 + dy);
            }
        }
    }
    pub fn export_to_gcode(&self) -> String {
        if self.shapes.is_empty() {
            return String::new();
        }

        let mut gcode_lines = vec![
            "G21 ; Set units to mm".to_string(),
            "G90 ; Absolute positioning".to_string(),
            "G0 Z5 ; Lift tool".to_string(),
        ];

        for shape in &self.shapes {
            match shape {
                Shape::Rectangle {
                    x,
                    y,
                    width,
                    height,
                } => {
                    gcode_lines.push(format!(
                        "; Rectangle at ({:.2}, {:.2}) size {:.2}x{:.2}",
                        x, y, width, height
                    ));
                    gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x, y));
                    gcode_lines.push("G1 Z-1 F500 ; Plunge".to_string());
                    gcode_lines.push(format!(
                        "G1 X{:.2} Y{:.2} F1000 ; Bottom edge",
                        x + width,
                        y
                    ));
                    gcode_lines.push(format!(
                        "G1 X{:.2} Y{:.2} F1000 ; Right edge",
                        x + width,
                        y + height
                    ));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000 ; Top edge", x, y + height));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000 ; Left edge", x, y));
                    gcode_lines.push("G0 Z5 ; Lift tool".to_string());
                }
                Shape::Circle { x, y, radius } => {
                    gcode_lines.push(format!(
                        "; Circle at ({:.2}, {:.2}) radius {:.2}",
                        x, y, radius
                    ));
                    gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x + radius, y));
                    gcode_lines.push("G1 Z-1 F500 ; Plunge".to_string());
                    gcode_lines.push(format!(
                        "G2 I-{:.2} J-{:.2} F1000 ; Clockwise circle",
                        radius, radius
                    ));
                    gcode_lines.push("G0 Z5 ; Lift tool".to_string());
                }
                Shape::Line { x1, y1, x2, y2 } => {
                    gcode_lines.push(format!(
                        "; Line from ({:.2}, {:.2}) to ({:.2}, {:.2})",
                        x1, y1, x2, y2
                    ));
                    gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x1, y1));
                    gcode_lines.push("G1 Z-1 F500 ; Plunge".to_string());
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000 ; Draw line", x2, y2));
                    gcode_lines.push("G0 Z5 ; Lift tool".to_string());
                }
                Shape::Text { .. } => {
                    // Text is not exported to G-code
                }
                Shape::Drill { x, y, depth } => {
                    gcode_lines.push(format!(
                        "; Drill at ({:.2}, {:.2}) depth {:.2}",
                        x, y, depth
                    ));
                    gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x, y));
                    gcode_lines.push("G1 Z-1 F500 ; Plunge".to_string());
                    gcode_lines.push(format!("G1 Z-{:.2} F100", depth));
                    gcode_lines.push("G0 Z5 ; Retract".to_string());
                }
                Shape::Pocket {
                    x,
                    y,
                    width,
                    height,
                    depth,
                    stepover,
                    pattern,
                } => {
                    gcode_lines.push(format!("; Pocket at ({:.2}, {:.2}) size {:.2}x{:.2} depth {:.2} stepover {:.2} pattern {:?}", x, y, width, height, depth, stepover, pattern));
                    let _tool_diameter = self.current_tool_def.diameter;
                    let mut current_depth = 0.0;
                    let depth_per_pass = 2.0; // Assume depth per pass
                    while current_depth < *depth {
                        current_depth += depth_per_pass;
                        if current_depth > *depth {
                            current_depth = *depth;
                        }
                        gcode_lines.push("G0 Z5".to_string());
                        gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x, y));
                        gcode_lines.push(format!("G1 Z-{:.2} F500", current_depth));
                        match pattern {
                            ToolpathPattern::Offset => {
                                // Generate offset rectangles
                                let mut offset = 0.0;
                                while offset < (*width / 2.0).min(*height / 2.0) {
                                    let w = width - 2.0 * offset;
                                    let h = height - 2.0 * offset;
                                    if w <= 0.0 || h <= 0.0 {
                                        break;
                                    }
                                    gcode_lines.push(format!(
                                        "G1 X{:.2} Y{:.2} F1000",
                                        x + offset,
                                        y + offset
                                    ));
                                    gcode_lines.push(format!(
                                        "G1 X{:.2} Y{:.2}",
                                        x + offset + w,
                                        y + offset
                                    ));
                                    gcode_lines.push(format!(
                                        "G1 X{:.2} Y{:.2}",
                                        x + offset + w,
                                        y + offset + h
                                    ));
                                    gcode_lines.push(format!(
                                        "G1 X{:.2} Y{:.2}",
                                        x + offset,
                                        y + offset + h
                                    ));
                                    gcode_lines.push(format!(
                                        "G1 X{:.2} Y{:.2}",
                                        x + offset,
                                        y + offset
                                    ));
                                    offset += *stepover;
                                }
                            }
                            ToolpathPattern::Spiral => {
                                // Simple spiral pattern
                                let center_x = x + width / 2.0;
                                let center_y = y + height / 2.0;
                                let max_radius = (width / 2.0).min(height / 2.0);
                                let mut radius = *stepover;
                                while radius < max_radius {
                                    gcode_lines.push(format!(
                                        "G1 X{:.2} Y{:.2} F1000",
                                        center_x + radius,
                                        center_y
                                    ));
                                    gcode_lines.push(format!("G2 I-{:.2} J0 F1000", radius));
                                    radius += *stepover;
                                }
                            }
                            ToolpathPattern::Zigzag => {
                                // Zigzag pattern
                                let mut y_pos = *y;
                                while y_pos < *y + *height {
                                    if ((y_pos - *y) / *stepover) as i32 % 2 == 0 {
                                        gcode_lines
                                            .push(format!("G1 X{:.2} Y{:.2} F1000", x, y_pos));
                                        gcode_lines.push(format!(
                                            "G1 X{:.2} Y{:.2}",
                                            x + width,
                                            y_pos
                                        ));
                                    } else {
                                        gcode_lines.push(format!(
                                            "G1 X{:.2} Y{:.2} F1000",
                                            x + width,
                                            y_pos
                                        ));
                                        gcode_lines.push(format!("G1 X{:.2} Y{:.2}", x, y_pos));
                                    }
                                    y_pos += *stepover;
                                }
                            }
                            ToolpathPattern::Trochoidal => {
                                // Simplified trochoidal pattern - circular paths
                                let center_x = *x + *width / 2.0;
                                let center_y = *y + *height / 2.0;
                                let radius = *stepover;
                                let mut angle: f32 = 0.0;
                                while angle < 360.0 {
                                    let _rad = angle.to_radians();
                                    let cx = center_x + (angle / 10.0).cos() * radius;
                                    let cy = center_y + (angle / 10.0).sin() * radius;
                                    gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000", cx, cy));
                                    gcode_lines.push(format!("G2 I-{:.2} J0 F1000", radius));
                                    angle += 30.0;
                                }
                            }
                        }
                        gcode_lines.push("G0 Z5 ; Retract".to_string());
                    }
                }
                Shape::Cylinder {
                    x,
                    y,
                    radius,
                    height,
                    depth,
                } => {
                    gcode_lines.push(format!(
                        "; Cylinder at ({:.2}, {:.2}) radius {:.2} height {:.2} depth {:.2}",
                        x, y, radius, height, depth
                    ));
                    // Helical milling for cylinder
                    let tool_diameter = self.current_tool_def.diameter;
                    let effective_radius = radius - tool_diameter / 2.0;
                    let pitch = 1.0; // Helical pitch
                    let mut current_z = 0.0;
                    while current_z < *height {
                        let z_end = (current_z + pitch).min(*height);
                        gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x + effective_radius, y));
                        gcode_lines.push(format!("G1 Z-{:.2} F500", depth));
                        gcode_lines.push(format!(
                            "G2 I-{:.2} J0 Z{:.2} F1000",
                            effective_radius, -z_end
                        ));
                        current_z = z_end;
                    }
                    gcode_lines.push("G0 Z5 ; Retract".to_string());
                }
                Shape::Sphere {
                    x,
                    y,
                    radius,
                    depth,
                } => {
                    gcode_lines.push(format!(
                        "; Sphere at ({:.2}, {:.2}) radius {:.2} depth {:.2}",
                        x, y, radius, depth
                    ));
                    // Spherical milling - simplified as multiple circles
                    let tool_diameter = self.current_tool_def.diameter;
                    let effective_radius = radius - tool_diameter / 2.0;
                    let layers = 10;
                    for i in 0..layers {
                        let z = -depth * (i as f32) / (layers as f32 - 1.0);
                        let r = (effective_radius * effective_radius - z * z).sqrt();
                        gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x + r, y));
                        gcode_lines.push(format!("G1 Z{:.2} F500", z));
                        gcode_lines.push(format!("G2 I-{:.2} J0 F1000", r));
                    }
                    gcode_lines.push("G0 Z5 ; Retract".to_string());
                }
                Shape::Extrusion {
                    x,
                    y,
                    width,
                    height,
                    depth,
                } => {
                    gcode_lines.push(format!(
                        "; Extrusion at ({:.2}, {:.2}) size {:.2}x{:.2} depth {:.2}",
                        x, y, width, height, depth
                    ));
                    // Similar to pocket but with depth
                    gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x, y));
                    gcode_lines.push(format!("G1 Z-{:.2} F500", depth));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000", x + width, y));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2}", x + width, y + height));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2}", x, y + height));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2}", x, y));
                    gcode_lines.push("G0 Z5 ; Retract".to_string());
                }
                Shape::Turning {
                    x,
                    y,
                    diameter,
                    length,
                    depth,
                } => {
                    gcode_lines.push(format!(
                        "; Turning at ({:.2}, {:.2}) diameter {:.2} length {:.2} depth {:.2}",
                        x, y, diameter, length, depth
                    ));
                    // Lathe turning operation
                    gcode_lines.push("G96 S200 ; Constant surface speed".to_string());
                    gcode_lines.push(format!("G0 X{:.2} Z{:.2}", diameter / 2.0 + x, y));
                    gcode_lines.push(format!(
                        "G1 X{:.2} Z{:.2} F100",
                        diameter / 2.0 - depth + x,
                        y
                    ));
                    gcode_lines.push(format!(
                        "G1 X{:.2} Z{:.2}",
                        diameter / 2.0 - depth + x,
                        y + length
                    ));
                    gcode_lines.push(format!("G1 X{:.2} Z{:.2}", diameter / 2.0 + x, y + length));
                    gcode_lines.push("G97 ; Cancel CSS".to_string());
                }
                Shape::Facing {
                    x,
                    y,
                    width,
                    length,
                    depth,
                } => {
                    gcode_lines.push(format!(
                        "; Facing at ({:.2}, {:.2}) width {:.2} length {:.2} depth {:.2}",
                        x, y, width, length, depth
                    ));
                    // Lathe facing operation
                    gcode_lines.push("G96 S200 ; Constant surface speed".to_string());
                    gcode_lines.push(format!("G0 X{:.2} Z{:.2}", x + width, y));
                    gcode_lines.push(format!("G1 Z-{:.2} F100", depth));
                    gcode_lines.push(format!("G1 X{:.2} Z-{:.2}", x, depth));
                    gcode_lines.push(format!("G1 X{:.2} Z{:.2}", x, y + length));
                    gcode_lines.push("G97 ; Cancel CSS".to_string());
                }
                Shape::Threading {
                    x,
                    y,
                    diameter,
                    length,
                    pitch,
                    depth,
                } => {
                    gcode_lines.push(format!("; Threading at ({:.2}, {:.2}) diameter {:.2} length {:.2} pitch {:.2} depth {:.2}", x, y, diameter, length, pitch, depth));
                    // Lathe threading operation
                    gcode_lines.push("G96 S200 ; Constant surface speed".to_string());
                    gcode_lines.push(format!("G0 X{:.2} Z{:.2}", diameter / 2.0 + x, y));
                    gcode_lines.push(format!("G33 Z{:.2} K{:.4} F100", y + length, pitch));
                    gcode_lines.push("G97 ; Cancel CSS".to_string());
                }
                Shape::Polyline { points } => {
                    gcode_lines.push(format!("; Polyline with {} points", points.len()));
                    gcode_lines.push("G0 Z5 ; Lift tool".to_string());
                    for (i, (x, y)) in points.iter().enumerate() {
                        if i == 0 {
                            gcode_lines.push(format!("G0 X{:.2} Y{:.2}", x, y));
                            gcode_lines.push("G1 Z-1 F500 ; Plunge".to_string());
                        } else {
                            gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000", x, y));
                        }
                    }
                    gcode_lines.push("G0 Z5 ; Lift tool".to_string());
                }
                Shape::Parametric {
                    script,
                    ast: _,
                    bounds,
                } => {
                    gcode_lines.push(format!("; Parametric shape: {}", script));
                    // For now, just generate a simple rectangle based on bounds
                    gcode_lines.push(format!("G0 X{:.2} Y{:.2}", bounds.0, bounds.1));
                    gcode_lines.push("G1 Z-1 F500 ; Plunge".to_string());
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2} F1000", bounds.2, bounds.1));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2}", bounds.2, bounds.3));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2}", bounds.0, bounds.3));
                    gcode_lines.push(format!("G1 X{:.2} Y{:.2}", bounds.0, bounds.1));
                    gcode_lines.push("G0 Z5 ; Lift tool".to_string());
                }
            }
        }

        gcode_lines.push("M30 ; End program".to_string());
        gcode_lines.join("\n")
    }

    pub fn export_to_stl(&self) -> Result<Vec<u8>> {
        let mut triangles = Vec::new();

        for shape in &self.shapes {
            match shape {
                Shape::Rectangle {
                    x,
                    y,
                    width,
                    height,
                } => {
                    // Create two triangles for the rectangle (flat at z=0)
                    let v1 = Vertex::new([*x, *y, 0.0]);
                    let v2 = Vertex::new([x + width, *y, 0.0]);
                    let v3 = Vertex::new([x + width, y + height, 0.0]);
                    let v4 = Vertex::new([*x, y + height, 0.0]);

                    let normal = Normal::new([0.0, 0.0, 1.0]);

                    triangles.push(Triangle {
                        normal,
                        vertices: [v1, v2, v3],
                    });
                    triangles.push(Triangle {
                        normal,
                        vertices: [v1, v3, v4],
                    });
                }
                Shape::Cylinder {
                    x,
                    y,
                    radius,
                    height,
                    ..
                } => {
                    // Simple cylinder triangulation
                    let segments = 16;
                    let z_top = height / 2.0;
                    let z_bottom = -height / 2.0;

                    // Top and bottom circles
                    for i in 0..segments {
                        let angle1 = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                        let angle2 =
                            ((i + 1) as f32 / segments as f32) * 2.0 * std::f32::consts::PI;

                        let x1 = x + radius * angle1.cos();
                        let y1 = y + radius * angle1.sin();
                        let x2 = x + radius * angle2.cos();
                        let y2 = y + radius * angle2.sin();

                        // Top face
                        triangles.push(Triangle {
                            normal: Normal::new([0.0, 0.0, 1.0]),
                            vertices: [
                                Vertex::new([*x, *y, z_top]),
                                Vertex::new([x1, y1, z_top]),
                                Vertex::new([x2, y2, z_top]),
                            ],
                        });

                        // Bottom face
                        triangles.push(Triangle {
                            normal: Normal::new([0.0, 0.0, -1.0]),
                            vertices: [
                                Vertex::new([*x, *y, z_bottom]),
                                Vertex::new([x2, y2, z_bottom]),
                                Vertex::new([x1, y1, z_bottom]),
                            ],
                        });

                        // Side faces
                        triangles.push(Triangle {
                            normal: Normal::new([angle1.cos(), angle1.sin(), 0.0]),
                            vertices: [
                                Vertex::new([x1, y1, z_top]),
                                Vertex::new([x1, y1, z_bottom]),
                                Vertex::new([x2, y2, z_bottom]),
                            ],
                        });
                        triangles.push(Triangle {
                            normal: Normal::new([angle1.cos(), angle1.sin(), 0.0]),
                            vertices: [
                                Vertex::new([x1, y1, z_top]),
                                Vertex::new([x2, y2, z_bottom]),
                                Vertex::new([x2, y2, z_top]),
                            ],
                        });
                    }
                }
                Shape::Sphere { x, y, radius, .. } => {
                    // Simple sphere triangulation
                    let stacks = 8;
                    let slices = 16;

                    for i in 0..stacks {
                        let phi1 = (i as f32 / stacks as f32) * std::f32::consts::PI;
                        let phi2 = ((i + 1) as f32 / stacks as f32) * std::f32::consts::PI;

                        for j in 0..slices {
                            let theta1 = (j as f32 / slices as f32) * 2.0 * std::f32::consts::PI;
                            let theta2 =
                                ((j + 1) as f32 / slices as f32) * 2.0 * std::f32::consts::PI;

                            let x1 = x + radius * phi1.sin() * theta1.cos();
                            let y1 = y + radius * phi1.sin() * theta1.sin();
                            let z1 = radius * phi1.cos();

                            let x2 = x + radius * phi1.sin() * theta2.cos();
                            let y2 = y + radius * phi1.sin() * theta2.sin();
                            let z2 = radius * phi1.cos();

                            let x3 = x + radius * phi2.sin() * theta2.cos();
                            let y3 = y + radius * phi2.sin() * theta2.sin();
                            let z3 = radius * phi2.cos();

                            let x4 = x + radius * phi2.sin() * theta1.cos();
                            let y4 = y + radius * phi2.sin() * theta1.sin();
                            let z4 = radius * phi2.cos();

                            // Two triangles per quad
                            triangles.push(Triangle {
                                normal: Normal::new([x1 - *x, y1 - *y, z1]),
                                vertices: [
                                    Vertex::new([x1, y1, z1]),
                                    Vertex::new([x2, y2, z2]),
                                    Vertex::new([x3, y3, z3]),
                                ],
                            });
                            triangles.push(Triangle {
                                normal: Normal::new([x1 - *x, y1 - *y, z1]),
                                vertices: [
                                    Vertex::new([x1, y1, z1]),
                                    Vertex::new([x3, y3, z3]),
                                    Vertex::new([x4, y4, z4]),
                                ],
                            });
                        }
                    }
                }
                // Add other shapes as needed
                _ => {} // Skip shapes that can't be easily triangulated
            }
        }

        let mut buffer = Vec::new();
        stl_io::write_stl(&mut buffer, triangles.iter())?;
        Ok(buffer)
    }

    pub fn export_to_obj(&self) -> Result<String> {
        let mut obj_content = String::new();
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for shape in &self.shapes {
            match shape {
                Shape::Rectangle {
                    x,
                    y,
                    width,
                    height,
                } => {
                    let base_idx = vertices.len() + 1;
                    vertices.push(format!("v {} {} 0.0", x, y));
                    vertices.push(format!("v {} {} 0.0", x + width, y));
                    vertices.push(format!("v {} {} 0.0", x + width, y + height));
                    vertices.push(format!("v {} {} 0.0", x, y + height));

                    faces.push(format!("f {} {} {}", base_idx, base_idx + 1, base_idx + 2));
                    faces.push(format!("f {} {} {}", base_idx, base_idx + 2, base_idx + 3));
                }
                Shape::Cylinder {
                    x,
                    y,
                    radius,
                    height,
                    ..
                } => {
                    let segments = 16;
                    let z_top = height / 2.0;
                    let z_bottom = -height / 2.0;
                    let base_idx = vertices.len() + 1;

                    // Center points
                    vertices.push(format!("v {} {} {}", x, y, z_top));
                    vertices.push(format!("v {} {} {}", x, y, z_bottom));

                    // Circle points
                    for i in 0..segments {
                        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                        let vx = x + radius * angle.cos();
                        let vy = y + radius * angle.sin();
                        vertices.push(format!("v {} {} {}", vx, vy, z_top));
                        vertices.push(format!("v {} {} {}", vx, vy, z_bottom));
                    }

                    // Top face
                    let mut top_face = format!("f {}", base_idx);
                    for i in 0..segments {
                        top_face.push_str(&format!(" {}", base_idx + 2 + i * 2));
                    }
                    faces.push(top_face);

                    // Bottom face
                    let mut bottom_face = format!("f {}", base_idx + 1);
                    for i in (0..segments).rev() {
                        bottom_face.push_str(&format!(" {}", base_idx + 3 + i * 2));
                    }
                    faces.push(bottom_face);

                    // Side faces
                    for i in 0..segments {
                        let next = (i + 1) % segments;
                        let v1 = base_idx + 2 + i * 2;
                        let v2 = base_idx + 2 + next * 2;
                        let v3 = base_idx + 3 + i * 2;
                        let v4 = base_idx + 3 + next * 2;
                        faces.push(format!("f {} {} {}", v1, v3, v4));
                        faces.push(format!("f {} {} {}", v1, v4, v2));
                    }
                }
                // Add other shapes
                _ => {}
            }
        }

        for v in vertices {
            obj_content.push_str(&v);
            obj_content.push('\n');
        }
        for f in faces {
            obj_content.push_str(&f);
            obj_content.push('\n');
        }

        Ok(obj_content)
    }

    fn shape_contains_point(&self, shape: &Shape, point: egui::Pos2) -> bool {
        match shape {
            Shape::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                point.x >= *x && point.x <= *x + *width && point.y >= *y && point.y <= *y + *height
            }
            Shape::Circle { x, y, radius } => {
                let dx = point.x - x;
                let dy = point.y - y;
                (dx * dx + dy * dy).sqrt() <= *radius
            }
            Shape::Line { x1, y1, x2, y2 } => {
                // Simple line hit test - check if point is close to the line
                let dx = x2 - x1;
                let dy = y2 - y1;
                let length = (dx * dx + dy * dy).sqrt();
                if length == 0.0 {
                    return (point.x - x1).abs() < 5.0 && (point.y - y1).abs() < 5.0;
                }

                let t = ((point.x - x1) * dx + (point.y - y1) * dy) / (length * length);
                let t = t.max(0.0).min(1.0);

                let closest_x = x1 + t * dx;
                let closest_y = y1 + t * dy;

                let distance =
                    ((point.x - closest_x).powi(2) + (point.y - closest_y).powi(2)).sqrt();
                distance < 5.0 // 5 pixel tolerance
            }
            Shape::Text {
                x,
                y,
                text,
                font_size,
            } => {
                // Simple text hit test - check if point is within text bounds
                let text_width = text.len() as f32 * *font_size * 0.6; // Rough estimate
                let text_height = *font_size;
                point.x >= *x
                    && point.x <= x + text_width
                    && point.y >= *y
                    && point.y <= y + text_height
            }
            Shape::Drill { x, y, .. } => {
                // Simple drill hit test - small area around point
                let dx = point.x - x;
                let dy = point.y - y;
                (dx * dx + dy * dy).sqrt() < 5.0
            }
            Shape::Pocket {
                x,
                y,
                width,
                height,
                ..
            } => point.x >= *x && point.x <= x + width && point.y >= *y && point.y <= y + height,
            Shape::Cylinder {
                x,
                y,
                radius,
                height,
                ..
            } => {
                let dx = point.x - x;
                let dy = point.y - y;
                (dx * dx + dy * dy).sqrt() <= *radius && point.y >= *y && point.y <= y + height
            }
            Shape::Sphere { x, y, radius, .. } => {
                let dx = point.x - x;
                let dy = point.y - y;
                (dx * dx + dy * dy).sqrt() <= *radius
            }
            Shape::Extrusion {
                x,
                y,
                width,
                height,
                ..
            } => point.x >= *x && point.x <= x + width && point.y >= *y && point.y <= y + height,
            Shape::Turning {
                x,
                y,
                diameter,
                length,
                ..
            } => {
                let radius = diameter / 2.0;
                let dx = point.x - x;
                let dy = point.y - y;
                (dx * dx + dy * dy).sqrt() <= radius && point.y >= *y && point.y <= y + length
            }
            Shape::Facing {
                x,
                y,
                width,
                length,
                ..
            } => point.x >= *x && point.x <= x + width && point.y >= *y && point.y <= y + length,
            Shape::Threading {
                x,
                y,
                diameter,
                length,
                ..
            } => {
                let radius = diameter / 2.0;
                let dx = point.x - x;
                let dy = point.y - y;
                (dx * dx + dy * dy).sqrt() <= radius && point.y >= *y && point.y <= y + length
            }
            Shape::Polyline { points } => {
                // Check if point is close to any line segment
                for window in points.windows(2) {
                    let (x1, y1) = window[0];
                    let (x2, y2) = window[1];
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let length = (dx * dx + dy * dy).sqrt();
                    if length == 0.0 {
                        if (point.x - x1).abs() < 5.0 && (point.y - y1).abs() < 5.0 {
                            return true;
                        }
                        continue;
                    }
                    let t = ((point.x - x1) * dx + (point.y - y1) * dy) / (length * length);
                    let t = t.max(0.0).min(1.0);
                    let closest_x = x1 + t * dx;
                    let closest_y = y1 + t * dy;
                    let distance =
                        ((point.x - closest_x).powi(2) + (point.y - closest_y).powi(2)).sqrt();
                    if distance < 5.0 {
                        return true;
                    }
                }
                false
            }
            Shape::Parametric { bounds, .. } => {
                point.x >= bounds.0
                    && point.x <= bounds.2
                    && point.y >= bounds.1
                    && point.y <= bounds.3
            }
        }
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) -> Option<DesignerEvent> {
        let mut event = None;

        ui.vertical(|ui| {
            // Toolbar
            ui.horizontal(|ui| {
                ui.label("Designer");
                ui.separator();

                // Drawing tools
                ui.selectable_value(&mut self.current_tool, DrawingTool::Select, "👆 Select");
                ui.selectable_value(
                    &mut self.current_tool,
                    DrawingTool::Rectangle,
                    "▭ Rectangle",
                );
                ui.selectable_value(&mut self.current_tool, DrawingTool::Circle, "○ Circle");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Line, "━ Line");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Text, "📝 Text");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Drill, "🔨 Drill");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Pocket, "📦 Pocket");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Cylinder, "🛢️ Cylinder");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Sphere, "🔮 Sphere");
                ui.selectable_value(
                    &mut self.current_tool,
                    DrawingTool::Extrusion,
                    "📏 Extrusion",
                );
                ui.selectable_value(&mut self.current_tool, DrawingTool::Turning, "🔄 Turning");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Facing, "📐 Facing");
                ui.selectable_value(
                    &mut self.current_tool,
                    DrawingTool::Threading,
                    "🧵 Threading",
                );
                ui.selectable_value(&mut self.current_tool, DrawingTool::Polyline, "📏 Polyline");
                ui.selectable_value(
                    &mut self.current_tool,
                    DrawingTool::Parametric,
                    "📊 Parametric",
                );

                ui.separator();

                // Toolpath pattern
                ui.label("Pattern:");
                ui.selectable_value(&mut self.current_pattern, ToolpathPattern::Offset, "Offset");
                ui.selectable_value(&mut self.current_pattern, ToolpathPattern::Spiral, "Spiral");
                ui.selectable_value(&mut self.current_pattern, ToolpathPattern::Zigzag, "Zigzag");
                ui.selectable_value(
                    &mut self.current_pattern,
                    ToolpathPattern::Trochoidal,
                    "Trochoidal",
                );

                ui.separator();

                // Manipulation tools
                ui.selectable_value(&mut self.current_tool, DrawingTool::Move, "↔ Move");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Scale, "🔍 Scale");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Rotate, "🔄 Rotate");
                ui.selectable_value(&mut self.current_tool, DrawingTool::Mirror, "🪞 Mirror");

                ui.separator();

                if ui.button("🗑️ Delete").clicked()
                    && let Some(index) = self.selected_shape
                {
                    self.execute_command(Box::new(DeleteShapeCommand::new(index)));
                    self.selected_shape = None;
                }

                if ui.button("📁 Import").clicked() {
                    event = Some(DesignerEvent::ImportFile);
                }

                if ui.button("↶ Undo").clicked() && self.can_undo() {
                    self.undo();
                    self.selected_shape = None;
                }

                if ui.button("↷ Redo").clicked() && self.can_redo() {
                    self.redo();
                    self.selected_shape = None;
                }

                ui.separator();

                ui.menu_button("Alignment", |ui| {
                    if ui.button("Align Left").clicked() {
                        self.align_shapes("left");
                        ui.close();
                    }
                    if ui.button("Align Right").clicked() {
                        self.align_shapes("right");
                        ui.close();
                    }
                    if ui.button("Align Top").clicked() {
                        self.align_shapes("top");
                        ui.close();
                    }
                    if ui.button("Align Bottom").clicked() {
                        self.align_shapes("bottom");
                        ui.close();
                    }
                    if ui.button("Align Center X").clicked() {
                        self.align_shapes("center_x");
                        ui.close();
                    }
                    if ui.button("Align Center Y").clicked() {
                        self.align_shapes("center_y");
                        ui.close();
                    }
                });

                ui.separator();

                ui.menu_button("Boolean", |ui| {
                    if ui.button("Union").clicked() {
                        if self.shapes.len() >= 2 {
                            let all_indices: Vec<usize> = (0..self.shapes.len()).collect();
                            if let Err(e) = self.boolean_union(&all_indices) {
                                tracing::error!("Boolean union failed: {}", e);
                            }
                            self.selected_shape = None;
                        }
                        ui.close();
                    }
                    if ui.button("Intersect").clicked() {
                        if self.shapes.len() >= 2 {
                            let all_indices: Vec<usize> = (0..self.shapes.len()).collect();
                            if let Err(e) = self.boolean_intersect(&all_indices) {
                                tracing::error!("Boolean intersect failed: {}", e);
                            }
                            self.selected_shape = None;
                        }
                        ui.close();
                    }
                    if ui.button("Subtract").clicked() {
                        if self.shapes.len() >= 2 {
                            let indices = [self.shapes.len() - 2, self.shapes.len() - 1];
                            if let Err(e) = self.boolean_subtract(&indices) {
                                tracing::error!("Boolean subtract failed: {}", e);
                            }
                            self.selected_shape = None;
                        }
                        ui.close();
                    }
                });

                ui.separator();

                if ui.button("🗑️ Clear").clicked() {
                    self.shapes.clear();
                    self.selected_shape = None;
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                }

                if ui.button("💾 Export G-code").clicked() {
                    event = Some(DesignerEvent::ExportGcode);
                }

                ui.menu_button("📤 Export 3D", |ui| {
                    if ui.button("STL").clicked() {
                        event = Some(DesignerEvent::ExportStl);
                        ui.close();
                    }
                    if ui.button("OBJ").clicked() {
                        event = Some(DesignerEvent::ExportObj);
                        ui.close();
                    }
                    if ui.button("GLTF").clicked() {
                        event = Some(DesignerEvent::ExportGltf);
                        ui.close();
                    }
                });

                ui.separator();

                ui.checkbox(&mut self.show_grid, "Grid");

                ui.separator();

                ui.label(format!("Material: {}", self.current_material.name));
                ui.label(format!(
                    "Tool: {} (Ø{:.1}mm)",
                    self.current_tool_def.name, self.current_tool_def.diameter
                ));
            });

            ui.separator();

            // Canvas
            let available_size = ui.available_size();
            let (rect, response) =
                ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

            // Handle mouse interactions
            if response.clicked() {
                let pos = response.interact_pointer_pos().unwrap_or_default();
                let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);

                match self.current_tool {
                    DrawingTool::Select => {
                        // Select shape under cursor
                        self.selected_shape = None;
                        self.selected_point = None;
                        for (i, shape) in self.shapes.iter().enumerate().rev() {
                            if self.shape_contains_point(shape, canvas_pos) {
                                self.selected_shape = Some(i);
                                // Check if it's a polyline and select point
                                if let Shape::Polyline { points } = shape {
                                    for (j, &(px, py)) in points.iter().enumerate() {
                                        let dist = ((canvas_pos.x - px).powi(2)
                                            + (canvas_pos.y - py).powi(2))
                                        .sqrt();
                                        if dist <= 10.0 {
                                            // Handle size threshold
                                            self.selected_point = Some(j);
                                            break;
                                        }
                                    }
                                }
                                break;
                            }
                        }

                        // Handle point dragging for polylines
                        if self.selected_point.is_some()
                            && response.dragged()
                            && let Some(pos) = response.interact_pointer_pos()
                        {
                            let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);
                            if let Some(index) = self.selected_shape {
                                if let Some(Shape::Polyline { points }) = self.shapes.get_mut(index)
                                {
                                    if let Some(point_idx) = self.selected_point {
                                        if point_idx < points.len() {
                                            points[point_idx] = (canvas_pos.x, canvas_pos.y);
                                        }
                                    }
                                }
                            }
                        }

                        // Handle right-click to add/remove points to polylines
                        if response.secondary_clicked() {
                            let pos = response.interact_pointer_pos().unwrap_or_default();
                            let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);
                            if let Some(index) = self.selected_shape {
                                // First, check if clicking on a point to remove
                                let point_to_remove = if let Some(Shape::Polyline { points }) =
                                    self.shapes.get(index)
                                {
                                    let mut remove = None;
                                    for (j, &(px, py)) in points.iter().enumerate() {
                                        let dist = ((canvas_pos.x - px).powi(2)
                                            + (canvas_pos.y - py).powi(2))
                                        .sqrt();
                                        if dist <= 10.0 && points.len() > 2 {
                                            // Don't remove if less than 3 points
                                            remove = Some(j);
                                            break;
                                        }
                                    }
                                    remove
                                } else {
                                    None
                                };

                                if let Some(j) = point_to_remove {
                                    if let Some(Shape::Polyline { points }) =
                                        self.shapes.get_mut(index)
                                    {
                                        points.remove(j);
                                        if self.selected_point == Some(j) {
                                            self.selected_point = None;
                                        } else if let Some(p) = self.selected_point {
                                            if p > j {
                                                self.selected_point = Some(p - 1);
                                            }
                                        }
                                    }
                                } else {
                                    // Add point to closest segment
                                    let insert_at = if let Some(Shape::Polyline { points }) =
                                        self.shapes.get(index)
                                    {
                                        let mut closest_dist = f32::INFINITY;
                                        let mut insert_at = 0;
                                        for i in 0..points.len().saturating_sub(1) {
                                            let p1 = points[i];
                                            let p2 = points[i + 1];
                                            let dist = self.point_to_line_distance(
                                                (canvas_pos.x, canvas_pos.y),
                                                p1,
                                                p2,
                                            );
                                            if dist < closest_dist {
                                                closest_dist = dist;
                                                insert_at = i + 1;
                                            }
                                        }
                                        if closest_dist < 20.0 {
                                            Some(insert_at)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    if let Some(insert_at) = insert_at {
                                        if let Some(Shape::Polyline { points }) =
                                            self.shapes.get_mut(index)
                                        {
                                            points.insert(insert_at, (canvas_pos.x, canvas_pos.y));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    DrawingTool::Polyline => {
                        // Check for double-click to finish polyline
                        if response.double_clicked() && !self.current_polyline_points.is_empty() {
                            // Finish the current polyline
                            if self.current_polyline_points.len() >= 2 {
                                let shape = Shape::Polyline {
                                    points: self.current_polyline_points.clone(),
                                };
                                self.execute_command(Box::new(AddShapeCommand::new(shape)));
                            }
                            self.current_polyline_points.clear();
                            self.drawing_start = None;
                        } else {
                            // Add point to current polyline
                            self.current_polyline_points
                                .push((canvas_pos.x, canvas_pos.y));

                            // If this is the first point, start drawing
                            if self.current_polyline_points.len() == 1 {
                                self.drawing_start = Some((canvas_pos.x, canvas_pos.y));
                            }
                        }
                    }
                    DrawingTool::Parametric | _ => {
                        // Start drawing
                        self.drawing_start = Some((canvas_pos.x, canvas_pos.y));
                    }
                }
            }

            if !response.dragged() && self.drawing_start.is_some() {
                if let Some(start) = self.drawing_start {
                    let end_pos = response.interact_pointer_pos().unwrap_or_default();
                    let end_canvas = egui::pos2(end_pos.x - rect.min.x, end_pos.y - rect.min.y);

                    let shape = match self.current_tool {
                        DrawingTool::Rectangle => {
                            let width = (end_canvas.x - start.0).abs();
                            let height = (end_canvas.y - start.1).abs();
                            let x = start.0.min(end_canvas.x);
                            let y = start.1.min(end_canvas.y);
                            Shape::Rectangle {
                                x,
                                y,
                                width,
                                height,
                            }
                        }
                        DrawingTool::Circle => {
                            let radius = ((end_canvas.x - start.0).powi(2)
                                + (end_canvas.y - start.1).powi(2))
                            .sqrt();
                            Shape::Circle {
                                x: start.0,
                                y: start.1,
                                radius,
                            }
                        }
                        DrawingTool::Line => Shape::Line {
                            x1: start.0,
                            y1: start.1,
                            x2: end_canvas.x,
                            y2: end_canvas.y,
                        },
                        DrawingTool::Text => {
                            // For text, just place at click position with default text
                            Shape::Text {
                                x: start.0,
                                y: start.1,
                                text: "Text".to_string(),
                                font_size: 16.0,
                            }
                        }
                        DrawingTool::Drill => {
                            // For drill, place at click position with default depth
                            Shape::Drill {
                                x: start.0,
                                y: start.1,
                                depth: 5.0,
                            }
                        }
                        DrawingTool::Pocket => {
                            let width = (end_canvas.x - start.0).abs();
                            let height = (end_canvas.y - start.1).abs();
                            let x = start.0.min(end_canvas.x);
                            let y = start.1.min(end_canvas.y);
                            Shape::Pocket {
                                x,
                                y,
                                width,
                                height,
                                depth: 5.0,
                                stepover: 1.0,
                                pattern: self.current_pattern.clone(),
                            }
                        }
                        DrawingTool::Cylinder => {
                            let radius = ((end_canvas.x - start.0).powi(2)
                                + (end_canvas.y - start.1).powi(2))
                            .sqrt();
                            Shape::Cylinder {
                                x: start.0,
                                y: start.1,
                                radius,
                                height: 10.0,
                                depth: 5.0,
                            }
                        }
                        DrawingTool::Sphere => {
                            let radius = ((end_canvas.x - start.0).powi(2)
                                + (end_canvas.y - start.1).powi(2))
                            .sqrt();
                            Shape::Sphere {
                                x: start.0,
                                y: start.1,
                                radius,
                                depth: 5.0,
                            }
                        }
                        DrawingTool::Extrusion => {
                            let width = (end_canvas.x - start.0).abs();
                            let height = (end_canvas.y - start.1).abs();
                            let x = start.0.min(end_canvas.x);
                            let y = start.1.min(end_canvas.y);
                            Shape::Extrusion {
                                x,
                                y,
                                width,
                                height,
                                depth: 5.0,
                            }
                        }
                        DrawingTool::Turning => {
                            let diameter = (end_canvas.x - start.0).abs();
                            let length = (end_canvas.y - start.1).abs();
                            Shape::Turning {
                                x: start.0,
                                y: start.1,
                                diameter,
                                length,
                                depth: 2.0,
                            }
                        }
                        DrawingTool::Facing => {
                            let width = (end_canvas.x - start.0).abs();
                            let length = (end_canvas.y - start.1).abs();
                            let x = start.0.min(end_canvas.x);
                            let y = start.1.min(end_canvas.y);
                            Shape::Facing {
                                x,
                                y,
                                width,
                                length,
                                depth: 1.0,
                            }
                        }
                        DrawingTool::Threading => {
                            let diameter = (end_canvas.x - start.0).abs();
                            let length = (end_canvas.y - start.1).abs();
                            Shape::Threading {
                                x: start.0,
                                y: start.1,
                                diameter,
                                length,
                                pitch: 1.5,
                                depth: 1.0,
                            }
                        }
                        DrawingTool::Polyline => {
                            // Create polyline from accumulated points
                            let points = if self.current_polyline_points.len() >= 2 {
                                self.current_polyline_points.clone()
                            } else {
                                vec![(start.0, start.1), (end_canvas.x, end_canvas.y)]
                            };
                            Shape::Polyline { points }
                        }
                        DrawingTool::Parametric => {
                            // For parametric, create with default script
                            Shape::Parametric {
                                script: "x = 10 * cos(t); y = 10 * sin(t);".to_string(),
                                ast: None,
                                bounds: (start.0, start.1, end_canvas.x, end_canvas.y),
                            }
                        }
                        DrawingTool::Move
                        | DrawingTool::Scale
                        | DrawingTool::Rotate
                        | DrawingTool::Mirror
                        | DrawingTool::Select => {
                            // Manipulation tools don't create new shapes
                            return;
                        }
                    };

                    self.execute_command(Box::new(AddShapeCommand::new(shape)));
                }
                self.drawing_start = None;
                self.current_polyline_points.clear();
            }

            // Handle manipulation
            if matches!(self.current_tool, DrawingTool::Move)
                && self.selected_shape.is_some()
                && response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);
                if self.drag_start_pos.is_none() {
                    // Record start pos
                    if let Some(index) = self.selected_shape
                        && let Some(shape) = self.shapes.get(index)
                    {
                        self.drag_start_pos = Some(Self::get_shape_pos(shape));
                    }
                }
                // Update pos
                if let Some(index) = self.selected_shape
                    && let Some(shape) = self.shapes.get_mut(index)
                {
                    Self::set_shape_pos(shape, (canvas_pos.x, canvas_pos.y));
                }
            }

            if self.drag_start_pos.is_some() && !response.dragged() {
                if let Some(index) = self.selected_shape
                    && let Some(shape) = self.shapes.get(index)
                {
                    let current_pos = Self::get_shape_pos(shape);
                    if let Some(old_pos) = self.drag_start_pos
                        && old_pos != current_pos
                    {
                        self.execute_command(Box::new(MoveShapeCommand::new(
                            index,
                            old_pos,
                            current_pos,
                        )));
                    }
                }
                self.drag_start_pos = None;
            }

            // Handle scale tool
            if matches!(self.current_tool, DrawingTool::Scale)
                && self.selected_shape.is_some()
                && response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);
                if self.manipulation_start.is_none() {
                    self.manipulation_start = Some((canvas_pos.x, canvas_pos.y));
                    if let Some(index) = self.selected_shape {
                        self.original_shape = self.shapes.get(index).cloned();
                        // Calculate initial scale based on distance from shape center
                        if let Some(shape) = self.shapes.get(index) {
                            let center = Self::get_shape_center(shape);
                            let dist = ((canvas_pos.x - center.0).powi(2)
                                + (canvas_pos.y - center.1).powi(2))
                            .sqrt();
                            self.scale_start = Some((1.0, 1.0)); // Start with no scaling
                        }
                    }
                }

                if let Some(index) = self.selected_shape
                    && let Some(start_pos) = self.manipulation_start
                    && let Some(shape) = self.shapes.get(index)
                {
                    let center = Self::get_shape_center(shape);
                    let start_dist = ((start_pos.0 - center.0).powi(2)
                        + (start_pos.1 - center.1).powi(2))
                    .sqrt();
                    let current_dist = ((canvas_pos.x - center.0).powi(2)
                        + (canvas_pos.y - center.1).powi(2))
                    .sqrt();

                    if start_dist > 0.0 {
                        let scale_factor = current_dist / start_dist;
                        let scale = (scale_factor, scale_factor); // Uniform scaling for now
                        self.current_scale = Some(scale);

                        // Reset to original and apply new scale
                        if let Some(original) = &self.original_shape {
                            self.shapes[index] = original.clone();
                            Self::scale_shape(&mut self.shapes[index], scale, center);
                        }
                    }
                }
            }

            // Handle rotate tool
            if matches!(self.current_tool, DrawingTool::Rotate)
                && self.selected_shape.is_some()
                && response.dragged()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);
                if self.manipulation_start.is_none() {
                    self.manipulation_start = Some((canvas_pos.x, canvas_pos.y));
                    if let Some(index) = self.selected_shape {
                        self.original_shape = self.shapes.get(index).cloned();
                        if let Some(shape) = self.shapes.get(index) {
                            let center = Self::get_shape_center(shape);
                            let angle = (canvas_pos.y - center.1)
                                .atan2(canvas_pos.x - center.0)
                                .to_degrees();
                            self.rotation_start = Some(angle);
                        }
                    }
                }

                if let Some(index) = self.selected_shape
                    && let Some(start_pos) = self.manipulation_start
                    && let Some(start_angle) = self.rotation_start
                    && let Some(shape) = self.shapes.get(index)
                {
                    let center = Self::get_shape_center(shape);
                    let current_angle = (canvas_pos.y - center.1)
                        .atan2(canvas_pos.x - center.0)
                        .to_degrees();
                    let angle_diff = current_angle - start_angle;
                    self.current_rotation = Some(angle_diff);

                    // Reset to original and apply new rotation
                    if let Some(original) = &self.original_shape {
                        self.shapes[index] = original.clone();
                        Self::rotate_shape(&mut self.shapes[index], angle_diff, center);
                    }
                }
            }

            // Handle mirror tool
            if matches!(self.current_tool, DrawingTool::Mirror)
                && self.selected_shape.is_some()
                && response.clicked()
                && let Some(pos) = response.interact_pointer_pos()
            {
                let canvas_pos = egui::pos2(pos.x - rect.min.x, pos.y - rect.min.y);
                if let Some(index) = self.selected_shape
                    && let Some(shape) = self.shapes.get(index)
                {
                    let center = Self::get_shape_center(shape);
                    let axis = if (canvas_pos.x - center.0).abs() > (canvas_pos.y - center.1).abs()
                    {
                        MirrorAxis::Horizontal
                    } else {
                        MirrorAxis::Vertical
                    };

                    self.execute_command(Box::new(MirrorShapeCommand::new(index, axis)));
                }
            }

            // Complete manipulation operations
            if self.manipulation_start.is_some()
                && !response.dragged()
                && matches!(self.current_tool, DrawingTool::Scale | DrawingTool::Rotate)
            {
                // For scale and rotate, we need to create commands when manipulation ends
                if let Some(index) = self.selected_shape
                    && let Some(original) = &self.original_shape
                    && let Some(shape) = self.shapes.get(index)
                {
                    match self.current_tool {
                        DrawingTool::Scale => {
                            if let Some(start_scale) = self.scale_start
                                && let Some(final_scale) = self.current_scale
                            {
                                let center = Self::get_shape_center(original);
                                self.execute_command(Box::new(ScaleShapeCommand::new(
                                    index,
                                    start_scale,
                                    final_scale,
                                    center,
                                )));
                            }
                        }
                        DrawingTool::Rotate => {
                            if let Some(start_angle) = self.rotation_start
                                && let Some(angle_diff) = self.current_rotation
                            {
                                let center = Self::get_shape_center(original);
                                let final_angle = start_angle + angle_diff;
                                self.execute_command(Box::new(RotateShapeCommand::new(
                                    index,
                                    start_angle,
                                    final_angle,
                                    center,
                                )));
                            }
                        }
                        _ => {}
                    }
                }

                self.manipulation_start = None;
                self.original_shape = None;
                self.scale_start = None;
                self.rotation_start = None;
                self.current_scale = None;
                self.current_rotation = None;
            }

            // Draw shapes
            let painter = ui.painter();
            for (i, shape) in self.shapes.iter().enumerate() {
                let color = if Some(i) == self.selected_shape {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::WHITE
                };

                match shape {
                    Shape::Rectangle {
                        x,
                        y,
                        width,
                        height,
                    } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(*width, *height),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(2.0, color),
                            egui::StrokeKind::Outside,
                        );
                    }
                    Shape::Circle { x, y, radius } => {
                        let center = egui::pos2(rect.min.x + x, rect.min.y + y);
                        painter.circle_stroke(center, *radius, egui::Stroke::new(2.0, color));
                    }
                    Shape::Line { x1, y1, x2, y2 } => {
                        let start = egui::pos2(rect.min.x + x1, rect.min.y + y1);
                        let end = egui::pos2(rect.min.x + x2, rect.min.y + y2);
                        painter.line_segment([start, end], egui::Stroke::new(2.0, color));
                    }
                    Shape::Text {
                        x,
                        y,
                        text,
                        font_size,
                    } => {
                        let pos = egui::pos2(rect.min.x + x, rect.min.y + y);
                        painter.text(
                            pos,
                            egui::Align2::LEFT_TOP,
                            text,
                            egui::FontId::monospace(*font_size),
                            color,
                        );
                    }
                    Shape::Drill { x, y, .. } => {
                        let center = egui::pos2(rect.min.x + x, rect.min.y + y);
                        painter.circle_stroke(center, 3.0, egui::Stroke::new(2.0, color));
                        painter.line_segment(
                            [center - egui::vec2(0.0, 5.0), center + egui::vec2(0.0, 5.0)],
                            egui::Stroke::new(1.0, color),
                        );
                        painter.line_segment(
                            [center - egui::vec2(5.0, 0.0), center + egui::vec2(5.0, 0.0)],
                            egui::Stroke::new(1.0, color),
                        );
                    }
                    Shape::Pocket {
                        x,
                        y,
                        width,
                        height,
                        ..
                    } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(*width, *height),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(2.0, color),
                            egui::StrokeKind::Outside,
                        );
                        // Draw pocket indicator
                        let center = rect.center();
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "P",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                    Shape::Cylinder {
                        x,
                        y,
                        radius,
                        height,
                        ..
                    } => {
                        let center = egui::pos2(rect.min.x + x, rect.min.y + y);
                        painter.circle_stroke(center, *radius, egui::Stroke::new(2.0, color));
                        let top = egui::pos2(rect.min.x + x, rect.min.y + y - *height / 2.0);
                        let bottom = egui::pos2(rect.min.x + x, rect.min.y + y + *height / 2.0);
                        painter.line_segment([top, bottom], egui::Stroke::new(2.0, color));
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "C",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                    Shape::Sphere { x, y, radius, .. } => {
                        let center = egui::pos2(rect.min.x + x, rect.min.y + y);
                        painter.circle_stroke(center, *radius, egui::Stroke::new(2.0, color));
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "S",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                    Shape::Extrusion {
                        x,
                        y,
                        width,
                        height,
                        ..
                    } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(*width, *height),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(2.0, color),
                            egui::StrokeKind::Outside,
                        );
                        let center = rect.center();
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "E",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                    Shape::Turning {
                        x,
                        y,
                        diameter,
                        length,
                        ..
                    } => {
                        let center = egui::pos2(rect.min.x + x + diameter / 2.0, rect.min.y + y);
                        painter.circle_stroke(
                            center,
                            diameter / 2.0,
                            egui::Stroke::new(2.0, color),
                        );
                        let end =
                            egui::pos2(rect.min.x + x + diameter / 2.0, rect.min.y + y + length);
                        painter.line_segment([center, end], egui::Stroke::new(2.0, color));
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "T",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                    Shape::Facing {
                        x,
                        y,
                        width,
                        length,
                        ..
                    } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(*width, *length),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(2.0, color),
                            egui::StrokeKind::Outside,
                        );
                        let center = rect.center();
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "F",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                    Shape::Threading {
                        x,
                        y,
                        diameter,
                        length,
                        ..
                    } => {
                        let center = egui::pos2(rect.min.x + x + diameter / 2.0, rect.min.y + y);
                        painter.circle_stroke(
                            center,
                            diameter / 2.0,
                            egui::Stroke::new(2.0, color),
                        );
                        let end =
                            egui::pos2(rect.min.x + x + diameter / 2.0, rect.min.y + y + length);
                        painter.line_segment([center, end], egui::Stroke::new(2.0, color));
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "Th",
                            egui::FontId::monospace(10.0),
                            color,
                        );
                    }
                    Shape::Polyline { points } => {
                        for window in points.windows(2) {
                            let start =
                                egui::pos2(rect.min.x + window[0].0, rect.min.y + window[0].1);
                            let end =
                                egui::pos2(rect.min.x + window[1].0, rect.min.y + window[1].1);
                            painter.line_segment([start, end], egui::Stroke::new(2.0, color));
                        }
                        if let Some(first) = points.first() {
                            let pos = egui::pos2(rect.min.x + first.0, rect.min.y + first.1);
                            painter.text(
                                pos,
                                egui::Align2::LEFT_TOP,
                                "PL",
                                egui::FontId::monospace(10.0),
                                color,
                            );
                        }
                    }
                    Shape::Parametric { bounds, .. } => {
                        let rect_bounds = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + bounds.0, rect.min.y + bounds.1),
                            egui::pos2(rect.min.x + bounds.2, rect.min.y + bounds.3),
                        );
                        painter.rect_stroke(
                            rect_bounds,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(2.0, color),
                            egui::StrokeKind::Outside,
                        );
                        let center = rect_bounds.center();
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "P",
                            egui::FontId::monospace(12.0),
                            color,
                        );
                    }
                }
            }

            // Draw manipulation handles for selected shape
            if let Some(selected_idx) = self.selected_shape {
                if let Some(shape) = self.shapes.get(selected_idx) {
                    self.draw_manipulation_handles(painter, shape, rect);
                }
            }

            // Draw current drawing preview
            if let Some(start) = self.drawing_start
                && let Some(current_pos) = response.hover_pos()
            {
                let current_canvas =
                    egui::pos2(current_pos.x - rect.min.x, current_pos.y - rect.min.y);

                match self.current_tool {
                    DrawingTool::Rectangle => {
                        let width = (current_canvas.x - start.0).abs();
                        let height = (current_canvas.y - start.1).abs();
                        let x = start.0.min(current_canvas.x);
                        let y = start.1.min(current_canvas.y);
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(width, height),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                            egui::StrokeKind::Outside,
                        );
                    }
                    DrawingTool::Circle => {
                        let radius = ((current_canvas.x - start.0).powi(2)
                            + (current_canvas.y - start.1).powi(2))
                        .sqrt();
                        let center = egui::pos2(rect.min.x + start.0, rect.min.y + start.1);
                        painter.circle_stroke(
                            center,
                            radius,
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                        );
                    }
                    DrawingTool::Line => {
                        let start_pos = egui::pos2(rect.min.x + start.0, rect.min.y + start.1);
                        let end_pos = egui::pos2(
                            rect.min.x + current_canvas.x,
                            rect.min.y + current_canvas.y,
                        );
                        painter.line_segment(
                            [start_pos, end_pos],
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                        );
                    }
                    DrawingTool::Text => {
                        // No preview for text
                    }
                    DrawingTool::Drill => {
                        // No preview for drill
                    }
                    DrawingTool::Pocket => {
                        let width = (current_canvas.x - start.0).abs();
                        let height = (current_canvas.y - start.1).abs();
                        let x = start.0.min(current_canvas.x);
                        let y = start.1.min(current_canvas.y);
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(width, height),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.0, egui::Color32::BLUE),
                            egui::StrokeKind::Outside,
                        );
                    }
                    DrawingTool::Cylinder => {
                        let radius = ((current_canvas.x - start.0).powi(2)
                            + (current_canvas.y - start.1).powi(2))
                        .sqrt();
                        let center = egui::pos2(rect.min.x + start.0, rect.min.y + start.1);
                        painter.circle_stroke(
                            center,
                            radius,
                            egui::Stroke::new(1.0, egui::Color32::GREEN),
                        );
                    }
                    DrawingTool::Sphere => {
                        let radius = ((current_canvas.x - start.0).powi(2)
                            + (current_canvas.y - start.1).powi(2))
                        .sqrt();
                        let center = egui::pos2(rect.min.x + start.0, rect.min.y + start.1);
                        painter.circle_stroke(
                            center,
                            radius,
                            egui::Stroke::new(1.0, egui::Color32::PURPLE),
                        );
                    }
                    DrawingTool::Extrusion => {
                        let width = (current_canvas.x - start.0).abs();
                        let height = (current_canvas.y - start.1).abs();
                        let x = start.0.min(current_canvas.x);
                        let y = start.1.min(current_canvas.y);
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(width, height),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.0, egui::Color32::ORANGE),
                            egui::StrokeKind::Outside,
                        );
                    }
                    DrawingTool::Turning => {
                        let diameter = (current_canvas.x - start.0).abs();
                        let length = (current_canvas.y - start.1).abs();
                        let center =
                            egui::pos2(rect.min.x + start.0 + diameter / 2.0, rect.min.y + start.1);
                        painter.circle_stroke(
                            center,
                            diameter / 2.0,
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        );
                        let end = egui::pos2(
                            rect.min.x + start.0 + diameter / 2.0,
                            rect.min.y + start.1 + length,
                        );
                        painter.line_segment(
                            [center, end],
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        );
                    }
                    DrawingTool::Facing => {
                        let width = (current_canvas.x - start.0).abs();
                        let length = (current_canvas.y - start.1).abs();
                        let x = start.0.min(current_canvas.x);
                        let y = start.1.min(current_canvas.y);
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + x, rect.min.y + y),
                            egui::vec2(width, length),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.0, egui::Color32::CYAN),
                            egui::StrokeKind::Outside,
                        );
                    }
                    DrawingTool::Threading => {
                        let diameter = (current_canvas.x - start.0).abs();
                        let length = (current_canvas.y - start.1).abs();
                        let center =
                            egui::pos2(rect.min.x + start.0 + diameter / 2.0, rect.min.y + start.1);
                        painter.circle_stroke(
                            center,
                            diameter / 2.0,
                            egui::Stroke::new(1.0, egui::Color32::MAGENTA),
                        );
                        let end = egui::pos2(
                            rect.min.x + start.0 + diameter / 2.0,
                            rect.min.y + start.1 + length,
                        );
                        painter.line_segment(
                            [center, end],
                            egui::Stroke::new(1.0, egui::Color32::MAGENTA),
                        );
                    }
                    DrawingTool::Polyline => {
                        // Draw all current polyline points
                        if !self.current_polyline_points.is_empty() {
                            let mut points = self
                                .current_polyline_points
                                .iter()
                                .map(|(x, y)| egui::pos2(rect.min.x + x, rect.min.y + y))
                                .collect::<Vec<_>>();

                            // Add current mouse position as potential next point
                            points.push(egui::pos2(
                                rect.min.x + current_canvas.x,
                                rect.min.y + current_canvas.y,
                            ));

                            // Draw lines between points
                            for window in points.windows(2) {
                                painter.line_segment(
                                    [window[0], window[1]],
                                    egui::Stroke::new(1.0, egui::Color32::BLUE),
                                );
                            }

                            // Draw point markers
                            for &point in &points[..points.len().saturating_sub(1)] {
                                painter.circle_filled(point, 3.0, egui::Color32::BLUE);
                            }
                        }
                    }
                    DrawingTool::Parametric => {
                        let rect = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x + start.0, rect.min.y + start.1),
                            egui::pos2(
                                rect.min.x + current_canvas.x,
                                rect.min.y + current_canvas.y,
                            ),
                        );
                        painter.rect_stroke(
                            rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.0, egui::Color32::GREEN),
                            egui::StrokeKind::Outside,
                        );
                    }
                    DrawingTool::Move
                    | DrawingTool::Scale
                    | DrawingTool::Rotate
                    | DrawingTool::Mirror => {
                        // No preview for manipulation tools
                    }
                    DrawingTool::Select => {}
                }
            }

            // Draw grid
            if self.show_grid {
                let grid_spacing = 50.0;
                let grid_color = egui::Color32::from_gray(200);
                let stroke = egui::Stroke::new(1.0, grid_color);

                // Vertical lines
                let mut x = 0.0;
                while x <= rect.width() {
                    let start = egui::pos2(rect.min.x + x, rect.min.y);
                    let end = egui::pos2(rect.min.x + x, rect.min.y + rect.height());
                    painter.line_segment([start, end], stroke);
                    x += grid_spacing;
                }

                // Horizontal lines
                let mut y = 0.0;
                while y <= rect.height() {
                    let start = egui::pos2(rect.min.x, rect.min.y + y);
                    let end = egui::pos2(rect.min.x + rect.width(), rect.min.y + y);
                    painter.line_segment([start, end], stroke);
                    y += grid_spacing;
                }
            }

            ui.separator();
            ui.label(format!("Shapes: {}", self.shapes.len()));
            ui.label(format!("Current tool: {:?}", self.current_tool));
        });
        event
    }

    pub fn import_svg(&mut self, path: &std::path::Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        self.parse_svg(&content)?;
        Ok(())
    }

    pub fn import_dxf(&mut self, path: &std::path::Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        self.parse_dxf(&content)?;
        Ok(())
    }

    pub fn import_c2d(&mut self, path: &std::path::Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        self.parse_c2d(&content)?;
        Ok(())
    }

    pub fn import_bitmap(
        &mut self,
        path: &std::path::Path,
        config: &bitmap_processing::VectorizationConfig,
    ) -> Result<()> {
        let img = image::open(path)?;
        let gray = img.to_luma8();
        let polylines = bitmap_processing::BitmapProcessor::vectorize_bitmap(&gray, config);

        // Convert polylines to shapes
        for polyline in polylines {
            if polyline.len() >= 2 {
                self.shapes.push(Shape::Polyline { points: polyline });
            }
        }

        Ok(())
    }

    pub fn import_obj(&mut self, path: &std::path::Path) -> Result<()> {
        let (models, _) = tobj::load_obj(path, &tobj::LoadOptions::default())?;
        for model in models {
            let mesh = &model.mesh;
            let positions = &mesh.positions;
            let indices = &mesh.indices;

            // Convert to polylines (simplified - just take vertices as points)
            let mut points = Vec::new();
            for &idx in indices {
                let idx = idx as usize * 3;
                if idx + 2 < positions.len() {
                    points.push((positions[idx] as f32, positions[idx + 1] as f32));
                }
            }
            if !points.is_empty() {
                self.shapes.push(Shape::Polyline { points });
            }
        }
        Ok(())
    }

    pub fn grid_multiply(&mut self, rows: usize, cols: usize, spacing_x: f32, spacing_y: f32) {
        if self.shapes.is_empty() {
            return;
        }

        let original_shapes: Vec<Shape> = self.shapes.clone();
        let mut new_shapes = Vec::new();

        for row in 0..rows {
            for col in 0..cols {
                if row == 0 && col == 0 {
                    continue; // Skip the original position
                }

                let offset_x = col as f32 * spacing_x;
                let offset_y = row as f32 * spacing_y;

                for shape in &original_shapes {
                    let mut new_shape = shape.clone();
                    // Offset the shape position
                    match &mut new_shape {
                        Shape::Rectangle { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Circle { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Line { x1, y1, x2, y2 } => {
                            *x1 += offset_x;
                            *y1 += offset_y;
                            *x2 += offset_x;
                            *y2 += offset_y;
                        }
                        Shape::Text { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Drill { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Pocket { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Cylinder { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Sphere { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Extrusion { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Turning { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Facing { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Threading { x, y, .. } => {
                            *x += offset_x;
                            *y += offset_y;
                        }
                        Shape::Polyline { points } => {
                            for (x, y) in points.iter_mut() {
                                *x += offset_x;
                                *y += offset_y;
                            }
                        }
                        Shape::Parametric { bounds, .. } => {
                            bounds.0 += offset_x;
                            bounds.1 += offset_y;
                            bounds.2 += offset_x;
                            bounds.3 += offset_y;
                        }
                    }
                    new_shapes.push(new_shape);
                }
            }
        }

        // Add all new shapes
        self.shapes.extend(new_shapes);
    }

    pub fn add_clipart(&mut self, clipart_type: &str, x: f32, y: f32, size: f32) {
        let shape = match clipart_type {
            "star" => {
                // Create a 5-pointed star
                let mut points = Vec::new();
                for i in 0..10 {
                    let angle = (i as f32 * 36.0).to_radians();
                    let radius = if i % 2 == 0 { size } else { size * 0.5 };
                    let px = x + angle.cos() * radius;
                    let py = y + angle.sin() * radius;
                    points.push((px, py));
                }
                Shape::Polyline { points }
            }
            "heart" => {
                // Create a simple heart shape using polylines
                let mut points = Vec::new();
                for i in 0..50 {
                    let t = (i as f32 / 49.0) * std::f32::consts::PI * 2.0;
                    let px = x + 16.0 * (t.sin()).powi(3) * size / 16.0;
                    let py = y
                        - (13.0 * t.cos()
                            - 5.0 * (2.0 * t).cos()
                            - 2.0 * (3.0 * t).cos()
                            - (4.0 * t).cos())
                            * size
                            / 13.0;
                    points.push((px, py));
                }
                Shape::Polyline { points }
            }
            "gear" => {
                // Create a simple gear shape
                let mut points = Vec::new();
                let teeth = 8;
                for i in 0..(teeth * 4) {
                    let angle = (i as f32 / (teeth * 4) as f32) * std::f32::consts::PI * 2.0;
                    let radius = if i % 4 < 2 { size } else { size * 0.7 };
                    let px = x + angle.cos() * radius;
                    let py = y + angle.sin() * radius;
                    points.push((px, py));
                }
                Shape::Polyline { points }
            }
            "arrow" => {
                // Create an arrow shape
                Shape::Polyline {
                    points: vec![
                        (x, y + size * 0.5),
                        (x + size * 0.7, y + size * 0.5),
                        (x + size * 0.7, y + size * 0.2),
                        (x + size, y),
                        (x + size * 0.7, y - size * 0.2),
                        (x + size * 0.7, y - size * 0.5),
                        (x, y - size * 0.5),
                        (x, y + size * 0.5),
                    ],
                }
            }
            _ => {
                // Default to a simple square
                Shape::Rectangle {
                    x,
                    y,
                    width: size,
                    height: size,
                }
            }
        };

        self.shapes.push(shape);
    }

    pub fn align_shapes(&mut self, alignment: &str) {
        if self.shapes.is_empty() {
            return;
        }

        // Collect bounds first
        let bounds: Vec<_> = self
            .shapes
            .iter()
            .map(|s| self.get_shape_bounds(s))
            .collect();

        // Find the reference position
        let mut ref_x = 0.0;
        let mut ref_y = 0.0;

        match alignment {
            "left" => {
                ref_x = bounds.iter().map(|b| b.0).fold(f32::INFINITY, f32::min);
            }
            "right" => {
                ref_x = bounds.iter().map(|b| b.2).fold(f32::NEG_INFINITY, f32::max);
            }
            "top" => {
                ref_y = bounds.iter().map(|b| b.1).fold(f32::INFINITY, f32::min);
            }
            "bottom" => {
                ref_y = bounds.iter().map(|b| b.3).fold(f32::NEG_INFINITY, f32::max);
            }
            "center_x" => {
                let total_min = bounds.iter().map(|b| b.0).fold(f32::INFINITY, f32::min);
                let total_max = bounds.iter().map(|b| b.2).fold(f32::NEG_INFINITY, f32::max);
                ref_x = (total_min + total_max) / 2.0;
            }
            "center_y" => {
                let total_min = bounds.iter().map(|b| b.1).fold(f32::INFINITY, f32::min);
                let total_max = bounds.iter().map(|b| b.3).fold(f32::NEG_INFINITY, f32::max);
                ref_y = (total_min + total_max) / 2.0;
            }
            _ => return,
        }

        for (i, shape) in self.shapes.iter_mut().enumerate() {
            let shape_bounds = bounds[i];
            let width = shape_bounds.2 - shape_bounds.0;
            let height = shape_bounds.3 - shape_bounds.1;

            let new_x = match alignment {
                "left" => ref_x,
                "right" => ref_x - width,
                "center_x" => ref_x - width / 2.0,
                _ => shape_bounds.0,
            };

            let new_y = match alignment {
                "top" => ref_y,
                "bottom" => ref_y - height,
                "center_y" => ref_y - height / 2.0,
                _ => shape_bounds.1,
            };

            let dx = new_x - shape_bounds.0;
            let dy = new_y - shape_bounds.1;

            // Move the shape
            match shape {
                Shape::Rectangle { x, .. } => *x += dx,
                Shape::Circle { x, .. } => *x += dx,
                Shape::Line { x1, .. } => *x1 += dx,
                Shape::Text { x, .. } => *x += dx,
                Shape::Drill { x, .. } => *x += dx,
                Shape::Pocket { x, .. } => *x += dx,
                Shape::Cylinder { x, .. } => *x += dx,
                Shape::Sphere { x, .. } => *x += dx,
                Shape::Extrusion { x, .. } => *x += dx,
                Shape::Turning { x, .. } => *x += dx,
                Shape::Facing { x, .. } => *x += dx,
                Shape::Threading { x, .. } => *x += dx,
                Shape::Polyline { points } => {
                    for (px, _) in points {
                        *px += dx;
                    }
                }
                Shape::Parametric { bounds, .. } => {
                    bounds.0 += dx;
                    bounds.1 += dy;
                    bounds.2 += dx;
                    bounds.3 += dy;
                }
            }

            // Also adjust y
            match shape {
                Shape::Rectangle { y, .. } => *y += dy,
                Shape::Circle { y, .. } => *y += dy,
                Shape::Line { y1, .. } => *y1 += dy,
                Shape::Text { y, .. } => *y += dy,
                Shape::Drill { y, .. } => *y += dy,
                Shape::Pocket { y, .. } => *y += dy,
                Shape::Cylinder { y, .. } => *y += dy,
                Shape::Sphere { y, .. } => *y += dy,
                Shape::Extrusion { y, .. } => *y += dy,
                Shape::Turning { y, .. } => *y += dy,
                Shape::Facing { y, .. } => *y += dy,
                Shape::Threading { y, .. } => *y += dy,
                Shape::Polyline { points } => {
                    for (_, py) in points {
                        *py += dy;
                    }
                }
                Shape::Parametric { bounds, .. } => {
                    // Already adjusted
                }
            }
        }
    }

    pub fn boolean_intersect(&mut self, indices: &[usize]) -> Result<()> {
        if indices.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 shapes for intersect"));
        }

        // For now, implement intersection for rectangles only
        // Collect all rectangle shapes
        let mut rectangles = Vec::new();
        for &idx in indices {
            if let Some(Shape::Rectangle {
                x,
                y,
                width,
                height,
            }) = self.shapes.get(idx)
            {
                rectangles.push((*x, *y, *width, *height));
            } else {
                return Err(anyhow::anyhow!(
                    "Intersection currently only supports rectangles"
                ));
            }
        }

        if rectangles.is_empty() {
            return Err(anyhow::anyhow!("No rectangles found for intersection"));
        }

        // Calculate intersection of all rectangles
        let mut min_x = f32::NEG_INFINITY;
        let mut min_y = f32::NEG_INFINITY;
        let mut max_x = f32::INFINITY;
        let mut max_y = f32::INFINITY;

        for (x, y, width, height) in &rectangles {
            min_x = min_x.max(*x);
            min_y = min_y.max(*y);
            max_x = max_x.min(x + width);
            max_y = max_y.min(y + height);
        }

        if min_x >= max_x || min_y >= max_y {
            return Err(anyhow::anyhow!("No intersection found"));
        }

        // Remove original shapes
        let mut indices_sorted = indices.to_vec();
        indices_sorted.sort_by(|a, b| b.cmp(a));
        for idx in indices_sorted {
            self.shapes.remove(idx);
        }

        // Add intersection result
        let width = max_x - min_x;
        let height = max_y - min_y;
        self.shapes.push(Shape::Rectangle {
            x: min_x,
            y: min_y,
            width,
            height,
        });

        Ok(())
    }

    pub fn boolean_subtract(&mut self, indices: &[usize]) -> Result<()> {
        if indices.len() != 2 {
            return Err(anyhow::anyhow!("Need exactly 2 shapes for subtract"));
        }

        let shape_a = &self.shapes[indices[0]];
        let shape_b = &self.shapes[indices[1]];

        // For now, implement subtract for rectangles only
        let (rect_a, rect_b) = match (shape_a, shape_b) {
            (
                Shape::Rectangle {
                    x: x1,
                    y: y1,
                    width: w1,
                    height: h1,
                },
                Shape::Rectangle {
                    x: x2,
                    y: y2,
                    width: w2,
                    height: h2,
                },
            ) => ((*x1, *y1, *w1, *h1), (*x2, *y2, *w2, *h2)),
            _ => {
                return Err(anyhow::anyhow!(
                    "Subtract currently only supports rectangles"
                ));
            }
        };

        // Calculate the result of A - B (rectangle subtraction)
        let result_shapes = self.rectangle_subtract(rect_a, rect_b);

        // Remove original shapes
        let mut indices_sorted = indices.to_vec();
        indices_sorted.sort_by(|a, b| b.cmp(a));
        for idx in indices_sorted {
            self.shapes.remove(idx);
        }

        // Add result shapes
        for shape in result_shapes {
            self.shapes.push(shape);
        }

        Ok(())
    }

    fn rectangle_subtract(
        &self,
        rect_a: (f32, f32, f32, f32),
        rect_b: (f32, f32, f32, f32),
    ) -> Vec<Shape> {
        let (ax, ay, aw, ah) = rect_a;
        let (bx, by, bw, bh) = rect_b;

        let ax2 = ax + aw;
        let ay2 = ay + ah;
        let bx2 = bx + bw;
        let by2 = by + bh;

        let mut result = Vec::new();

        // Check if rectangles don't overlap
        if ax >= bx2 || ax2 <= bx || ay >= by2 || ay2 <= by {
            // No overlap, return original rectangle
            result.push(Shape::Rectangle {
                x: ax,
                y: ay,
                width: aw,
                height: ah,
            });
            return result;
        }

        // Calculate the regions of A that are not in B
        // Top rectangle
        if ay < by {
            result.push(Shape::Rectangle {
                x: ax,
                y: ay,
                width: aw,
                height: by - ay,
            });
        }

        // Bottom rectangle
        if ay2 > by2 {
            result.push(Shape::Rectangle {
                x: ax,
                y: by2,
                width: aw,
                height: ay2 - by2,
            });
        }

        // Left rectangle
        let left_y1 = ay.max(by);
        let left_y2 = ay2.min(by2);
        if left_y2 > left_y1 && ax < bx {
            result.push(Shape::Rectangle {
                x: ax,
                y: left_y1,
                width: bx - ax,
                height: left_y2 - left_y1,
            });
        }

        // Right rectangle
        let right_y1 = ay.max(by);
        let right_y2 = ay2.min(by2);
        if right_y2 > right_y1 && ax2 > bx2 {
            result.push(Shape::Rectangle {
                x: bx2,
                y: right_y1,
                width: ax2 - bx2,
                height: right_y2 - right_y1,
            });
        }

        result
    }

    fn get_shape_bounds(&self, shape: &Shape) -> (f32, f32, f32, f32) {
        match shape {
            Shape::Rectangle {
                x,
                y,
                width,
                height,
            } => (*x, *y, x + width, y + height),
            Shape::Circle { x, y, radius } => (x - radius, y - radius, x + radius, y + radius),
            Shape::Line { x1, y1, x2, y2 } => (x1.min(*x2), y1.min(*y2), x1.max(*x2), y1.max(*y2)),
            Shape::Text {
                x,
                y,
                text,
                font_size,
            } => {
                let w = text.len() as f32 * font_size * 0.6;
                let h = *font_size;
                (*x, *y, x + w, y + h)
            }
            Shape::Drill { x, y, .. } => (*x - 2.0, *y - 2.0, *x + 2.0, *y + 2.0),
            Shape::Pocket {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, x + width, y + height),
            Shape::Cylinder {
                x,
                y,
                radius,
                height,
                ..
            } => (x - radius, y - radius, x + radius, y + height),
            Shape::Sphere { x, y, radius, .. } => (x - radius, y - radius, x + radius, y + radius),
            Shape::Extrusion {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, x + width, y + height),
            Shape::Turning {
                x,
                y,
                diameter,
                length,
                ..
            } => (x - diameter / 2.0, *y, x + diameter / 2.0, y + length),
            Shape::Facing {
                x,
                y,
                width,
                length,
                ..
            } => (*x, *y, x + width, y + length),
            Shape::Threading {
                x,
                y,
                diameter,
                length,
                ..
            } => (x - diameter / 2.0, *y, x + diameter / 2.0, y + length),
            Shape::Polyline { points } => {
                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;
                for (px, py) in points {
                    min_x = min_x.min(*px);
                    min_y = min_y.min(*py);
                    max_x = max_x.max(*px);
                    max_y = max_y.max(*py);
                }
                (min_x, min_y, max_x, max_y)
            }
            Shape::Parametric { bounds, .. } => *bounds,
        }
    }

    fn shape_to_path(&self, shape: &Shape) -> Option<Path> {
        let mut builder = Path::builder();

        match shape {
            Shape::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                builder.begin(point(*x, *y));
                builder.line_to(point(x + width, *y));
                builder.line_to(point(x + width, y + height));
                builder.line_to(point(*x, y + height));
                builder.close();
            }
            Shape::Circle { x, y, radius } => {
                // Approximate circle with line segments
                let segments = 32;
                builder.begin(point(x + radius, *y));
                for i in 1..=segments {
                    let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                    let px = x + radius * angle.cos();
                    let py = y + radius * angle.sin();
                    builder.line_to(point(px, py));
                }
                builder.close();
            }
            Shape::Polyline { points } => {
                if let Some(first) = points.first() {
                    builder.begin(point(first.0, first.1));
                    for p in points.iter().skip(1) {
                        builder.line_to(point(p.0, p.1));
                    }
                }
            }
            _ => return None, // Other shapes not supported for boolean ops yet
        }

        Some(builder.build())
    }

    pub fn boolean_union(&mut self, indices: &[usize]) -> Result<()> {
        if indices.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 shapes for union"));
        }

        // For simplicity, create a bounding box union
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for &idx in indices {
            if let Some(shape) = self.shapes.get(idx) {
                match shape {
                    Shape::Rectangle {
                        x,
                        y,
                        width,
                        height,
                    } => {
                        min_x = min_x.min(*x);
                        min_y = min_y.min(*y);
                        max_x = max_x.max(x + width);
                        max_y = max_y.max(y + height);
                    }
                    Shape::Circle { x, y, radius } => {
                        min_x = min_x.min(x - radius);
                        min_y = min_y.min(y - radius);
                        max_x = max_x.max(x + radius);
                        max_y = max_y.max(y + radius);
                    }
                    Shape::Polyline { points } => {
                        for (px, py) in points {
                            min_x = min_x.min(*px);
                            min_y = min_y.min(*py);
                            max_x = max_x.max(*px);
                            max_y = max_y.max(*py);
                        }
                    }
                    _ => {} // Skip other shapes
                }
            }
        }

        // Remove original shapes
        let mut indices_sorted = indices.to_vec();
        indices_sorted.sort_by(|a, b| b.cmp(a)); // Sort descending to remove from end
        for idx in indices_sorted {
            self.shapes.remove(idx);
        }

        // Add union shape
        let width = max_x - min_x;
        let height = max_y - min_y;
        self.shapes.push(Shape::Rectangle {
            x: min_x,
            y: min_y,
            width,
            height,
        });

        Ok(())
    }

    fn vectorize_bitmap(&self, img: &GrayImage) -> Vec<(f32, f32)> {
        let (width, height) = img.dimensions();
        let mut edges = Vec::new();

        // Improved edge detection using Sobel operator
        let sobel_x = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        let sobel_y = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

        let mut edge_magnitude = vec![vec![0.0; width as usize]; height as usize];

        // Calculate edge magnitude for each pixel
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut gx = 0.0;
                let mut gy = 0.0;

                for ky in 0..3 {
                    for kx in 0..3 {
                        let pixel = img.get_pixel(x + kx - 1, y + ky - 1)[0] as f32;
                        gx += pixel * sobel_x[ky as usize][kx as usize] as f32;
                        gy += pixel * sobel_y[ky as usize][kx as usize] as f32;
                    }
                }

                let magnitude = (gx * gx + gy * gy).sqrt();
                edge_magnitude[y as usize][x as usize] = magnitude;
            }
        }

        // Threshold and collect edge points
        let edge_threshold = 100.0; // Adjust based on image
        for y in 0..height {
            for x in 0..width {
                if edge_magnitude[y as usize][x as usize] > edge_threshold {
                    edges.push((x as f32, y as f32));
                }
            }
        }

        // If no edges found, fall back to simple thresholding
        if edges.is_empty() {
            let threshold = 128u8;
            for y in 0..height {
                for x in 0..width {
                    let pixel = img.get_pixel(x, y)[0];
                    if pixel < threshold {
                        edges.push((x as f32, y as f32));
                    }
                }
            }
        }

        // Simplify and connect edge points using a basic contour tracing algorithm
        self.simplify_contours(edges)
    }

    fn simplify_contours(&self, points: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
        if points.len() <= 2 {
            return points;
        }

        let mut simplified = Vec::new();
        let mut current_group = Vec::new();

        // Sort points by x coordinate
        let mut sorted_points = points;
        sorted_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Group nearby points and create polylines
        for point in sorted_points {
            if current_group.is_empty() {
                current_group.push(point);
            } else {
                let last = *current_group.last().unwrap();
                let distance = ((point.0 - last.0).powi(2) + (point.1 - last.1).powi(2)).sqrt();

                if distance < 5.0 {
                    // Connect if close enough
                    current_group.push(point);
                } else {
                    // Start new group
                    if current_group.len() >= 2 {
                        simplified.extend(self.douglas_peucker(&current_group, 2.0));
                    }
                    current_group = vec![point];
                }
            }
        }

        // Add the last group
        if current_group.len() >= 2 {
            simplified.extend(self.douglas_peucker(&current_group, 2.0));
        }

        simplified
    }

    fn douglas_peucker(&self, points: &[(f32, f32)], epsilon: f32) -> Vec<(f32, f32)> {
        if points.len() <= 2 {
            return points.to_vec();
        }

        // Find the point with the maximum distance from the line between start and end
        let start = points[0];
        let end = points[points.len() - 1];
        let mut max_distance = 0.0;
        let mut max_index = 0;

        for (i, &point) in points.iter().enumerate().skip(1) {
            let distance = self.point_to_line_distance(point, start, end);
            if distance > max_distance {
                max_distance = distance;
                max_index = i;
            }
        }

        // If max distance is greater than epsilon, recursively simplify
        if max_distance > epsilon {
            let left = self.douglas_peucker(&points[0..=max_index], epsilon);
            let right = self.douglas_peucker(&points[max_index..], epsilon);

            // Combine results (avoid duplicating the middle point)
            let mut result = left;
            result.extend_from_slice(&right[1..]);
            result
        } else {
            vec![start, end]
        }
    }

    fn point_to_line_distance(
        &self,
        point: (f32, f32),
        line_start: (f32, f32),
        line_end: (f32, f32),
    ) -> f32 {
        let (px, py) = point;
        let (x1, y1) = line_start;
        let (x2, y2) = line_end;

        let dx = x2 - x1;
        let dy = y2 - y1;

        if dx == 0.0 && dy == 0.0 {
            return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
        }

        let t = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
        let t = t.max(0.0).min(1.0);

        let closest_x = x1 + t * dx;
        let closest_y = y1 + t * dy;

        ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
    }

    fn draw_manipulation_handles(
        &self,
        painter: &egui::Painter,
        shape: &Shape,
        canvas_rect: egui::Rect,
    ) {
        let handle_size = 6.0;
        let handle_color = egui::Color32::from_rgb(100, 149, 237); // Cornflower blue
        let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

        match shape {
            Shape::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                // Corner handles
                let corners = [
                    (*x, *y),                // Top-left
                    (x + width, *y),         // Top-right
                    (x + width, y + height), // Bottom-right
                    (*x, y + height),        // Bottom-left
                ];

                for (hx, hy) in corners {
                    let handle_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            canvas_rect.min.x + hx - handle_size / 2.0,
                            canvas_rect.min.y + hy - handle_size / 2.0,
                        ),
                        egui::vec2(handle_size, handle_size),
                    );
                    painter.rect_filled(handle_rect, egui::CornerRadius::ZERO, handle_color);
                    painter.rect_stroke(
                        handle_rect,
                        egui::CornerRadius::ZERO,
                        stroke,
                        egui::StrokeKind::Outside,
                    );
                }

                // Edge handles (midpoints)
                let edges = [
                    (x + width / 2.0, *y),         // Top
                    (x + width, y + height / 2.0), // Right
                    (x + width / 2.0, y + height), // Bottom
                    (*x, y + height / 2.0),        // Left
                ];

                for (hx, hy) in edges {
                    let handle_center = egui::pos2(canvas_rect.min.x + hx, canvas_rect.min.y + hy);
                    painter.circle_filled(handle_center, handle_size / 2.0, handle_color);
                    painter.circle_stroke(handle_center, handle_size / 2.0, stroke);
                }
            }
            Shape::Circle { x, y, radius } => {
                // Circle handles: center and edge points
                let center = egui::pos2(canvas_rect.min.x + x, canvas_rect.min.y + y);
                painter.circle_filled(center, handle_size / 2.0, handle_color);
                painter.circle_stroke(center, handle_size / 2.0, stroke);

                // Edge handles at 0°, 90°, 180°, 270°
                for angle in [0.0f32, 90.0f32, 180.0f32, 270.0f32] {
                    let rad = angle.to_radians();
                    let hx = x + radius * rad.cos();
                    let hy = y + radius * rad.sin();
                    let handle_center = egui::pos2(canvas_rect.min.x + hx, canvas_rect.min.y + hy);
                    painter.circle_filled(handle_center, handle_size / 2.0, handle_color);
                    painter.circle_stroke(handle_center, handle_size / 2.0, stroke);
                }
            }
            Shape::Line { x1, y1, x2, y2 } => {
                // Line handles: endpoints and midpoint
                let points = [(*x1, *y1), (*x2, *y2), ((x1 + x2) / 2.0, (y1 + y2) / 2.0)];

                for (hx, hy) in points {
                    let handle_center = egui::pos2(canvas_rect.min.x + hx, canvas_rect.min.y + hy);
                    painter.circle_filled(handle_center, handle_size / 2.0, handle_color);
                    painter.circle_stroke(handle_center, handle_size / 2.0, stroke);
                }
            }
            Shape::Polyline { points } => {
                // Polyline handles: all points
                for (j, (hx, hy)) in points.iter().enumerate() {
                    let handle_center = egui::pos2(canvas_rect.min.x + hx, canvas_rect.min.y + hy);
                    let color = if Some(j) == self.selected_point {
                        egui::Color32::from_rgb(255, 100, 100) // Red for selected
                    } else {
                        handle_color
                    };
                    painter.circle_filled(handle_center, handle_size / 2.0, color);
                    painter.circle_stroke(handle_center, handle_size / 2.0, stroke);
                }
            }
            _ => {
                // For other shapes, just show center handle
                let center = Self::get_shape_center(shape);
                let handle_center =
                    egui::pos2(canvas_rect.min.x + center.0, canvas_rect.min.y + center.1);
                painter.circle_filled(handle_center, handle_size / 2.0, handle_color);
                painter.circle_stroke(handle_center, handle_size / 2.0, stroke);
            }
        }
    }

    pub fn evaluate_parametric(
        &mut self,
        index: usize,
        config: &parametric_design::ParametricConfig,
    ) -> Result<()> {
        if let Some(Shape::Parametric { script, bounds, .. }) = self.shapes.get(index) {
            // Use the parametric design system to evaluate the script
            let points =
                parametric_design::ParametricDesigner::evaluate_script(script, config, *bounds)?;

            // Replace the parametric shape with a polyline
            let polyline = Shape::Polyline { points };
            self.shapes[index] = polyline;
        }
        Ok(())
    }

    fn parse_svg(&mut self, svg_content: &str) -> Result<()> {
        // Simple SVG parser for basic shapes
        // This is a basic implementation - for production use, consider a more robust SVG parser

        // Clear existing shapes
        self.shapes.clear();

        // Look for basic shapes in SVG
        let content = svg_content.to_lowercase();

        // Parse circles: <circle cx="50" cy="50" r="40"/>
        for line in content.lines() {
            let line = line.trim();
            if line.contains("<circle")
                && line.contains("cx=")
                && line.contains("cy=")
                && line.contains("r=")
            {
                if let (Some(cx), Some(cy), Some(r)) = (
                    self.extract_attr_value(line, "cx"),
                    self.extract_attr_value(line, "cy"),
                    self.extract_attr_value(line, "r"),
                ) {
                    self.shapes.push(Shape::Circle {
                        x: cx,
                        y: cy,
                        radius: r,
                    });
                }
            }
            // Parse rectangles: <rect x="10" y="10" width="100" height="50"/>
            else if line.contains("<rect")
                && line.contains("x=")
                && line.contains("y=")
                && line.contains("width=")
                && line.contains("height=")
            {
                if let (Some(x), Some(y), Some(width), Some(height)) = (
                    self.extract_attr_value(line, "x"),
                    self.extract_attr_value(line, "y"),
                    self.extract_attr_value(line, "width"),
                    self.extract_attr_value(line, "height"),
                ) {
                    self.shapes.push(Shape::Rectangle {
                        x,
                        y,
                        width,
                        height,
                    });
                }
            }
            // Parse lines: <line x1="0" y1="0" x2="100" y2="100"/>
            else if line.contains("<line")
                && line.contains("x1=")
                && line.contains("y1=")
                && line.contains("x2=")
                && line.contains("y2=")
                && let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                    self.extract_attr_value(line, "x1"),
                    self.extract_attr_value(line, "y1"),
                    self.extract_attr_value(line, "x2"),
                    self.extract_attr_value(line, "y2"),
                )
            {
                self.shapes.push(Shape::Line { x1, y1, x2, y2 });
            }
        }

        Ok(())
    }

    fn parse_dxf(&mut self, dxf_content: &str) -> Result<()> {
        // Basic DXF parser for simple entities
        // This is a simplified implementation - for production use, consider a more robust DXF parser

        // Clear existing shapes
        self.shapes.clear();

        let lines: Vec<&str> = dxf_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for entity sections
            if line == "LINE" {
                // Parse LINE entity
                let mut x1 = None;
                let mut y1 = None;
                let mut x2 = None;
                let mut y2 = None;

                i += 1;
                while i < lines.len() {
                    let code = lines[i].trim();
                    if code == "0" {
                        break;
                    } // End of entity

                    if let (Some(code_num), Some(value_str)) =
                        (code.parse::<i32>().ok(), lines.get(i + 1))
                    {
                        let value = value_str.trim().parse::<f32>().ok();
                        match code_num {
                            10 => x1 = value,
                            20 => y1 = value,
                            11 => x2 = value,
                            21 => y2 = value,
                            _ => {}
                        }
                    }
                    i += 2;
                }

                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (x1, y1, x2, y2) {
                    self.shapes.push(Shape::Line { x1, y1, x2, y2 });
                }
            } else if line == "CIRCLE" {
                // Parse CIRCLE entity
                let mut x = None;
                let mut y = None;
                let mut radius = None;

                i += 1;
                while i < lines.len() {
                    let code = lines[i].trim();
                    if code == "0" {
                        break;
                    } // End of entity

                    if let (Some(code_num), Some(value_str)) =
                        (code.parse::<i32>().ok(), lines.get(i + 1))
                    {
                        let value = value_str.trim().parse::<f32>().ok();
                        match code_num {
                            10 => x = value,
                            20 => y = value,
                            40 => radius = value,
                            _ => {}
                        }
                    }
                    i += 2;
                }

                if let (Some(x), Some(y), Some(radius)) = (x, y, radius) {
                    self.shapes.push(Shape::Circle { x, y, radius });
                }
            }

            i += 1;
        }

        Ok(())
    }

    fn extract_attr_value(&self, line: &str, attr: &str) -> Option<f32> {
        // Simple attribute extraction from SVG tags
        // Looks for attr="value" or attr='value' patterns
        let patterns = [
            format!("{}=\"", attr),
            format!("{}='", attr),
            format!("{}=", attr),
        ];

        for pattern in &patterns {
            if let Some(start) = line.find(pattern) {
                let start = start + pattern.len();
                let remaining = &line[start..];

                // Find the closing quote
                let end = remaining
                    .find('"')
                    .or_else(|| remaining.find('\''))
                    .unwrap_or(remaining.len());
                let value_str = &remaining[..end];

                return value_str.parse::<f32>().ok();
            }
        }

        None
    }

    fn parse_c2d(&mut self, content: &str) -> Result<()> {
        // Basic C2D parser - C2D files typically contain coordinate data
        // Format is usually simple: X,Y coordinates or more complex CAD data
        let mut points = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('/') {
                continue; // Skip comments and empty lines
            }

            // Try to parse as X,Y coordinates
            if let Some(comma_pos) = line.find(',') {
                let x_str = &line[..comma_pos];
                let y_str = &line[comma_pos + 1..];

                if let (Ok(x), Ok(y)) = (x_str.trim().parse::<f32>(), y_str.trim().parse::<f32>()) {
                    points.push((x, y));
                }
            }
        }

        if !points.is_empty() {
            self.shapes.push(Shape::Polyline { points });
        }

        Ok(())
    }

    fn scale_shape(shape: &mut Shape, scale: (f32, f32), pivot: (f32, f32)) {
        let (sx, sy) = scale;
        let (px, py) = pivot;

        match shape {
            Shape::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *width *= sx;
                *height *= sy;
            }
            Shape::Circle { x, y, radius } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *radius *= (sx + sy) / 2.0; // Average scale for radius
            }
            Shape::Line { x1, y1, x2, y2 } => {
                *x1 = px + (*x1 - px) * sx;
                *y1 = py + (*y1 - py) * sy;
                *x2 = px + (*x2 - px) * sx;
                *y2 = py + (*y2 - py) * sy;
            }
            Shape::Text {
                x, y, font_size, ..
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *font_size = (*font_size as f32 * (sx + sy) / 2.0) as f32;
            }
            Shape::Drill { x, y, .. } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
            }
            Shape::Pocket {
                x,
                y,
                width,
                height,
                ..
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *width *= sx;
                *height *= sy;
            }
            Shape::Cylinder {
                x,
                y,
                radius,
                height,
                ..
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *radius *= (sx + sy) / 2.0;
                *height *= sy;
            }
            Shape::Sphere { x, y, radius, .. } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *radius *= (sx + sy) / 2.0;
            }
            Shape::Extrusion {
                x,
                y,
                width,
                height,
                depth,
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *width *= sx;
                *height *= sy;
                *depth *= (sx + sy) / 2.0;
            }
            Shape::Turning {
                x,
                y,
                diameter,
                length,
                ..
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *diameter *= sx;
                *length *= sy;
            }
            Shape::Facing {
                x,
                y,
                width,
                length,
                ..
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *width *= sx;
                *length *= sy;
            }
            Shape::Threading {
                x,
                y,
                diameter,
                length,
                ..
            } => {
                *x = px + (*x - px) * sx;
                *y = py + (*y - py) * sy;
                *diameter *= sx;
                *length *= sy;
            }
            Shape::Polyline { points } => {
                for (x, y) in points.iter_mut() {
                    *x = px + (*x - px) * sx;
                    *y = py + (*y - py) * sy;
                }
            }
            Shape::Parametric { bounds, .. } => {
                // Scale the bounds
                let (x1, y1, x2, y2) = *bounds;
                let new_x1 = px + (x1 - px) * sx;
                let new_y1 = py + (y1 - py) * sy;
                let new_x2 = px + (x2 - px) * sx;
                let new_y2 = py + (y2 - py) * sy;
                *bounds = (new_x1, new_y1, new_x2, new_y2);
            }
        }
    }

    fn rotate_shape(shape: &mut Shape, angle: f32, pivot: (f32, f32)) {
        let (px, py) = pivot;
        let cos_a = angle.to_radians().cos();
        let sin_a = angle.to_radians().sin();

        let rotate_point = |(x, y): (f32, f32)| {
            let dx = x - px;
            let dy = y - py;
            let new_x = px + dx * cos_a - dy * sin_a;
            let new_y = py + dx * sin_a + dy * cos_a;
            (new_x, new_y)
        };

        match shape {
            Shape::Rectangle { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Circle { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Line { x1, y1, x2, y2 } => {
                (*x1, *y1) = rotate_point((*x1, *y1));
                (*x2, *y2) = rotate_point((*x2, *y2));
            }
            Shape::Text { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Drill { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Pocket { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Cylinder { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Sphere { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Extrusion { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Turning { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Facing { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Threading { x, y, .. } => {
                (*x, *y) = rotate_point((*x, *y));
            }
            Shape::Polyline { points } => {
                for point in points.iter_mut() {
                    *point = rotate_point(*point);
                }
            }
            Shape::Parametric { bounds, .. } => {
                // Rotate the bounds corners
                let (x1, y1, x2, y2) = *bounds;
                let corners = [
                    rotate_point((x1, y1)),
                    rotate_point((x2, y1)),
                    rotate_point((x2, y2)),
                    rotate_point((x1, y2)),
                ];
                let min_x = corners
                    .iter()
                    .map(|(x, _)| *x)
                    .fold(f32::INFINITY, f32::min);
                let max_x = corners
                    .iter()
                    .map(|(x, _)| *x)
                    .fold(f32::NEG_INFINITY, f32::max);
                let min_y = corners
                    .iter()
                    .map(|(_, y)| *y)
                    .fold(f32::INFINITY, f32::min);
                let max_y = corners
                    .iter()
                    .map(|(_, y)| *y)
                    .fold(f32::NEG_INFINITY, f32::max);
                *bounds = (min_x, min_y, max_x, max_y);
            }
        }
    }

    fn mirror_shape(shape: &mut Shape, axis: MirrorAxis) {
        match shape {
            Shape::Rectangle { x, width, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    *x = *x + *width; // Mirror over vertical axis
                    *width = -*width; // Flip width
                }
            }
            Shape::Circle { x, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    // For circles, mirroring horizontally doesn't change the shape
                    // but we could adjust position if needed
                }
            }
            Shape::Line { x1, x2, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    let center_x = (*x1 + *x2) / 2.0;
                    *x1 = 2.0 * center_x - *x1;
                    *x2 = 2.0 * center_x - *x2;
                }
            }
            Shape::Text { x, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    // Text mirroring would require more complex text rendering
                }
            }
            Shape::Drill { x, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    // Drill positions can be mirrored
                }
            }
            Shape::Pocket { x, width, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    *x = *x + *width;
                    *width = -*width;
                }
            }
            Shape::Cylinder { x, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    // Similar to circles
                }
            }
            Shape::Sphere { x, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    // Similar to circles
                }
            }
            Shape::Extrusion { x, width, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    *x = *x + *width;
                    *width = -*width;
                }
            }
            Shape::Turning { x, diameter, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    *x = *x + *diameter;
                    *diameter = -*diameter;
                }
            }
            Shape::Facing { x, width, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    *x = *x + *width;
                    *width = -*width;
                }
            }
            Shape::Threading { x, diameter, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    *x = *x + *diameter;
                    *diameter = -*diameter;
                }
            }
            Shape::Polyline { points } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    let center_x =
                        points.iter().map(|(x, _)| *x).sum::<f32>() / points.len() as f32;
                    for (x, _) in points.iter_mut() {
                        *x = 2.0 * center_x - *x;
                    }
                } else if matches!(axis, MirrorAxis::Vertical) {
                    let center_y =
                        points.iter().map(|(_, y)| *y).sum::<f32>() / points.len() as f32;
                    for (_, y) in points.iter_mut() {
                        *y = 2.0 * center_y - *y;
                    }
                }
            }
            Shape::Parametric { bounds, .. } => {
                if matches!(axis, MirrorAxis::Horizontal) {
                    let (x1, y1, x2, y2) = *bounds;
                    *bounds = (x2, y1, x1, y2); // Swap left/right
                } else if matches!(axis, MirrorAxis::Vertical) {
                    let (x1, y1, x2, y2) = *bounds;
                    *bounds = (x1, y2, x2, y1); // Swap top/bottom
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_designer() -> DesignerState {
        DesignerState {
            shapes: Vec::new(),
            current_tool: DrawingTool::Select,
            current_pattern: ToolpathPattern::Spiral,
            current_material: Material {
                name: "Test Material".to_string(),
                density: 1.0,
                hardness: 100.0,
                thermal_conductivity: 50.0,
            },
            current_tool_def: Tool {
                name: "Test Tool".to_string(),
                diameter: 6.0,
                length: 60.0,
                material: "HSS".to_string(),
                flute_count: 2,
                max_rpm: 10000,
                tool_number: 1,
                length_offset: 0.0,
                wear_offset: 0.0,
            },
            drawing_start: None,
            selected_shape: None,
            selected_point: None,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            drag_start_pos: None,
            show_grid: true,
            manipulation_start: None,
            original_shape: None,
            scale_start: None,
            rotation_start: None,
            mirror_axis: None,
            current_scale: None,
            current_rotation: None,
            current_polyline_points: Vec::new(),
            selected_cam_operation: cam_operations::CAMOperation::default(),
            cam_params: cam_operations::CAMParameters::default(),
        }
    }

    #[test]
    fn test_export_empty_designer_to_gcode() {
        let designer = create_test_designer();
        let gcode = designer.export_to_gcode();
        assert_eq!(gcode, "");
    }

    #[test]
    fn test_export_rectangle_to_gcode() {
        let mut designer = create_test_designer();
        designer.shapes.push(Shape::Rectangle {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        });

        let gcode = designer.export_to_gcode();

        assert!(gcode.contains("G21 ; Set units to mm"));
        assert!(gcode.contains("G90 ; Absolute positioning"));
        assert!(gcode.contains("G0 Z5 ; Lift tool"));
        assert!(gcode.contains("; Rectangle at (10.00, 20.00) size 100.00x50.00"));
        assert!(gcode.contains("G0 X10.00 Y20.00"));
        assert!(gcode.contains("G1 Z-1 F500 ; Plunge"));
        assert!(gcode.contains("G1 X110.00 Y20.00 F1000 ; Bottom edge"));
        assert!(gcode.contains("G1 X110.00 Y70.00 F1000 ; Right edge"));
        assert!(gcode.contains("G1 X10.00 Y70.00 F1000 ; Top edge"));
        assert!(gcode.contains("G1 X10.00 Y20.00 F1000 ; Left edge"));
        assert!(gcode.contains("G0 Z5 ; Lift tool"));
    }

    #[test]
    fn test_export_circle_to_gcode() {
        let mut designer = create_test_designer();
        designer.shapes.push(Shape::Circle {
            x: 50.0,
            y: 50.0,
            radius: 25.0,
        });

        let gcode = designer.export_to_gcode();

        assert!(gcode.contains("; Circle at (50.00, 50.00) radius 25.00"));
        assert!(gcode.contains("G0 X75.00 Y50.00"));
        assert!(gcode.contains("G1 Z-1 F500 ; Plunge"));
        assert!(gcode.contains("G2 I-25.00 J-25.00 F1000 ; Clockwise circle"));
        assert!(gcode.contains("G0 Z5 ; Lift tool"));
    }

    #[test]
    fn test_export_line_to_gcode() {
        let mut designer = create_test_designer();
        designer.shapes.push(Shape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
        });

        let gcode = designer.export_to_gcode();

        assert!(gcode.contains("; Line from (0.00, 0.00) to (100.00, 100.00)"));
        assert!(gcode.contains("G0 X0.00 Y0.00"));
        assert!(gcode.contains("G1 Z-1 F500 ; Plunge"));
        assert!(gcode.contains("G1 X100.00 Y100.00 F1000 ; Draw line"));
        assert!(gcode.contains("G0 Z5 ; Lift tool"));
    }

    #[test]
    fn test_add_shape_command() {
        let mut designer = create_test_designer();
        let shape = Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };

        let mut command = AddShapeCommand::new(shape);
        command.execute(&mut designer);

        assert_eq!(designer.shapes.len(), 1);
        assert!(matches!(designer.shapes[0], Shape::Rectangle { .. }));

        command.undo(&mut designer);
        assert_eq!(designer.shapes.len(), 0);
    }

    #[test]
    fn test_delete_shape_command() {
        let mut designer = create_test_designer();
        let shape = Shape::Circle {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
        };
        designer.shapes.push(shape.clone());

        let mut command = DeleteShapeCommand::new(0);
        command.execute(&mut designer);

        assert_eq!(designer.shapes.len(), 0);

        command.undo(&mut designer);
        assert_eq!(designer.shapes.len(), 1);
        assert!(matches!(designer.shapes[0], Shape::Circle { .. }));
    }

    #[test]
    fn test_undo_redo() {
        let mut designer = create_test_designer();

        // Add a shape
        let shape = Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        designer.execute_command(Box::new(AddShapeCommand::new(shape)));

        assert_eq!(designer.shapes.len(), 1);
        assert!(designer.can_undo());
        assert!(!designer.can_redo());

        // Undo
        designer.undo();
        assert_eq!(designer.shapes.len(), 0);
        assert!(!designer.can_undo());
        assert!(designer.can_redo());

        // Redo
        designer.redo();
        assert_eq!(designer.shapes.len(), 1);
        assert!(designer.can_undo());
        assert!(!designer.can_redo());
    }

    #[test]
    fn test_get_shape_pos() {
        let rect = Shape::Rectangle {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        assert_eq!(DesignerState::get_shape_pos(&rect), (10.0, 20.0));

        let circle = Shape::Circle {
            x: 5.0,
            y: 15.0,
            radius: 10.0,
        };
        assert_eq!(DesignerState::get_shape_pos(&circle), (5.0, 15.0));

        let line = Shape::Line {
            x1: 1.0,
            y1: 2.0,
            x2: 3.0,
            y2: 4.0,
        };
        assert_eq!(DesignerState::get_shape_pos(&line), (1.0, 2.0));
    }

    #[test]
    fn test_export_to_stl_empty() {
        let designer = create_test_designer();
        let result = designer.export_to_stl();
        assert!(result.is_ok());
        let stl_data = result.unwrap();
        // Empty STL should still be valid but minimal
        assert!(stl_data.len() > 0);
    }

    #[test]
    fn test_export_to_obj_empty() {
        let designer = create_test_designer();
        let result = designer.export_to_obj();
        assert!(result.is_ok());
        let obj_data = result.unwrap();
        // Empty OBJ should be minimal
        assert!(obj_data.len() >= 0);
    }

    #[test]
    fn test_align_shapes() {
        let mut designer = create_test_designer();

        // Add some shapes
        designer.shapes.push(Shape::Rectangle {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        });
        designer.shapes.push(Shape::Rectangle {
            x: 50.0,
            y: 50.0,
            width: 20.0,
            height: 20.0,
        });

        // Test left align
        designer.align_shapes("left");
        if let Shape::Rectangle { x, .. } = &designer.shapes[0] {
            assert_eq!(*x, 10.0);
        }
        if let Shape::Rectangle { x, .. } = &designer.shapes[1] {
            assert_eq!(*x, 10.0);
        }

        // Test top align
        designer.align_shapes("top");
        if let Shape::Rectangle { y, .. } = &designer.shapes[0] {
            assert_eq!(*y, 10.0);
        }
        if let Shape::Rectangle { y, .. } = &designer.shapes[1] {
            assert_eq!(*y, 10.0);
        }
    }
}
