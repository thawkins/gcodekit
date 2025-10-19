use gcodekit::gcode::*;
use gcodekit::types::{MoveType, PathSegment};

#[cfg(test)]
mod gcode_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_empty_gcode() {
        let gcode = "";
        let paths = parse_gcode(gcode);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_parse_g0_move() {
        let gcode = "G0 X10 Y20 Z5";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].move_type, MoveType::Rapid);
        assert_eq!(paths[0].end_x, 10.0);
        assert_eq!(paths[0].end_y, 20.0);
        assert_eq!(paths[0].end_z, Some(5.0));
    }

    #[test]
    fn test_parse_g1_move() {
        let gcode = "G1 X30 Y40 F500";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].move_type, MoveType::Linear);
        assert_eq!(paths[0].end_x, 30.0);
        assert_eq!(paths[0].end_y, 40.0);
    }

    #[test]
    fn test_parse_g2_arc() {
        let gcode = "G2 X10 Y10 I5 J5";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].move_type, MoveType::ArcCW);
    }

    #[test]
    fn test_parse_g3_arc() {
        let gcode = "G3 X10 Y10 I5 J5";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].move_type, MoveType::ArcCCW);
    }

    #[test]
    fn test_parse_multiple_lines() {
        let gcode = "G0 X0 Y0\nG1 X10 Y10\nG1 X20 Y0\nG0 X0 Y0";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 4);
        assert_eq!(paths[0].move_type, MoveType::Rapid);
        assert_eq!(paths[1].move_type, MoveType::Linear);
        assert_eq!(paths[2].move_type, MoveType::Linear);
        assert_eq!(paths[3].move_type, MoveType::Rapid);
    }

    #[test]
    fn test_parse_with_comments() {
        let gcode = "G0 X10 Y20 ; Move to start\nG1 X30 Y40 ; Linear move";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_parse_with_empty_lines() {
        let gcode = "G0 X10 Y10\n\nG1 X20 Y20\n\n";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let gcode = "g0 x10 y20\nG1 X30 Y40";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_parse_with_f_parameter() {
        let gcode = "G1 X10 Y10 F1000";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        // F parameter is parsed but may not be stored in PathSegment
    }

    #[test]
    fn test_parse_with_z_axis() {
        let gcode = "G1 X10 Y10 Z5";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].end_z, Some(5.0));
    }

    #[test]
    fn test_parse_with_negative_coordinates() {
        let gcode = "G1 X-10 Y-20 Z-5";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].end_x, -10.0);
        assert_eq!(paths[0].end_y, -20.0);
        assert_eq!(paths[0].end_z, Some(-5.0));
    }

    #[test]
    fn test_parse_with_decimal_coordinates() {
        let gcode = "G1 X10.5 Y20.75 Z3.25";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].end_x, 10.5);
        assert_eq!(paths[0].end_y, 20.75);
        assert_eq!(paths[0].end_z, Some(3.25));
    }

    #[test]
    fn test_parse_rectangle() {
        let gcode = "G0 X0 Y0\nG1 X100 Y0\nG1 X100 Y50\nG1 X0 Y50\nG1 X0 Y0";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 5);

        // Check it forms a rectangle
        assert_eq!(paths[1].end_x, 100.0);
        assert_eq!(paths[1].end_y, 0.0);
        assert_eq!(paths[2].end_x, 100.0);
        assert_eq!(paths[2].end_y, 50.0);
        assert_eq!(paths[3].end_x, 0.0);
        assert_eq!(paths[3].end_y, 50.0);
        assert_eq!(paths[4].end_x, 0.0);
        assert_eq!(paths[4].end_y, 0.0);
    }

    #[test]
    fn test_parse_modal_commands() {
        // G1 is modal - should stay in linear mode
        let gcode = "G1 X10 Y10\nX20 Y20\nX30 Y30";
        let paths = parse_gcode(gcode);

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].move_type, MoveType::Linear);
        // Note: Modal behavior depends on parser implementation
    }
}

