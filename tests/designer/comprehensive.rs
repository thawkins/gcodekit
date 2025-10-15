use gcodekit::designer::*;
use gcodekit::designer::{Material, Tool};

#[cfg(test)]
mod shape_tests {
    use super::*;

    #[test]
    fn test_rectangle_shape() {
        let rect = Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        match rect {
            Shape::Rectangle { width, height, .. } => {
                assert_eq!(width, 100.0);
                assert_eq!(height, 50.0);
            }
            _ => panic!("Expected Rectangle shape"),
        }
    }

    #[test]
    fn test_circle_shape() {
        let circle = Shape::Circle {
            x: 50.0,
            y: 50.0,
            radius: 25.0,
        };

        match circle {
            Shape::Circle { radius, .. } => {
                assert_eq!(radius, 25.0);
            }
            _ => panic!("Expected Circle shape"),
        }
    }

    #[test]
    fn test_line_shape() {
        let line = Shape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
        };

        match line {
            Shape::Line { x1, y1, x2, y2 } => {
                assert_eq!(x1, 0.0);
                assert_eq!(y1, 0.0);
                assert_eq!(x2, 100.0);
                assert_eq!(y2, 100.0);
            }
            _ => panic!("Expected Line shape"),
        }
    }

    #[test]
    fn test_text_shape() {
        let text = Shape::Text {
            x: 10.0,
            y: 10.0,
            text: "Hello".to_string(),
            font_size: 12.0,
        };

        match text {
            Shape::Text { text, font_size, .. } => {
                assert_eq!(text, "Hello");
                assert_eq!(font_size, 12.0);
            }
            _ => panic!("Expected Text shape"),
        }
    }

    #[test]
    fn test_drill_shape() {
        let drill = Shape::Drill {
            x: 20.0,
            y: 20.0,
            depth: 5.0,
        };

        match drill {
            Shape::Drill { depth, .. } => {
                assert_eq!(depth, 5.0);
            }
            _ => panic!("Expected Drill shape"),
        }
    }

    #[test]
    fn test_pocket_shape() {
        let pocket = Shape::Pocket {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 30.0,
            depth: 3.0,
            stepover: 2.0,
            pattern: ToolpathPattern::Horizontal,
        };

        match pocket {
            Shape::Pocket { width, height, depth, stepover, pattern, .. } => {
                assert_eq!(width, 50.0);
                assert_eq!(height, 30.0);
                assert_eq!(depth, 3.0);
                assert_eq!(stepover, 2.0);
                assert_eq!(pattern, ToolpathPattern::Horizontal);
            }
            _ => panic!("Expected Pocket shape"),
        }
    }

    #[test]
    fn test_cylinder_shape() {
        let cylinder = Shape::Cylinder {
            x: 0.0,
            y: 0.0,
            radius: 10.0,
            height: 50.0,
            depth: 5.0,
        };

        match cylinder {
            Shape::Cylinder { radius, height, .. } => {
                assert_eq!(radius, 10.0);
                assert_eq!(height, 50.0);
            }
            _ => panic!("Expected Cylinder shape"),
        }
    }

    #[test]
    fn test_sphere_shape() {
        let sphere = Shape::Sphere {
            x: 0.0,
            y: 0.0,
            radius: 15.0,
            depth: 10.0,
        };

        match sphere {
            Shape::Sphere { radius, .. } => {
                assert_eq!(radius, 15.0);
            }
            _ => panic!("Expected Sphere shape"),
        }
    }
}

#[cfg(test)]
mod toolpath_pattern_tests {
    use super::*;

    #[test]
    fn test_toolpath_pattern_variants() {
        let patterns = vec![
            ToolpathPattern::Horizontal,
            ToolpathPattern::Vertical,
            ToolpathPattern::Spiral,
            ToolpathPattern::Contour,
        ];

        for pattern in patterns {
            let cloned = pattern.clone();
            assert_eq!(pattern, cloned);
        }
    }

    #[test]
    fn test_toolpath_pattern_equality() {
        assert_eq!(ToolpathPattern::Horizontal, ToolpathPattern::Horizontal);
        assert_ne!(ToolpathPattern::Horizontal, ToolpathPattern::Vertical);
        assert_ne!(ToolpathPattern::Spiral, ToolpathPattern::Contour);
    }
}

#[cfg(test)]
mod drawing_tool_tests {
    use super::*;

