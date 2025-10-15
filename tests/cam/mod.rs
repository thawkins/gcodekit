use gcodekit::cam::*;

#[cfg(test)]
mod cam_operation_tests {
    use super::*;

    #[test]
    fn test_cam_operation_enum() {
        let operations = vec![
            CAMOperation::Pocket,
            CAMOperation::Profile,
            CAMOperation::Drill,
            CAMOperation::Engrave,
            CAMOperation::VCarve,
        ];

        for op in operations {
            // Test clone
            let cloned = op.clone();
            assert_eq!(op, cloned);

            // Test debug
            let debug_str = format!("{:?}", op);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_cam_parameters_default() {
        let params = CAMParameters::default();

        assert_eq!(params.tool_diameter, 3.0);
        assert_eq!(params.stepover_percentage, 40.0);
        assert_eq!(params.stepdown, 1.0);
        assert_eq!(params.feed_rate, 100.0);
        assert_eq!(params.plunge_rate, 50.0);
        assert_eq!(params.spindle_speed, 10000.0);
        assert_eq!(params.cut_depth, 5.0);
        assert!(params.climb_milling);
    }

    #[test]
    fn test_cam_parameters_builder() {
        let params = CAMParameters {
            tool_diameter: 6.0,
            stepover_percentage: 50.0,
            stepdown: 2.0,
            feed_rate: 200.0,
            plunge_rate: 75.0,
            spindle_speed: 15000.0,
            cut_depth: 10.0,
            climb_milling: false,
        };

        assert_eq!(params.tool_diameter, 6.0);
        assert_eq!(params.stepover_percentage, 50.0);
        assert_eq!(params.stepdown, 2.0);
        assert_eq!(params.feed_rate, 200.0);
        assert_eq!(params.plunge_rate, 75.0);
        assert_eq!(params.spindle_speed, 15000.0);
        assert_eq!(params.cut_depth, 10.0);
        assert!(!params.climb_milling);
    }

    #[test]
    fn test_cam_parameters_clone() {
        let params1 = CAMParameters::default();
        let params2 = params1.clone();

        assert_eq!(params1.tool_diameter, params2.tool_diameter);
        assert_eq!(params1.stepover_percentage, params2.stepover_percentage);
        assert_eq!(params1.stepdown, params2.stepdown);
        assert_eq!(params1.feed_rate, params2.feed_rate);
    }

    #[test]
    fn test_stepover_calculation() {
        let params = CAMParameters {
            tool_diameter: 10.0,
            stepover_percentage: 50.0,
            ..Default::default()
        };

        let stepover = params.tool_diameter * (params.stepover_percentage / 100.0);
        assert_eq!(stepover, 5.0);
    }

    #[test]
    fn test_cam_parameters_validation() {
        let params = CAMParameters {
            tool_diameter: 3.0,
            stepover_percentage: 40.0,
            stepdown: 1.0,
            feed_rate: 100.0,
            plunge_rate: 50.0,
            spindle_speed: 10000.0,
            cut_depth: 5.0,
            climb_milling: true,
        };

        // Validate positive values
        assert!(params.tool_diameter > 0.0);
        assert!(params.stepover_percentage > 0.0);
        assert!(params.stepover_percentage <= 100.0);
        assert!(params.stepdown > 0.0);
        assert!(params.feed_rate > 0.0);
        assert!(params.plunge_rate > 0.0);
        assert!(params.spindle_speed > 0.0);
        assert!(params.cut_depth > 0.0);
    }

    #[test]
    fn test_plunge_rate_slower_than_feed_rate() {
        let params = CAMParameters::default();
        assert!(params.plunge_rate <= params.feed_rate);
    }
}

#[cfg(test)]
mod toolpath_tests {
    use super::*;

    #[test]
    fn test_toolpath_types() {
        // Test that different operations can be created
        let operations = vec![
            CAMOperation::Pocket,
            CAMOperation::Profile,
            CAMOperation::Drill,
            CAMOperation::Engrave,
            CAMOperation::VCarve,
        ];

        for op in operations {
            assert!(matches!(
                op,
                CAMOperation::Pocket
                    | CAMOperation::Profile
                    | CAMOperation::Drill
                    | CAMOperation::Engrave
                    | CAMOperation::VCarve
            ));
        }
    }

    #[test]
    fn test_cam_operation_debug_formatting() {
        assert_eq!(format!("{:?}", CAMOperation::Pocket), "Pocket");
        assert_eq!(format!("{:?}", CAMOperation::Profile), "Profile");
        assert_eq!(format!("{:?}", CAMOperation::Drill), "Drill");
        assert_eq!(format!("{:?}", CAMOperation::Engrave), "Engrave");
        assert_eq!(format!("{:?}", CAMOperation::VCarve), "VCarve");
    }
}

#[cfg(test)]
mod nesting_tests {
    use gcodekit::cam::nesting::*;

    #[test]
    fn test_rectangle_creation() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            rotation: 0.0,
        };

        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
        assert_eq!(rect.rotation, 0.0);
    }