#[cfg(test)]
mod gcode_generation_tests {
    use super::*;

    #[test]
    fn test_generate_rectangle_gcode() {
        let gcode = generate_rectangle(100.0, 50.0, 500.0);

        assert!(gcode.contains("G21")); // Metric units
        assert!(gcode.contains("G90")); // Absolute positioning
        assert!(gcode.contains("G0 X0 Y0")); // Move to origin
        assert!(gcode.contains("G1 X100 Y0 F500")); // Bottom edge
        assert!(gcode.contains("G1 X100 Y50 F500")); // Right edge
        assert!(gcode.contains("G1 X0 Y50 F500")); // Top edge
        assert!(gcode.contains("G1 X0 Y0 F500")); // Left edge
        assert!(gcode.contains("M30")); // End program
    }

    #[test]
    fn test_generate_rectangle_different_sizes() {
        let gcode1 = generate_rectangle(50.0, 25.0, 300.0);
        let gcode2 = generate_rectangle(200.0, 100.0, 800.0);

        assert!(gcode1.contains("X50"));
        assert!(gcode1.contains("Y25"));
        assert!(gcode1.contains("F300"));

        assert!(gcode2.contains("X200"));
        assert!(gcode2.contains("Y100"));
        assert!(gcode2.contains("F800"));
    }

    #[test]
    fn test_generate_circle_gcode() {
        let gcode = generate_circle(25.0, 300.0);

        assert!(gcode.contains("G21")); // Metric units
        assert!(gcode.contains("G90")); // Absolute positioning
        assert!(gcode.contains("G0 X25 Y25")); // Move to center
        assert!(gcode.contains("G2 I-25 J-25 F300")); // Clockwise arc
        assert!(gcode.contains("M30")); // End program
    }

    #[test]
    fn test_generate_circle_different_radii() {
        let gcode1 = generate_circle(10.0, 200.0);
        let gcode2 = generate_circle(50.0, 500.0);

        assert!(gcode1.contains("X10 Y10"));
        assert!(gcode1.contains("I-10 J-10"));
        assert!(gcode1.contains("F200"));

        assert!(gcode2.contains("X50 Y50"));
        assert!(gcode2.contains("I-50 J-50"));
        assert!(gcode2.contains("F500"));
    }

    #[test]
    fn test_add_toolpath_parameters() {
        let base_gcode = "G1 X10 Y10\nG1 X20 Y20";
        let result = add_toolpath_parameters(base_gcode, 1000.0, 400.0);

        assert!(result.contains("G21")); // Metric units
        assert!(result.contains("M3 S1000")); // Spindle on
        assert!(result.contains("G1 F400")); // Set feed rate
        assert!(result.contains("G1 X10 Y10")); // Original move
        assert!(result.contains("G1 X20 Y20")); // Original move
        assert!(result.contains("M5")); // Spindle off
    }

    #[test]
    fn test_add_toolpath_parameters_empty_gcode() {
        let result = add_toolpath_parameters("", 1000.0, 400.0);

        assert!(result.contains("G21"));
        assert!(result.contains("M3 S1000"));
        assert!(result.contains("G1 F400"));
        assert!(result.contains("M5"));
    }
}

#[cfg(test)]
mod path_segment_tests {
    use super::*;

    #[test]
    fn test_path_segment_creation() {
        let segment = PathSegment {
            start_x: 0.0,
            start_y: 0.0,
            start_z: None,
            end_x: 10.0,
            end_y: 10.0,
            end_z: Some(5.0),
            move_type: MoveType::Linear,
            feed_rate: Some(500.0),
        };

        assert_eq!(segment.start_x, 0.0);
        assert_eq!(segment.start_y, 0.0);
        assert_eq!(segment.end_x, 10.0);
        assert_eq!(segment.end_y, 10.0);
        assert_eq!(segment.end_z, Some(5.0));
        assert_eq!(segment.move_type, MoveType::Linear);
        assert_eq!(segment.feed_rate, Some(500.0));
    }