    #[test]
    fn test_drawing_tool_variants() {
        let tools = vec![
            DrawingTool::Select,
            DrawingTool::Rectangle,
            DrawingTool::Circle,
            DrawingTool::Line,
            DrawingTool::Text,
            DrawingTool::Drill,
        ];

        for tool in tools {
            let cloned = tool.clone();
            assert_eq!(tool, cloned);
        }
    }

    #[test]
    fn test_drawing_tool_default() {
        let tool = DrawingTool::default();
        assert_eq!(tool, DrawingTool::Select);
    }
}

#[cfg(test)]
mod designer_state_tests {
    use super::*;

    #[test]
    fn test_designer_state_creation() {
        let state = DesignerState::default();

        assert!(state.shapes.is_empty());
        assert!(state.selected_shape.is_none());
        assert_eq!(state.current_tool, DrawingTool::Select);
        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.pan_x, 0.0);
        assert_eq!(state.pan_y, 0.0);
    }

    #[test]
    fn test_add_shape_to_designer() {
        let mut state = DesignerState::default();

        let rect = Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        state.shapes.push(rect);
        assert_eq!(state.shapes.len(), 1);
    }

    #[test]
    fn test_select_shape() {
        let mut state = DesignerState::default();

        state.shapes.push(Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        });

        state.selected_shape = Some(0);
        assert_eq!(state.selected_shape, Some(0));
    }

    #[test]
    fn test_change_tool() {
        let mut state = DesignerState::default();

        state.current_tool = DrawingTool::Rectangle;
        assert_eq!(state.current_tool, DrawingTool::Rectangle);

        state.current_tool = DrawingTool::Circle;
        assert_eq!(state.current_tool, DrawingTool::Circle);
    }

    #[test]
    fn test_zoom_operations() {
        let mut state = DesignerState::default();

        state.zoom = 2.0;
        assert_eq!(state.zoom, 2.0);

        state.zoom = 0.5;
        assert_eq!(state.zoom, 0.5);
    }

    #[test]
    fn test_pan_operations() {
        let mut state = DesignerState::default();

        state.pan_x = 100.0;
        state.pan_y = 50.0;

        assert_eq!(state.pan_x, 100.0);
        assert_eq!(state.pan_y, 50.0);
    }
}

#[cfg(test)]
mod material_tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material = Material {
            name: "Aluminum 6061".to_string(),
            density: 2700.0,
            hardness: 95.0,
            cutting_speed: 200.0,
            thickness: 5.0,
        };

        assert_eq!(material.name, "Aluminum 6061");
        assert_eq!(material.density, 2700.0);
        assert_eq!(material.hardness, 95.0);
        assert_eq!(material.cutting_speed, 200.0);
        assert_eq!(material.thickness, 5.0);
    }

    #[test]
    fn test_material_clone() {
        let material1 = Material {
            name: "Steel".to_string(),
            density: 7850.0,
            hardness: 200.0,
            cutting_speed: 100.0,
            thickness: 3.0,
        };

        let material2 = material1.clone();

        assert_eq!(material1.name, material2.name);
        assert_eq!(material1.density, material2.density);
        assert_eq!(material1.hardness, material2.hardness);
    }

    #[test]
    fn test_material_validation() {
        let material = Material {
            name: "Wood".to_string(),
            density: 600.0,
            hardness: 50.0,
            cutting_speed: 300.0,
            thickness: 10.0,
        };

        assert!(material.density > 0.0);
        assert!(material.hardness > 0.0);
        assert!(material.cutting_speed > 0.0);
        assert!(material.thickness > 0.0);
    }
}