    #[test]
    fn test_rectangle_area() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 20.0,
            rotation: 0.0,
        };

        let area = rect.area();
        assert_eq!(area, 200.0);
    }

    #[test]
    fn test_rectangle_overlaps_no_overlap() {
        let rect1 = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        let rect2 = Rectangle {
            x: 20.0,
            y: 20.0,
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        assert!(!rect1.overlaps(&rect2));
        assert!(!rect2.overlaps(&rect1));
    }

    #[test]
    fn test_rectangle_overlaps_with_overlap() {
        let rect1 = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 15.0,
            height: 15.0,
            rotation: 0.0,
        };

        let rect2 = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 15.0,
            height: 15.0,
            rotation: 0.0,
        };

        assert!(rect1.overlaps(&rect2));
        assert!(rect2.overlaps(&rect1));
    }

    #[test]
    fn test_rectangle_touching_edges_not_overlap() {
        let rect1 = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        let rect2 = Rectangle {
            x: 10.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            rotation: 0.0,
        };

        // Touching edges should not be considered overlapping
        assert!(!rect1.overlaps(&rect2));
    }

    #[test]
    fn test_nested_layout_creation() {
        let layout = NestedLayout {
            sheet_width: 1000.0,
            sheet_height: 600.0,
            parts: vec![],
            utilization: 0.0,
        };

        assert_eq!(layout.sheet_width, 1000.0);
        assert_eq!(layout.sheet_height, 600.0);
        assert!(layout.parts.is_empty());
        assert_eq!(layout.utilization, 0.0);
    }

    #[test]
    fn test_utilization_calculation() {
        let parts = vec![
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                rotation: 0.0,
            },
            Rectangle {
                x: 100.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                rotation: 0.0,
            },
        ];

        let layout = NestedLayout {
            sheet_width: 1000.0,
            sheet_height: 1000.0,
            parts: parts.clone(),
            utilization: 0.0,
        };

        let total_part_area: f32 = parts.iter().map(|p| p.area()).sum();
        let sheet_area = layout.sheet_width * layout.sheet_height;
        let expected_utilization = (total_part_area / sheet_area) * 100.0;

        assert_eq!(expected_utilization, 2.0); // 20000 / 1000000 * 100
    }

    #[test]
    fn test_nest_parts_empty() {
        let parts = vec![];
        let result = nest_parts(&parts, 1000.0, 600.0, 5.0);

        assert!(result.parts.is_empty());
        assert_eq!(result.utilization, 0.0);
    }

    #[test]
    fn test_nest_parts_single_part() {
        let parts = vec![Rectangle {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            rotation: 0.0,
        }];

        let result = nest_parts(&parts, 1000.0, 600.0, 5.0);

        assert_eq!(result.parts.len(), 1);
        assert!(result.utilization > 0.0);
        assert!(result.parts[0].x >= 0.0);
        assert!(result.parts[0].y >= 0.0);
    }

    #[test]
    fn test_nest_parts_multiple_parts() {
        let parts = vec![
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                rotation: 0.0,
            },
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 80.0,
                rotation: 0.0,
            },
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 60.0,
                height: 60.0,
                rotation: 0.0,
            },
        ];

        let result = nest_parts(&parts, 1000.0, 600.0, 5.0);

        assert_eq!(result.parts.len(), 3);
        assert!(result.utilization > 0.0);

        // Check no parts overlap
        for (i, part1) in result.parts.iter().enumerate() {
            for part2 in result.parts.iter().skip(i + 1) {
                assert!(!part1.overlaps(part2), "Parts should not overlap");
            }
        }
    }

    #[test]
    fn test_nest_parts_with_spacing() {
        let parts = vec![
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
                rotation: 0.0,
            },
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
                rotation: 0.0,
            },
        ];

        let result = nest_parts(&parts, 1000.0, 600.0, 10.0);

        // Verify spacing between parts
        if result.parts.len() == 2 {
            let part1 = &result.parts[0];
            let part2 = &result.parts[1];

            let horizontal_distance = (part2.x - (part1.x + part1.width)).abs();
            let vertical_distance = (part2.y - (part1.y + part1.height)).abs();

            // At least one distance should be >= spacing
            assert!(horizontal_distance >= 9.9 || vertical_distance >= 9.9);
        }
    }

    #[test]
    fn test_nest_parts_too_large() {
        let parts = vec![Rectangle {
            x: 0.0,
            y: 0.0,
            width: 2000.0, // Larger than sheet
            height: 100.0,
            rotation: 0.0,
        }];

        let result = nest_parts(&parts, 1000.0, 600.0, 5.0);

        // Should still attempt to place it (implementation dependent)
        assert_eq!(result.parts.len(), 1);
    }
}