    #[test]
    fn test_path_segment_length() {
        let segment = PathSegment {
            start_x: 0.0,
            start_y: 0.0,
            start_z: None,
            end_x: 3.0,
            end_y: 4.0,
            end_z: None,
            move_type: MoveType::Linear,
            feed_rate: None,
        };

        let length = segment.length();
        assert_eq!(length, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_path_segment_length_3d() {
        let segment = PathSegment {
            start_x: 0.0,
            start_y: 0.0,
            start_z: Some(0.0),
            end_x: 1.0,
            end_y: 0.0,
            end_z: Some(0.0),
            move_type: MoveType::Linear,
            feed_rate: None,
        };

        let length = segment.length();
        assert_eq!(length, 1.0);
    }
}

#[cfg(test)]
mod move_type_tests {
    use super::*;

    #[test]
    fn test_move_type_enum() {
        let types = vec![
            MoveType::Rapid,
            MoveType::Linear,
            MoveType::ArcCW,
            MoveType::ArcCCW,
        ];

        for move_type in types {
            let cloned = move_type.clone();
            assert_eq!(move_type, cloned);
        }
    }

    #[test]
    fn test_move_type_debug() {
        assert_eq!(format!("{:?}", MoveType::Rapid), "Rapid");
        assert_eq!(format!("{:?}", MoveType::Linear), "Linear");
        assert_eq!(format!("{:?}", MoveType::ArcCW), "ArcCW");
        assert_eq!(format!("{:?}", MoveType::ArcCCW), "ArcCCW");
    }
}

#[cfg(test)]
mod gcode_optimization_tests {
    use super::*;

    #[test]
    fn test_truncate_decimal_precision_basic() {
        let gcode = "G1 X10.123456 Y20.987654 Z5.555555";
        let result = truncate_decimal_precision(gcode, 3);

        assert!(result.contains("10.123"));
        assert!(result.contains("20.987"));
        assert!(result.contains("5.555"));
    }

    #[test]
    fn test_truncate_decimal_precision_multi_line() {
        let gcode = "G0 X10.123456 Y20.987654\nG1 X30.111111 Y40.222222";
        let result = truncate_decimal_precision(gcode, 2);

        assert!(result.contains("10.12"));
        assert!(result.contains("20.98"));
        assert!(result.contains("30.11"));
        assert!(result.contains("40.22"));
    }

    #[test]
    fn test_truncate_decimal_precision_preserves_comments() {
        let gcode = "G1 X10.123456 Y20.987654 ; This is a comment with 10.999999";
        let result = truncate_decimal_precision(gcode, 2);

        assert!(result.contains("10.12"));
        assert!(result.contains("20.98"));
        // Comment should be preserved
        assert!(result.contains("; This is a comment"));
    }

    #[test]
    fn test_truncate_decimal_precision_zero_decimals() {
        let gcode = "G1 X10.123456 Y20.987654";
        let result = truncate_decimal_precision(gcode, 0);

        assert!(result.contains("X10") || result.contains("X10."));
        assert!(result.contains("Y20") || result.contains("Y20."));
    }

    #[test]
    fn test_remove_redundant_whitespace_basic() {
        let gcode = "G1   X10    Y20    Z5";
        let result = remove_redundant_whitespace(gcode);

        assert_eq!(result.trim(), "G1 X10 Y20 Z5");
    }

    #[test]
    fn test_remove_redundant_whitespace_multi_line() {
        let gcode = "G0   X0   Y0  \nG1    X10   Y10   F500\n\nG1  X20  Y20";
        let result = remove_redundant_whitespace(gcode);

        // Should remove excessive whitespace but preserve structure
        assert!(result.contains("G0 X0 Y0"));
        assert!(result.contains("G1 X10 Y10 F500"));
    }

    #[test]
    fn test_remove_redundant_whitespace_preserves_comments() {
        let gcode = "G1 X10 Y10 ; Move   to   position";
        let result = remove_redundant_whitespace(gcode);

        assert!(result.contains("; Move"));
    }

    #[test]
    fn test_remove_redundant_whitespace_removes_empty_lines() {
        let gcode = "G1 X10 Y10\n\n\nG1 X20 Y20\n\n";
        let result = remove_redundant_whitespace(gcode);

        let lines: Vec<&str> = result.lines().collect();
        // Empty lines should be removed or consolidated
        assert!(lines.len() <= 2);
    }

    #[test]
    fn test_convert_arcs_to_lines_basic() {
        let gcode = "G90\nG0 X10 Y10\nG2 X20 Y20 I5 J0 F500";
        let result = convert_arcs_to_lines(gcode, 0.05);

        // Should replace G2 with G1 segments
        assert!(result.contains("G1"));
        assert!(!result.contains("G2"));
    }

    #[test]
    fn test_convert_arcs_to_lines_preserves_non_arc_moves() {
        let gcode = "G0 X0 Y0\nG1 X10 Y10\nG2 X20 Y20 I5 J0\nG1 X30 Y30";
        let result = convert_arcs_to_lines(gcode, 0.05);

        assert!(result.contains("G0 X0 Y0"));
        assert!(result.contains("G1 X10 Y10"));
        assert!(result.contains("G1 X30 Y30"));
    }

    #[test]
    fn test_convert_arcs_to_lines_maintains_feed_rate() {
        let gcode = "G90\nG2 X20 Y20 I10 J0 F1000";
        let result = convert_arcs_to_lines(gcode, 0.05);

        // Feed rate should be maintained in converted segments
        assert!(result.contains("F1000"));
    }

    #[test]
    fn test_convert_arcs_to_lines_g3_counterclockwise() {
        let gcode = "G90\nG0 X10 Y10\nG3 X20 Y20 I5 J0";
        let result = convert_arcs_to_lines(gcode, 0.05);

        // Should convert G3 (CCW arc) to line segments
        assert!(result.contains("G1"));
        assert!(!result.contains("G3"));
    }

    #[test]
    fn test_optimization_combination() {
        let gcode = "G90\nG1   X10.12345   Y20.98765   F1000\n\nG1  X30.11111  Y40.22222";
        let result = remove_redundant_whitespace(gcode);
        let result = truncate_decimal_precision(&result, 2);

        // Should have both optimizations applied
        assert!(result.contains("10.12"));
        assert!(result.contains("20.98"));
        assert!(result.contains("30.11"));
        assert!(result.contains("40.22"));
        // Should not have excessive whitespace
        assert!(!result.contains("   "));
    }

    #[test]
    fn test_optimization_empty_gcode() {
        let gcode = "";
        let result1 = remove_redundant_whitespace(gcode);
        let result2 = truncate_decimal_precision(gcode, 3);
        let result3 = convert_arcs_to_lines(gcode, 0.05);

        assert_eq!(result1.trim(), "");
        assert_eq!(result2.trim(), "");
        assert_eq!(result3.trim(), "");
    }

    #[test]
    fn test_optimization_only_comments() {
        let gcode = "; This is a comment\n; Another comment";
        let result = remove_redundant_whitespace(gcode);

        // Comments should be preserved
        assert!(result.contains("; This is a comment"));
        assert!(result.contains("; Another comment"));
    }

    #[test]
    fn test_decimal_precision_preserves_g_codes() {
        let gcode = "G90\nG21\nG1 X10.123 Y20.456";
        let result = truncate_decimal_precision(gcode, 1);

        assert!(result.contains("G90"));
        assert!(result.contains("G21"));
        assert!(result.contains("G1"));
    }

    #[test]
    fn test_arc_conversion_with_negative_coords() {
        let gcode = "G90\nG0 X-10 Y-10\nG2 X-20 Y-20 I-5 J0";
        let result = convert_arcs_to_lines(gcode, 0.05);

        assert!(result.contains("G1"));
        // Negative coordinates should be preserved
        assert!(result.contains("-"));
    }
}