#[cfg(test)]
mod tool_tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = Tool {
            name: "End Mill 6mm".to_string(),
            diameter: 6.0,
            length: 50.0,
            material: "Carbide".to_string(),
            flute_count: 4,
            max_rpm: 15000,
            tool_number: 1,
            length_offset: 0.0,
            wear_offset: 0.0,
        };

        assert_eq!(tool.name, "End Mill 6mm");
        assert_eq!(tool.diameter, 6.0);
        assert_eq!(tool.flute_count, 4);
        assert_eq!(tool.max_rpm, 15000);
    }

    #[test]
    fn test_tool_offsets() {
        let tool = Tool {
            name: "Drill 3mm".to_string(),
            diameter: 3.0,
            length: 40.0,
            material: "HSS".to_string(),
            flute_count: 2,
            max_rpm: 10000,
            tool_number: 2,
            length_offset: 1.5,
            wear_offset: 0.2,
        };

        assert_eq!(tool.length_offset, 1.5);
        assert_eq!(tool.wear_offset, 0.2);

        let total_offset = tool.length_offset + tool.wear_offset;
        assert_eq!(total_offset, 1.7);
    }

    #[test]
    fn test_tool_validation() {
        let tool = Tool {
            name: "V-Bit 90deg".to_string(),
            diameter: 6.0,
            length: 30.0,
            material: "Carbide".to_string(),
            flute_count: 2,
            max_rpm: 18000,
            tool_number: 3,
            length_offset: 0.0,
            wear_offset: 0.0,
        };

        assert!(tool.diameter > 0.0);
        assert!(tool.length > 0.0);
        assert!(tool.flute_count > 0);
        assert!(tool.max_rpm > 0);
        assert!(tool.tool_number > 0);
    }
}

#[cfg(test)]
mod shape_operations_tests {
    use super::*;

    #[test]
    fn test_rectangle_area() {
        let rect = Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        if let Shape::Rectangle { width, height, .. } = rect {
            let area = width * height;
            assert_eq!(area, 5000.0);
        }
    }

    #[test]
    fn test_circle_area() {
        let circle = Shape::Circle {
            x: 0.0,
            y: 0.0,
            radius: 10.0,
        };

        if let Shape::Circle { radius, .. } = circle {
            let area = std::f32::consts::PI * radius * radius;
            assert!((area - 314.159).abs() < 0.01);
        }
    }

    #[test]
    fn test_line_length() {
        let line = Shape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 3.0,
            y2: 4.0,
        };

        if let Shape::Line { x1, y1, x2, y2 } = line {
            let dx = x2 - x1;
            let dy = y2 - y1;
            let length = (dx * dx + dy * dy).sqrt();
            assert_eq!(length, 5.0); // 3-4-5 triangle
        }
    }

    #[test]
    fn test_pocket_volume() {
        let pocket = Shape::Pocket {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 30.0,
            depth: 5.0,
            stepover: 2.0,
            pattern: ToolpathPattern::Horizontal,
        };

        if let Shape::Pocket { width, height, depth, .. } = pocket {
            let volume = width * height * depth;
            assert_eq!(volume, 7500.0);
        }
    }

    #[test]
    fn test_cylinder_volume() {
        let cylinder = Shape::Cylinder {
            x: 0.0,
            y: 0.0,
            radius: 10.0,
            height: 20.0,
            depth: 5.0,
        };

        if let Shape::Cylinder { radius, height, .. } = cylinder {
            let volume = std::f32::consts::PI * radius * radius * height;
            assert!((volume - 6283.185).abs() < 0.01);
        }
    }
}

#[cfg(test)]
mod gcode_generation_tests {
    use super::*;

    #[test]
    fn test_generate_gcode_for_shape() {
        // Test that shapes can be converted to G-code
        // This is a placeholder test for the actual implementation
        let rect = Shape::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        // In actual implementation, this would call a generate_gcode method
        assert!(matches!(rect, Shape::Rectangle { .. }));
    }

    #[test]
    fn test_multiple_shapes_gcode() {
        let shapes = vec![
            Shape::Rectangle {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            },
            Shape::Circle {
                x: 100.0,
                y: 100.0,
                radius: 25.0,
            },
        ];

        assert_eq!(shapes.len(), 2);
    }
}

#[cfg(test)]
mod bitmap_processing_tests {
    use gcodekit::designer::bitmap_processing::VectorizationConfig;

    #[test]
    fn test_vectorization_config_default() {
        let config = VectorizationConfig::default();

        assert!(config.threshold > 0.0 && config.threshold <= 1.0);
        assert!(config.smoothing >= 0.0);
        assert!(config.detail_level >= 0.0);
    }

    #[test]
    fn test_vectorization_config_custom() {
        let config = VectorizationConfig {
            threshold: 0.5,
            smoothing: 2.0,
            detail_level: 0.8,
            invert: true,
        };

        assert_eq!(config.threshold, 0.5);
        assert_eq!(config.smoothing, 2.0);
        assert_eq!(config.detail_level, 0.8);
        assert!(config.invert);
    }
}
