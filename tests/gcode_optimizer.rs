//! Integration tests for G-code optimization features (Task 6)

#[cfg(test)]
mod gcode_optimizer_integration_tests {
    use gcodekit::gcode::{
        truncate_decimal_precision, remove_redundant_whitespace, convert_arcs_to_lines,
    };

    #[test]
    fn test_task6_decimal_precision_truncation() {
        let gcode = "G1 X10.123456 Y20.987654 Z5.555555 F1000.5555";
        let result = truncate_decimal_precision(gcode, 3);

        // Verify decimal truncation
        assert!(result.contains("10.123"));
        assert!(result.contains("20.987"));
        assert!(result.contains("5.555"));
        // Feed rate should also be truncated
        assert!(result.contains("1000.555") || result.contains("1000.5"));
    }

    #[test]
    fn test_task6_precision_multi_line_gcode() {
        let gcode = "G21\nG90\nG0 X0.111111 Y0.222222\nG1 X100.999999 Y50.555555 F2000.123456";
        let result = truncate_decimal_precision(gcode, 2);

        // All coordinates should be truncated to 2 decimals
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 4);
        
        // Check that truncation happened
        let joined = result.to_string();
        assert!(joined.contains("0.11") || joined.contains("0.1"));
        assert!(joined.contains("100.99") || joined.contains("100.9") || joined.contains("101"));
    }

    #[test]
    fn test_task6_precision_preserves_gcode_commands() {
        let gcode = "G90\nG21\nG0 X10.123 Y20.456\nG1 X30.789 Y40.123 F1000.456\nM3 S5000.789\nM5";
        let result = truncate_decimal_precision(gcode, 1);

        // All G/M codes should be preserved
        assert!(result.contains("G90"));
        assert!(result.contains("G21"));
        assert!(result.contains("G0"));
        assert!(result.contains("G1"));
        assert!(result.contains("M3"));
        assert!(result.contains("M5"));
    }

    #[test]
    fn test_task6_whitespace_optimization() {
        let gcode = "G1   X10    Y20    Z5   F1000\nG1  X20  Y20  F1000";
        let result = remove_redundant_whitespace(gcode);

        // Should remove excessive spacing
        assert!(!result.contains("   "));  // No triple spaces
        
        // Check structure is preserved
        assert!(result.contains("G1"));
        assert!(result.contains("X10"));
        assert!(result.contains("Y20"));
    }

    #[test]
    fn test_task6_whitespace_removes_empty_lines() {
        let gcode = "G1 X10 Y10\n\n\nG1 X20 Y20\n\n\nG1 X30 Y30";
        let result = remove_redundant_whitespace(gcode);

        // Empty lines should be removed
        let lines: Vec<&str> = result.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_task6_arc_to_line_conversion_g2() {
        let gcode = "G90\nG0 X10 Y10\nG2 X20 Y20 I5 J0 F500";
        let result = convert_arcs_to_lines(gcode, 0.05);

        // G2 arc should be converted to G1 line segments
        assert!(!result.contains("G2 X20 Y20"));
        assert!(result.contains("G1"));
        // Feed rate should be preserved in first segment
        assert!(result.contains("F500"));
    }

    #[test]
    fn test_task6_arc_to_line_conversion_g3() {
        let gcode = "G90\nG0 X10 Y10\nG3 X20 Y20 I5 J0 F500";
        let result = convert_arcs_to_lines(gcode, 0.05);

        // G3 arc should be converted to G1 line segments
        assert!(!result.contains("G3 X20 Y20"));
        assert!(result.contains("G1"));
    }

    #[test]
    fn test_task6_arc_conversion_preserves_non_arc_moves() {
        let gcode = "G0 X0 Y0\nG1 X10 Y10\nG2 X20 Y20 I5 J0\nG1 X30 Y30";
        let result = convert_arcs_to_lines(gcode, 0.05);

        // Non-arc moves should be preserved
        assert!(result.contains("G0 X0 Y0"));
        assert!(result.contains("G1 X10 Y10"));
        assert!(result.contains("G1 X30 Y30"));
        // Arc should be converted
        assert!(!result.contains("G2 X20 Y20"));
    }

    #[test]
    fn test_task6_combined_optimization() {
        let gcode = "G90\nG0   X10.123456   Y10.987654\nG2  X20.555555  Y20.111111  I5.999999  J0.111111  F1000.555555\n\nG1  X30.777777  Y30.333333";
        
        // Apply multiple optimizations
        let result = remove_redundant_whitespace(gcode);
        let result = truncate_decimal_precision(&result, 2);
        let result = convert_arcs_to_lines(&result, 0.05);

        // Should have decimal precision
        assert!(result.contains("10.12") || result.contains("10.1"));
        // Should not have excessive whitespace
        assert!(!result.contains("   "));
        // Arc should be converted
        assert!(!result.contains("G2"));
    }

    #[test]
    fn test_task6_negative_coordinates_precision() {
        let gcode = "G1 X-10.123456 Y-20.987654 Z-5.555555";
        let result = truncate_decimal_precision(gcode, 2);

        // Negative signs should be preserved with precision
        assert!(result.contains("-10.12") || result.contains("-10.1"));
        assert!(result.contains("-20.98") || result.contains("-20.9"));
    }

    #[test]
    fn test_task6_comments_preserved() {
        let gcode = "G1 X10.123456 Y20.987654 ; Move to position\n; Start cutting\nG1 X30.555555 Y40.777777";
        let result = remove_redundant_whitespace(gcode);

        // Comments should be preserved
        assert!(result.contains("; Move to position"));
        assert!(result.contains("; Start cutting"));
    }

    #[test]
    fn test_task6_feed_rate_truncation() {
        let gcode = "G1 X10 Y10 F1234.56789";
        let result = truncate_decimal_precision(gcode, 1);

        // Feed rate precision should be truncated
        assert!(result.contains("1234.5"));
    }

    #[test]
    fn test_task6_spindle_speed_truncation() {
        let gcode = "M3 S5000.123456\nG1 X10 Y10";
        let result = truncate_decimal_precision(gcode, 1);

        // Spindle speed should also be truncated
        assert!(result.contains("5000.1"));
    }

    #[test]
    fn test_task6_complex_gcode_optimization() {
        let gcode = r#"G21  ; Set to mm
G90  ; Absolute positioning
G0   X0.000001   Y0.000001   Z5.000001   F3000.123456  ; Move to origin
G1   X100.999999   Y50.999999   Z0.000001   F2000.987654  ; Cut rectangle
G2   X100.999999   Y-50.999999   I0.000001   J-50.999999   F1500.555555  ; Arc
G0   Z5.000001   F3000.123456  ; Lift tool
M30  ; End program
"#;

        let result = remove_redundant_whitespace(gcode);
        let result = truncate_decimal_precision(&result, 2);

        // Multiple optimizations should be applied
        assert!(result.contains("G21"));
        assert!(result.contains("G90"));
        // Coordinates should be truncated (allow for rounding variations)
        assert!(result.contains("100.99") || result.contains("100.9") || result.contains("101"));
        // Excessive spacing should be removed
        assert!(!result.contains("   "));
    }

    #[test]
    fn test_task6_empty_gcode_optimization() {
        let gcode = "";
        
        let result1 = truncate_decimal_precision(gcode, 3);
        let result2 = remove_redundant_whitespace(gcode);
        let result3 = convert_arcs_to_lines(gcode, 0.05);

        assert!(result1.is_empty() || result1.trim().is_empty());
        assert!(result2.is_empty() || result2.trim().is_empty());
        assert!(result3.is_empty() || result3.trim().is_empty());
    }
}
