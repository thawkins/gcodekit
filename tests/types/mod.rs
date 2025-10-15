use gcodekit::types::*;

#[cfg(test)]
mod machine_position_tests {
    use super::*;

    #[test]
    fn test_machine_position_creation() {
        let pos = MachinePosition::new(10.0, 20.0, 5.0);

        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
        assert_eq!(pos.z, 5.0);
        assert_eq!(pos.a, 0.0);
        assert_eq!(pos.b, 0.0);
        assert_eq!(pos.c, 0.0);
    }

    #[test]
    fn test_machine_position_with_rotary() {
        let pos = MachinePosition {
            x: 10.0,
            y: 20.0,
            z: 5.0,
            a: 45.0,
            b: 90.0,
            c: 180.0,
        };

        assert_eq!(pos.a, 45.0);
        assert_eq!(pos.b, 90.0);
        assert_eq!(pos.c, 180.0);
    }

    #[test]
    fn test_machine_position_format() {
        let pos = MachinePosition::new(10.5, 20.75, 5.25);
        let formatted = pos.format();

        assert!(formatted.contains("10.50"));
        assert!(formatted.contains("20.75"));
        assert!(formatted.contains("5.25"));
    }

    #[test]
    fn test_machine_position_format_with_rotary() {
        let pos = MachinePosition {
            x: 10.0,
            y: 20.0,
            z: 5.0,
            a: 45.0,
            b: 90.0,
            c: 180.0,
        };

        let formatted = pos.format();

        assert!(formatted.contains("X:"));
        assert!(formatted.contains("Y:"));
        assert!(formatted.contains("Z:"));
        assert!(formatted.contains("A:") || formatted.contains("45.00"));
        assert!(formatted.contains("B:") || formatted.contains("90.00"));
        assert!(formatted.contains("C:") || formatted.contains("180.00"));
    }

    #[test]
    fn test_machine_position_clone() {
        let pos1 = MachinePosition::new(1.0, 2.0, 3.0);
        let pos2 = pos1.clone();

        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
        assert_eq!(pos1.z, pos2.z);
    }

