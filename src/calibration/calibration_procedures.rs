//! Calibration procedures for automated machine calibration.
//!
//! Provides step-by-step calibration procedures for step calibration,
//! backlash measurement, and homing configuration.

use serde::{Deserialize, Serialize};

/// A single step in a calibration procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationStep {
    pub step_number: usize,
    pub description: String,
    pub command: Option<String>,
    pub expected_result: String,
    pub user_action_required: bool,
}

impl CalibrationStep {
    /// Create a new calibration step.
    pub fn new(
        step_number: usize,
        description: String,
        command: Option<String>,
        expected_result: String,
        user_action_required: bool,
    ) -> Self {
        Self {
            step_number,
            description,
            command,
            expected_result,
            user_action_required,
        }
    }
}

/// Type of calibration procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcedureType {
    StepCalibration,
    BacklashMeasurement,
    HomingSetup,
    FullCalibration,
}

/// A complete calibration procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationProcedure {
    pub procedure_type: ProcedureType,
    pub name: String,
    pub description: String,
    pub steps: Vec<CalibrationStep>,
    pub estimated_duration_minutes: u32,
}

impl CalibrationProcedure {
    /// Create step calibration procedure.
    pub fn step_calibration() -> Self {
        let steps = vec![
            CalibrationStep::new(
                1,
                "Home all axes".to_string(),
                Some("$H".to_string()),
                "Machine moves to limit switches".to_string(),
                false,
            ),
            CalibrationStep::new(
                2,
                "Measure accurate reference distance".to_string(),
                None,
                "Mark or use precision fixture at known distance (e.g., 100mm)".to_string(),
                true,
            ),
            CalibrationStep::new(
                3,
                "Command X axis movement".to_string(),
                Some("G0 X100".to_string()),
                "Machine moves to marked position".to_string(),
                false,
            ),
            CalibrationStep::new(
                4,
                "Measure actual position".to_string(),
                None,
                "Use calipers/ruler to measure actual distance traveled".to_string(),
                true,
            ),
            CalibrationStep::new(
                5,
                "Calculate new step value".to_string(),
                None,
                "new_steps = current_steps * commanded / actual".to_string(),
                false,
            ),
            CalibrationStep::new(
                6,
                "Apply calibration".to_string(),
                Some("$100=<calculated_value>".to_string()),
                "New step value is written to GRBL".to_string(),
                false,
            ),
            CalibrationStep::new(
                7,
                "Repeat for Y and Z axes".to_string(),
                None,
                "Perform steps 2-6 for Y and Z axes".to_string(),
                true,
            ),
        ];

        Self {
            procedure_type: ProcedureType::StepCalibration,
            name: "Step Calibration".to_string(),
            description: "Calibrate steps/mm for each axis using measured distance".to_string(),
            steps,
            estimated_duration_minutes: 30,
        }
    }

    /// Create backlash measurement procedure.
    pub fn backlash_measurement() -> Self {
        let steps = vec![
            CalibrationStep::new(
                1,
                "Home all axes".to_string(),
                Some("$H".to_string()),
                "Machine moves to limit switches".to_string(),
                false,
            ),
            CalibrationStep::new(
                2,
                "Move to test position".to_string(),
                Some("G0 X50".to_string()),
                "Machine moves to safe test position".to_string(),
                false,
            ),
            CalibrationStep::new(
                3,
                "Record forward position".to_string(),
                None,
                "Read current position from DRO or move back to mark".to_string(),
                true,
            ),
            CalibrationStep::new(
                4,
                "Move backward 10mm".to_string(),
                Some("G0 X40".to_string()),
                "Machine moves backward from test position".to_string(),
                false,
            ),
            CalibrationStep::new(
                5,
                "Move forward 10mm again".to_string(),
                Some("G0 X50".to_string()),
                "Machine moves back to same position".to_string(),
                false,
            ),
            CalibrationStep::new(
                6,
                "Record backward position".to_string(),
                None,
                "Position differs from step 3 due to backlash".to_string(),
                true,
            ),
            CalibrationStep::new(
                7,
                "Calculate backlash".to_string(),
                None,
                "backlash = |forward_pos - backward_pos| - 10mm".to_string(),
                false,
            ),
            CalibrationStep::new(
                8,
                "Repeat for Y and Z axes".to_string(),
                None,
                "Perform steps 2-7 for Y and Z axes".to_string(),
                true,
            ),
        ];

        Self {
            procedure_type: ProcedureType::BacklashMeasurement,
            name: "Backlash Measurement".to_string(),
            description: "Measure and compensate for mechanical backlash in drive systems".to_string(),
            steps,
            estimated_duration_minutes: 20,
        }
    }

