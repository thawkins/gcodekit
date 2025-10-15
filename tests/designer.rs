use egui;
use gcodekit::cam::types::{CAMOperation, CAMParameters};
use gcodekit::designer::Command;
use gcodekit::designer::{AddShapeCommand, DeleteShapeCommand};
use gcodekit::designer::{DesignerState, DrawingTool, Material, Shape, Tool, ToolpathPattern};
use std::collections::VecDeque;

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
        selected_cam_operation: CAMOperation::default(),
        cam_params: CAMParameters::default(),
        viewport_size: egui::vec2(800.0, 600.0),
        shape_width: 100.0,
        shape_height: 100.0,
        shape_radius: 50.0,
        stroke_width: 1.0,
        calligraphy_angle: 0.0,
        text_font_size: 12.0,
        current_mesh: None,
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
    assert!(obj_data.len() > 0);
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