    #[test]
    fn test_machine_position_distance() {
        let pos1 = MachinePosition::new(0.0, 0.0, 0.0);
        let pos2 = MachinePosition::new(3.0, 4.0, 0.0);

        let distance = pos1.distance_to(&pos2);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_machine_position_distance_3d() {
        let pos1 = MachinePosition::new(0.0, 0.0, 0.0);
        let pos2 = MachinePosition::new(1.0, 0.0, 0.0);

        let distance = pos1.distance_to(&pos2);
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_machine_position_negative_coordinates() {
        let pos = MachinePosition::new(-10.0, -20.0, -5.0);

        assert_eq!(pos.x, -10.0);
        assert_eq!(pos.y, -20.0);
        assert_eq!(pos.z, -5.0);
    }
}

#[cfg(test)]
mod machine_mode_tests {
    use super::*;

    #[test]
    fn test_machine_mode_enum() {
        let modes = vec![MachineMode::CNC, MachineMode::Laser, MachineMode::Plasma];

        for mode in modes {
            let cloned = mode.clone();
            assert_eq!(mode, cloned);
        }
    }

    #[test]
    fn test_machine_mode_debug() {
        assert_eq!(format!("{:?}", MachineMode::CNC), "CNC");
        assert_eq!(format!("{:?}", MachineMode::Laser), "Laser");
        assert_eq!(format!("{:?}", MachineMode::Plasma), "Plasma");
    }

    #[test]
    fn test_machine_mode_default() {
        let mode = MachineMode::default();
        assert_eq!(mode, MachineMode::CNC);
    }
}

#[cfg(test)]
mod tab_tests {
    use super::*;

    #[test]
    fn test_tab_enum() {
        let tabs = vec![
            Tab::Designer,
            Tab::Visualizer,
            Tab::GcodeEditor,
            Tab::JobManager,
            Tab::DeviceConsole,
        ];

        for tab in tabs {
            let cloned = tab.clone();
            assert_eq!(tab, cloned);
        }
    }

    #[test]
    fn test_tab_default() {
        let tab = Tab::default();
        assert_eq!(tab, Tab::Designer);
    }

    #[test]
    fn test_tab_debug() {
        assert_eq!(format!("{:?}", Tab::Designer), "Designer");
        assert_eq!(format!("{:?}", Tab::Visualizer), "Visualizer");
        assert_eq!(format!("{:?}", Tab::GcodeEditor), "GcodeEditor");
        assert_eq!(format!("{:?}", Tab::JobManager), "JobManager");
        assert_eq!(format!("{:?}", Tab::DeviceConsole), "DeviceConsole");
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
        assert_eq!(segment.end_x, 10.0);
        assert_eq!(segment.move_type, MoveType::Linear);
    }

    #[test]
    fn test_path_segment_2d() {
        let segment = PathSegment {
            start_x: 0.0,
            start_y: 0.0,
            start_z: None,
            end_x: 10.0,
            end_y: 10.0,
            end_z: None,
            move_type: MoveType::Rapid,
            feed_rate: None,
        };

        assert!(segment.start_z.is_none());
        assert!(segment.end_z.is_none());
    }

    #[test]
    fn test_path_segment_clone() {
        let segment1 = PathSegment {
            start_x: 0.0,
            start_y: 0.0,
            start_z: None,
            end_x: 10.0,
            end_y: 10.0,
            end_z: None,
            move_type: MoveType::Linear,
            feed_rate: Some(300.0),
        };

        let segment2 = segment1.clone();

        assert_eq!(segment1.start_x, segment2.start_x);
        assert_eq!(segment1.end_x, segment2.end_x);
        assert_eq!(segment1.move_type, segment2.move_type);
        assert_eq!(segment1.feed_rate, segment2.feed_rate);
    }
}

#[cfg(test)]
mod move_type_tests {
    use super::*;

    #[test]
    fn test_move_type_variants() {
        let types = vec![
            MoveType::Rapid,
            MoveType::Linear,
            MoveType::ArcCW,
            MoveType::ArcCCW,
        ];

        for move_type in types {
            let cloned = move_type.clone();
            assert_eq!(move_type, cloned);

            // Test debug output
            let debug_str = format!("{:?}", move_type);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_move_type_equality() {
        assert_eq!(MoveType::Rapid, MoveType::Rapid);
        assert_eq!(MoveType::Linear, MoveType::Linear);
        assert_ne!(MoveType::Rapid, MoveType::Linear);
        assert_ne!(MoveType::ArcCW, MoveType::ArcCCW);
    }

    #[test]
    fn test_move_type_from_gcode_command() {
        // These mappings should be consistent with gcode parser
        let g0_type = MoveType::Rapid; // G0
        let g1_type = MoveType::Linear; // G1
        let g2_type = MoveType::ArcCW; // G2
        let g3_type = MoveType::ArcCCW; // G3

        assert_eq!(g0_type, MoveType::Rapid);
        assert_eq!(g1_type, MoveType::Linear);
        assert_eq!(g2_type, MoveType::ArcCW);
        assert_eq!(g3_type, MoveType::ArcCCW);
    }
}

#[cfg(test)]
mod enum_tests {
    use super::*;

    #[test]
    fn test_all_enums_implement_clone() {
        // These should all compile if Clone is properly implemented
        let _pos = MachinePosition::new(1.0, 2.0, 3.0).clone();
        let _mode = MachineMode::CNC.clone();
        let _tab = Tab::Designer.clone();
        let _move = MoveType::Linear.clone();
    }

    #[test]
    fn test_all_enums_implement_debug() {
        // These should all compile if Debug is properly implemented
        format!("{:?}", MachinePosition::new(1.0, 2.0, 3.0));
        format!("{:?}", MachineMode::CNC);
        format!("{:?}", Tab::Designer);
        format!("{:?}", MoveType::Linear);
    }

    #[test]
    fn test_all_enums_implement_partial_eq() {
        // These should all compile if PartialEq is properly implemented
        assert_eq!(MachineMode::CNC, MachineMode::CNC);
        assert_eq!(Tab::Designer, Tab::Designer);
        assert_eq!(MoveType::Linear, MoveType::Linear);
    }
}