    /// Create homing setup procedure.
    pub fn homing_setup() -> Self {
        let steps = vec![
            CalibrationStep::new(
                1,
                "Inspect limit switches".to_string(),
                None,
                "Verify limit switches are present and functional on all axes".to_string(),
                true,
            ),
            CalibrationStep::new(
                2,
                "Enable homing".to_string(),
                Some("$22=1".to_string()),
                "Homing cycle is enabled in GRBL".to_string(),
                false,
            ),
            CalibrationStep::new(
                3,
                "Set homing seek rate (fast approach)".to_string(),
                Some("$25=500".to_string()),
                "Set appropriate speed for fast approach to limit switch (500 mm/min typical)".to_string(),
                true,
            ),
            CalibrationStep::new(
                4,
                "Set homing feed rate (slow approach)".to_string(),
                Some("$24=25".to_string()),
                "Set slow speed after switch contact (25 mm/min typical)".to_string(),
                true,
            ),
            CalibrationStep::new(
                5,
                "Set homing direction invert if needed".to_string(),
                Some("$23=0".to_string()),
                "Adjust if axes home in wrong direction (bit mask: 0=X, 1=Y, 2=Z)".to_string(),
                true,
            ),
            CalibrationStep::new(
                6,
                "Test homing cycle".to_string(),
                Some("$H".to_string()),
                "Machine homes to limit switches and reports zero position".to_string(),
                false,
            ),
            CalibrationStep::new(
                7,
                "Verify homing position".to_string(),
                None,
                "Check that machine stops at consistent position each time".to_string(),
                true,
            ),
        ];

        Self {
            procedure_type: ProcedureType::HomingSetup,
            name: "Homing Setup".to_string(),
            description: "Configure homing sequence for reliable machine zeroing".to_string(),
            steps,
            estimated_duration_minutes: 15,
        }
    }

    /// Create full calibration procedure (all three).
    pub fn full_calibration() -> Self {
        let mut steps = Vec::new();
        let mut step_num = 1;

        // Add step calibration steps
        for step in &Self::step_calibration().steps {
            let mut new_step = step.clone();
            new_step.step_number = step_num;
            steps.push(new_step);
            step_num += 1;
        }

        // Add backlash measurement steps
        for step in &Self::backlash_measurement().steps {
            let mut new_step = step.clone();
            new_step.step_number = step_num;
            steps.push(new_step);
            step_num += 1;
        }

        // Add homing setup steps
        for step in &Self::homing_setup().steps {
            let mut new_step = step.clone();
            new_step.step_number = step_num;
            steps.push(new_step);
            step_num += 1;
        }

        Self {
            procedure_type: ProcedureType::FullCalibration,
            name: "Full Machine Calibration".to_string(),
            description: "Complete calibration including steps, backlash, and homing".to_string(),
            steps,
            estimated_duration_minutes: 65,
        }
    }

    /// Get next step in procedure.
    pub fn get_next_step(&self, current_step: usize) -> Option<&CalibrationStep> {
        self.steps.iter().find(|s| s.step_number == current_step + 1)
    }

    /// Get current step.
    pub fn get_current_step(&self, step_num: usize) -> Option<&CalibrationStep> {
        self.steps.iter().find(|s| s.step_number == step_num)
    }

    /// Get total number of steps.
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// Check if all steps are user actions.
    pub fn get_user_action_steps(&self) -> Vec<&CalibrationStep> {
        self.steps.iter().filter(|s| s.user_action_required).collect()
    }

    /// Get steps with GRBL commands.
    pub fn get_command_steps(&self) -> Vec<&CalibrationStep> {
        self.steps.iter().filter(|s| s.command.is_some()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_step_new() {
        let step = CalibrationStep::new(
            1,
            "Test step".to_string(),
            Some("$H".to_string()),
            "Expected result".to_string(),
            true,
        );
        assert_eq!(step.step_number, 1);
        assert_eq!(step.description, "Test step");
        assert_eq!(step.command, Some("$H".to_string()));
        assert!(step.user_action_required);
    }

    #[test]
    fn test_step_calibration_procedure() {
        let proc = CalibrationProcedure::step_calibration();
        assert_eq!(proc.procedure_type, ProcedureType::StepCalibration);
        assert!(!proc.steps.is_empty());
        assert!(proc.estimated_duration_minutes > 0);
    }

    #[test]
    fn test_backlash_measurement_procedure() {
        let proc = CalibrationProcedure::backlash_measurement();
        assert_eq!(proc.procedure_type, ProcedureType::BacklashMeasurement);
        assert!(!proc.steps.is_empty());
        assert!(proc.estimated_duration_minutes > 0);
    }

    #[test]
    fn test_homing_setup_procedure() {
        let proc = CalibrationProcedure::homing_setup();
        assert_eq!(proc.procedure_type, ProcedureType::HomingSetup);
        assert!(!proc.steps.is_empty());
        assert!(proc.estimated_duration_minutes > 0);
    }

    #[test]
    fn test_full_calibration_procedure() {
        let proc = CalibrationProcedure::full_calibration();
        assert_eq!(proc.procedure_type, ProcedureType::FullCalibration);
        assert!(proc.steps.len() > 7); // More than any single procedure
        assert!(proc.estimated_duration_minutes > 50);
    }

    #[test]
    fn test_get_next_step() {
        let proc = CalibrationProcedure::step_calibration();
        let step1 = proc.get_current_step(1).unwrap();
        assert_eq!(step1.step_number, 1);

        let step2 = proc.get_next_step(1);
        assert!(step2.is_some());
        assert_eq!(step2.unwrap().step_number, 2);
    }

    #[test]
    fn test_total_steps() {
        let proc = CalibrationProcedure::step_calibration();
        assert_eq!(proc.total_steps(), 7);
    }

    #[test]
    fn test_get_user_action_steps() {
        let proc = CalibrationProcedure::step_calibration();
        let user_steps = proc.get_user_action_steps();
        assert!(!user_steps.is_empty());
    }

    #[test]
    fn test_get_command_steps() {
        let proc = CalibrationProcedure::step_calibration();
        let cmd_steps = proc.get_command_steps();
        assert!(!cmd_steps.is_empty());
    }
}
