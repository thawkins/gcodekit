//! Integration tests for the gcodeedit module.

use gcodekit::gcodeedit::*;
use gcodekit::types::{MachinePosition, MoveType, PathSegment};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_editor_new() {
        let editor = GcodeEditorState::new();
        assert!(editor.gcode_content.is_empty());
        assert!(editor.gcode_filename.is_empty());
        assert!(editor.current_file_path.is_none());
        assert!(editor.search_query.is_empty());
        assert!(editor.search_results.is_empty());
        assert_eq!(editor.current_search_index, 0);
        assert!(editor.selected_line.is_none());
    }

    #[test]
    fn test_parse_empty_gcode() {
        let editor = GcodeEditorState {
            gcode_content: String::new(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_parse_simple_gcode() {
        let editor = GcodeEditorState {
            gcode_content: "G0 X10 Y20\nG1 X30 Y40 F100".to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 2);

        // First segment: G0 X10 Y20
        assert_eq!(paths[0].start.x, 0.0);
        assert_eq!(paths[0].start.y, 0.0);
        assert_eq!(paths[0].end.x, 10.0);
        assert_eq!(paths[0].end.y, 20.0);
        assert_eq!(paths[0].move_type, gcodekit::MoveType::Rapid);
        assert_eq!(paths[0].line_number, 0);

        // Second segment: G1 X30 Y40 F100
        assert_eq!(paths[1].start.x, 10.0);
        assert_eq!(paths[1].start.y, 20.0);
        assert_eq!(paths[1].end.x, 30.0);
        assert_eq!(paths[1].end.y, 40.0);
        assert_eq!(paths[1].move_type, gcodekit::MoveType::Feed);
        assert_eq!(paths[1].line_number, 1);
    }

    #[test]
    fn test_parse_gcode_with_comments() {
        let editor = GcodeEditorState {
            gcode_content:
                "; This is a comment\nG0 X10 Y20 ; inline comment\n; Another comment\nG1 X30 Y40"
                    .to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].line_number, 1); // Line 1 (0-indexed)
        assert_eq!(paths[1].line_number, 3); // Line 3 (0-indexed)
    }

    #[test]
    fn test_parse_gcode_with_arcs() {
        let editor = GcodeEditorState {
            gcode_content: "G0 X10 Y10\nG2 X20 Y20 I5 J5\nG3 X30 Y30 I10 J10".to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].move_type, gcodekit::MoveType::Rapid);
        assert_eq!(paths[1].move_type, gcodekit::MoveType::Arc);
        assert_eq!(paths[2].move_type, gcodekit::MoveType::Arc);
    }

    #[test]
    fn test_parse_gcode_multiple_axes() {
        let editor = GcodeEditorState {
            gcode_content: "G0 X10 Y20 Z30 A45 B90".to_string(),
            ..Default::default()
        };
        let paths = editor.parse_gcode();

        assert_eq!(paths.len(), 1);
        let segment = &paths[0];
        assert_eq!(segment.end.x, 10.0);
        assert_eq!(segment.end.y, 20.0);
        assert_eq!(segment.end.z, 30.0);
        assert_eq!(segment.end.a, Some(45.0));
        assert_eq!(segment.end.b, Some(90.0));
    }

    #[test]
    fn test_optimize_gcode_empty() {
        let mut editor = GcodeEditorState::new();
        let result = editor.optimize_gcode();
        assert_eq!(result, "No G-code to optimize");
    }

    #[test]
    fn test_optimize_gcode_with_comments() {
        let mut editor = GcodeEditorState {
            gcode_content: "; Header comment\nG0 X10 ; move to start\n; inline comment\nG1 X20 Y30 F100\n; footer comment".to_string(),
            ..Default::default()
        };

        let result = editor.optimize_gcode();
        assert_eq!(result, "G-code optimized: 5 -> 2 lines");
        assert_eq!(editor.gcode_content, "G0 X10\nG1 X20 Y30 F100");
    }

    #[test]
    fn test_optimize_gcode_remove_empty_lines() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\n\nG1 X20\n  \nG2 X30 Y30 I5 J5".to_string(),
            ..Default::default()
        };

        let result = editor.optimize_gcode();
        assert_eq!(result, "G-code optimized: 5 -> 3 lines");
        assert_eq!(editor.gcode_content, "G0 X10\nG1 X20\nG2 X30 Y30 I5 J5");
    }

    #[test]
    fn test_search_empty_query() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20".to_string(),
            ..Default::default()
        };

        editor.search_query = String::new();
        editor.perform_search();

        assert!(editor.search_results.is_empty());
        assert_eq!(editor.current_search_index, 0);
        assert!(editor.selected_line.is_none());
    }

    #[test]
    fn test_search_single_result() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20 Y30\nG2 X40 Y50 I10 J10".to_string(),
            ..Default::default()
        };

        editor.search_query = "X20".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 1);
        assert_eq!(editor.search_results[0], 1);
        assert_eq!(editor.current_search_index, 0);
        assert_eq!(editor.selected_line, Some(1));
    }

    #[test]
    fn test_search_multiple_results() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20\nG2 X30".to_string(),
            ..Default::default()
        };

        editor.search_query = "X".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 3);
        assert_eq!(editor.search_results, vec![0, 1, 2]);
        assert_eq!(editor.selected_line, Some(0));
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut editor = GcodeEditorState {
            gcode_content: "g0 x10\nG1 X20".to_string(),
            ..Default::default()
        };

        editor.search_query = "G0".to_string();
        editor.perform_search();

        assert_eq!(editor.search_results.len(), 1);
        assert_eq!(editor.search_results[0], 0);
    }

    #[test]
    fn test_search_next() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20\nG2 X30".to_string(),
            search_results: vec![0, 1, 2],
            current_search_index: 0,
            ..Default::default()
        };

        assert!(editor.search_next());
        assert_eq!(editor.current_search_index, 1);
        assert_eq!(editor.selected_line, Some(1));

        assert!(editor.search_next());
        assert_eq!(editor.current_search_index, 2);
        assert_eq!(editor.selected_line, Some(2));

        assert!(editor.search_next()); // Wrap around
        assert_eq!(editor.current_search_index, 0);
        assert_eq!(editor.selected_line, Some(0));
    }

    #[test]
    fn test_search_prev() {
        let mut editor = GcodeEditorState {
            gcode_content: "G0 X10\nG1 X20\nG2 X30".to_string(),
            search_results: vec![0, 1, 2],
            current_search_index: 2,
            ..Default::default()
        };

        assert!(editor.search_prev());
        assert_eq!(editor.current_search_index, 1);
        assert_eq!(editor.selected_line, Some(1));

        assert!(editor.search_prev());
        assert_eq!(editor.current_search_index, 0);
        assert_eq!(editor.selected_line, Some(0));

        assert!(editor.search_prev()); // Wrap around
        assert_eq!(editor.current_search_index, 2);
        assert_eq!(editor.selected_line, Some(2));
    }

    #[test]
    fn test_search_next_empty_results() {
        let mut editor = GcodeEditorState::new();
        assert!(!editor.search_next());
    }
}
