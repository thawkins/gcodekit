//! Calibration module tests

use gcodekit::calibration::step_calibration::StepCalibration;
use gcodekit::calibration::backlash_compensation::BacklashCompensation;
use gcodekit::calibration::homing_config::HomingConfiguration;

#[test]
fn test_step_calibration_default() {
    let calib = StepCalibration::default();
    assert_eq!(calib.x_steps, 100.0);
    assert_eq!(calib.y_steps, 100.0);
    assert_eq!(calib.z_steps, 100.0);
}

#[test]
fn test_step_calibration_new() {
    let calib = StepCalibration::new(150.0, 200.0, 250.0);
    assert_eq!(calib.x_steps, 150.0);
    assert_eq!(calib.y_steps, 200.0);
    assert_eq!(calib.z_steps, 250.0);
}

#[test]
fn test_step_calibration_grbl_commands() {
    let calib = StepCalibration::new(120.0, 130.0, 140.0);
    let commands = calib.get_grbl_commands();
    
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0], "$100=120.000");
    assert_eq!(commands[1], "$101=130.000");
    assert_eq!(commands[2], "$102=140.000");
}

#[test]
fn test_step_calibration_correction() {
    // If we commanded 100mm but only moved 90mm with 100 steps/mm
    // The correction should be: 100 * 100 / 90 = 111.11 steps/mm
    let corrected = StepCalibration::calculate_correction(100.0, 100.0, 90.0);
    assert!((corrected - 111.11).abs() < 0.01);
}

#[test]
fn test_step_calibration_correction_exact() {
    // Perfect calibration should stay the same
    let corrected = StepCalibration::calculate_correction(100.0, 100.0, 100.0);
    assert_eq!(corrected, 100.0);
}

#[test]
fn test_step_calibration_correction_over_movement() {
    // If we commanded 100mm but moved 110mm
    // The correction should be: 100 * 100 / 110 = 90.91 steps/mm
    let corrected = StepCalibration::calculate_correction(100.0, 100.0, 110.0);
    assert!((corrected - 90.91).abs() < 0.01);
}

#[test]
fn test_backlash_compensation_default() {
    let backlash = BacklashCompensation::default();
    assert!(backlash.x_backlash >= 0.0);
    assert!(backlash.y_backlash >= 0.0);
    assert!(backlash.z_backlash >= 0.0);
}

#[test]
fn test_backlash_compensation_new() {
    let backlash = BacklashCompensation::new(0.1, 0.15, 0.2);
    assert_eq!(backlash.x_backlash, 0.1);
    assert_eq!(backlash.y_backlash, 0.15);
    assert_eq!(backlash.z_backlash, 0.2);
}

#[test]
fn test_backlash_compensation_apply() {
    let backlash = BacklashCompensation::new(0.1, 0.2, 0.3);
    
    // Test GRBL commands generation
    let commands = backlash.get_grbl_commands();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0], "$130=0.100");
    assert_eq!(commands[1], "$131=0.200");
    assert_eq!(commands[2], "$132=0.300");
}

#[test]
fn test_homing_config_default() {
    let config = HomingConfiguration::default();
    assert!(config.homing_enable);
    assert_eq!(config.homing_sequence, [true, true, true]);
}

#[test]
fn test_homing_config_get_grbl_commands() {
    let config = HomingConfiguration::default();
    let commands = config.get_grbl_commands();
    
    // Should return 4 commands for $22-$25
    assert_eq!(commands.len(), 4);
    assert!(commands[0].contains("$22"));
    assert!(commands[1].contains("$23"));
    assert!(commands[2].contains("$24"));
    assert!(commands[3].contains("$25"));
}

#[test]
fn test_homing_config_validate_feed_rate() {
    assert!(HomingConfiguration::validate_feed_rate(25.0));
    assert!(HomingConfiguration::validate_feed_rate(500.0));
    assert!(!HomingConfiguration::validate_feed_rate(0.0));
    assert!(!HomingConfiguration::validate_feed_rate(10000.0));
}

#[test]
fn test_homing_config_set_enable() {
    let mut config = HomingConfiguration::default();
    assert!(config.homing_enable);
    
    config.set_homing_enable(false);
    assert!(!config.homing_enable);
    
    config.set_homing_enable(true);
    assert!(config.homing_enable);
}

#[test]
fn test_homing_config_clone() {
    let config1 = HomingConfiguration::default();
    let config2 = config1.clone();
    
    assert_eq!(config1.homing_enable, config2.homing_enable);
    assert_eq!(config1.homing_sequence, config2.homing_sequence);
}

#[test]
fn test_step_calibration_clone() {
    let calib1 = StepCalibration::new(100.0, 110.0, 120.0);
    let calib2 = calib1;
    
    assert_eq!(calib1.x_steps, calib2.x_steps);
    assert_eq!(calib1.y_steps, calib2.y_steps);
    assert_eq!(calib1.z_steps, calib2.z_steps);
}

#[test]
fn test_backlash_compensation_grbl_compensation() {
    let _backlash = BacklashCompensation::new(0.05, 0.1, 0.15);
    
    // Test that backlash detection works
    let forward_pos = 5.0;
    let backward_pos = 4.95;
    let detected = BacklashCompensation::detect_backlash(forward_pos, backward_pos, 5.0);
    
    // Detected backlash should be the difference
    assert!((detected - 0.05).abs() < 0.001);
}

#[test]
fn test_backlash_compensation_validate() {
    assert!(BacklashCompensation::validate_backlash(0.0));
    assert!(BacklashCompensation::validate_backlash(0.5));
    assert!(BacklashCompensation::validate_backlash(5.0));
    assert!(!BacklashCompensation::validate_backlash(-0.1));
    assert!(!BacklashCompensation::validate_backlash(10.1));
}

#[test]
fn test_backlash_compensation_set_x() {
    let mut backlash = BacklashCompensation::default();
    
    let result = backlash.set_x_backlash(0.1);
    assert!(result.is_ok());
    assert_eq!(backlash.x_backlash, 0.1);
    
    let result = backlash.set_x_backlash(10.5);
    assert!(result.is_err());
}
