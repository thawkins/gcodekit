use anyhow::Result;
use eframe::egui;
use image::GrayImage;
use lyon::path::Path;
use lyon::math::point;
use lyon::tessellation::{FillTessellator, FillOptions};
use lyon::tessellation::geometry_builder::{simple_builder, VertexBuffers};
use rhai::{AST, Engine, Scope};
use std::collections::VecDeque;
use std::fs;

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
    pub material: String,
    pub flute_count: u32,
    pub max_rpm: u32,
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            name: "End Mill 3mm".to_string(),
            diameter: 3.0,
            material: "HSS".to_string(),
            flute_count: 2,
            max_rpm: 10000,
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

#[derive(Default)]
pub struct DesignerState {
    pub shapes: Vec<Shape>,
    pub current_tool: DrawingTool,
    pub current_pattern: ToolpathPattern,
    pub current_material: Material,
    pub current_tool_def: Tool,
    pub drawing_start: Option<(f32, f32)>,
    pub selected_shape: Option<usize>,
    undo_stack: VecDeque<Box<dyn Command>>,
    redo_stack: VecDeque<Box<dyn Command>>,
    drag_start_pos: Option<(f32, f32)>,
    show_grid: bool,
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
                                    let rad = angle.to_radians();
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
                    ast,
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

                ui.menu_button("Boolean", |ui| {
                    if ui.button("Union").clicked() {
                        if self.shapes.len() >= 2 {
                            let all_indices: Vec<usize> = (0..self.shapes.len()).collect();
                            if let Err(e) = self.boolean_union(&all_indices) {
                                tracing::error!("Boolean union failed: {}", e);
                            }
                            self.selected_shape = None;
                        }
                        ui.close_menu();
                    }
                    if ui.button("Intersect").clicked() {
                        // TODO: Implement intersect
                        ui.close_menu();
                    }
                    if ui.button("Subtract").clicked() {
                        // TODO: Implement subtract
                        ui.close_menu();
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
                        for (i, shape) in self.shapes.iter().enumerate().rev() {
                            if self.shape_contains_point(shape, canvas_pos) {
                                self.selected_shape = Some(i);
                                break;
                            }
                        }
                    }
                    DrawingTool::Polyline | DrawingTool::Parametric | _ => {
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
                            // For polyline, we need to collect multiple points
                            // For now, create a simple line
                            Shape::Polyline {
                                points: vec![(start.0, start.1), (end_canvas.x, end_canvas.y)],
                            }
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
                        let start_pos = egui::pos2(rect.min.x + start.0, rect.min.y + start.1);
                        let end_pos = egui::pos2(
                            rect.min.x + current_canvas.x,
                            rect.min.y + current_canvas.y,
                        );
                        painter.line_segment(
                            [start_pos, end_pos],
                            egui::Stroke::new(1.0, egui::Color32::BLUE),
                        );
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

    pub fn import_bitmap(&mut self, path: &std::path::Path) -> Result<()> {
        let img = image::open(path)?;
        let gray = img.to_luma8();
        let points = self.vectorize_bitmap(&gray);
        self.shapes.push(Shape::Polyline { points });
        Ok(())
    }

    fn shape_to_path(&self, shape: &Shape) -> Option<Path> {
        use lyon::path::builder::*;
        let mut builder = Path::builder();

        match shape {
            Shape::Rectangle { x, y, width, height } => {
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
                    Shape::Rectangle { x, y, width, height } => {
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
        // Simple edge detection and vectorization
        // This is a basic implementation - for production, use a proper vectorization library
        let mut points = Vec::new();
        let (width, height) = img.dimensions();

        // Simple threshold-based edge detection
        let threshold = 128u8;

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y)[0];
                if pixel < threshold {
                    // Dark pixel, add as point
                    points.push((x as f32, y as f32));
                }
            }
        }

        // Simplify by connecting nearby points
        if points.len() > 1 {
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let mut simplified = vec![points[0]];
            for &point in &points[1..] {
                let last = *simplified.last().unwrap();
                if (point.0 - last.0).abs() > 2.0 || (point.1 - last.1).abs() > 2.0 {
                    simplified.push(point);
                }
            }
            simplified
        } else {
            points
        }
    }

    pub fn evaluate_parametric(&mut self, index: usize) -> Result<()> {
        if let Some(Shape::Parametric {
            script,
            ast,
            bounds,
        }) = self.shapes.get_mut(index)
        {
            let engine = Engine::new();

            // Compile the script if not already done
            if ast.is_none() {
                *ast = Some(engine.compile(script)?);
            }

            // Generate points by evaluating the script with different t values
            let mut points = Vec::new();
            let steps = 100; // Number of points to generate

            for i in 0..steps {
                let mut scope = Scope::new();
                let t = i as f64 / (steps - 1) as f64;

                scope.push("t", t);
                scope.push("width", (bounds.2 - bounds.0) as f64);
                scope.push("height", (bounds.3 - bounds.1) as f64);

                // TODO: fix rhai eval
                // let _result: rhai::Dynamic = match engine.eval_ast_with_scope(ast.as_ref().unwrap(), &mut scope) {
                //     Ok(result) => result,
                //     Err(_) => return Err(anyhow::anyhow!("Rhai evaluation error")),
                // };

                // Extract x, y from scope
                if let (Some(x), Some(y)) =
                    (scope.get_value::<f64>("x"), scope.get_value::<f64>("y"))
                {
                    let scaled_x = bounds.0 + (x as f32 / 100.0) * (bounds.2 - bounds.0);
                    let scaled_y = bounds.1 + (y as f32 / 100.0) * (bounds.3 - bounds.1);
                    points.push((scaled_x, scaled_y));
                }
            }

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
}
