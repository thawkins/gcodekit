//! Back Plotting Module
//!
//! Provides visual G-code simulation and step-through visualization capabilities.
//! Supports stepping forward/backward through G-code execution with real-time
//! machine position tracking and tool path visualization.

use crate::types::MoveType;
use std::collections::VecDeque;

/// Represents a single step in the back plot execution
#[derive(Clone, Debug)]
pub struct BackPlotStep {
    /// Line number in the original G-code
    pub line_number: usize,
    /// Machine position after this step
    pub position: (f32, f32, f32),
    /// Move type (rapid, feed, arc)
    pub move_type: MoveType,
    /// Start position before move
    pub start_position: (f32, f32, f32),
    /// Feed rate for this move
    pub feed_rate: f32,
    /// Spindle speed for this move
    pub spindle_speed: f32,
}

/// State of the back plot simulator
#[derive(Clone, Debug, PartialEq)]
pub enum BackPlotState {
    /// Not running
    Idle,
    /// Currently stepping through G-code
    Running,
    /// Paused at a specific step
    Paused,
    /// Completed all steps
    Completed,
}

/// Back plot simulator for visual G-code execution
#[derive(Clone, Debug)]
pub struct BackPlotter {
    /// Current state
    pub state: BackPlotState,
    /// Current step index
    pub current_step: usize,
    /// All recorded steps
    pub steps: Vec<BackPlotStep>,
    /// Step history for undo/redo
    pub history: VecDeque<BackPlotStep>,
    /// Current machine position
    pub current_position: (f32, f32, f32),
    /// Current feed rate
    pub feed_rate: f32,
    /// Current spindle speed
    pub spindle_speed: f32,
    /// Speed multiplier for simulation (default 1.0)
    pub simulation_speed: f32,
    /// Max history size for undo
    pub max_history: usize,
}

impl Default for BackPlotter {
    fn default() -> Self {
        Self {
            state: BackPlotState::Idle,
            current_step: 0,
            steps: Vec::new(),
            history: VecDeque::with_capacity(100),
            current_position: (0.0, 0.0, 0.0),
            feed_rate: 1000.0,
            spindle_speed: 0.0,
            simulation_speed: 1.0,
            max_history: 1000,
        }
    }
}

impl BackPlotter {
    /// Create a new back plotter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a step to the back plot
    pub fn add_step(&mut self, step: BackPlotStep) {
        self.steps.push(step);
    }

    /// Start the back plot simulation
    pub fn start(&mut self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err("No steps to execute".to_string());
        }
        self.state = BackPlotState::Running;
        self.current_step = 0;
        self.current_position = (0.0, 0.0, 0.0);
        Ok(())
    }

    /// Step forward one move
    pub fn step_forward(&mut self) -> Result<(), String> {
        if self.current_step >= self.steps.len() {
            self.state = BackPlotState::Completed;
            return Ok(());
        }

        let step = &self.steps[self.current_step];
        self.current_position = step.position;
        self.feed_rate = step.feed_rate;
        self.spindle_speed = step.spindle_speed;

        // Record in history
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(step.clone());

        self.current_step += 1;

        if self.current_step >= self.steps.len() {
            self.state = BackPlotState::Completed;
        }

        Ok(())
    }

    /// Step backward one move
    pub fn step_backward(&mut self) -> Result<(), String> {
        if self.current_step == 0 {
            return Err("Already at beginning".to_string());
        }

        self.current_step -= 1;

        if self.current_step < self.steps.len() {
            let step = &self.steps[self.current_step];
            self.current_position = step.start_position;
            self.feed_rate = step.feed_rate;
            self.spindle_speed = step.spindle_speed;
        }

        self.state = BackPlotState::Running;
        Ok(())
    }

    /// Jump to a specific step
    pub fn jump_to_step(&mut self, step: usize) -> Result<(), String> {
        if step > self.steps.len() {
            return Err(format!("Step {} out of range", step));
        }

        self.current_step = step;

        if self.current_step == 0 {
            self.current_position = (0.0, 0.0, 0.0);
        } else if self.current_step <= self.steps.len() {
            let step_data = &self.steps[self.current_step - 1];
            self.current_position = step_data.position;
        }

        if self.current_step >= self.steps.len() {
            self.state = BackPlotState::Completed;
        } else {
            self.state = BackPlotState::Running;
        }

        Ok(())
    }

    /// Pause the simulation
    pub fn pause(&mut self) {
        if !matches!(self.state, BackPlotState::Idle) {
            self.state = BackPlotState::Paused;
        }
    }

    /// Resume the simulation
    pub fn resume(&mut self) {
        if matches!(self.state, BackPlotState::Paused) {
            self.state = BackPlotState::Running;
        }
    }

    /// Stop and reset the simulation
    pub fn stop(&mut self) {
        self.state = BackPlotState::Idle;
        self.current_step = 0;
        self.current_position = (0.0, 0.0, 0.0);
        self.history.clear();
    }

    /// Get progress as percentage
    pub fn get_progress_percent(&self) -> f32 {
        if self.steps.is_empty() {
            0.0
        } else {
            (self.current_step as f32 / self.steps.len() as f32) * 100.0
        }
    }

    /// Get the current step data
    pub fn get_current_step_data(&self) -> Option<&BackPlotStep> {
        if self.current_step > 0 && self.current_step <= self.steps.len() {
            Some(&self.steps[self.current_step - 1])
        } else {
            None
        }
    }

    /// Get steps in a range
    pub fn get_steps_in_range(&self, start: usize, end: usize) -> Vec<&BackPlotStep> {
        self.steps[start.min(self.steps.len())..end.min(self.steps.len())].iter().collect()
    }

    /// Clear all steps
    pub fn clear(&mut self) {
        self.steps.clear();
        self.history.clear();
        self.current_step = 0;
        self.current_position = (0.0, 0.0, 0.0);
        self.state = BackPlotState::Idle;
    }

    /// Set simulation speed (1.0 = normal)
    pub fn set_simulation_speed(&mut self, speed: f32) -> Result<(), String> {
        if speed <= 0.0 {
            return Err("Simulation speed must be positive".to_string());
        }
        self.simulation_speed = speed;
        Ok(())
    }

    /// Get the total number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if simulation is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state, BackPlotState::Completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_step(
        line: usize,
        pos: (f32, f32, f32),
        move_type: MoveType,
    ) -> BackPlotStep {
        BackPlotStep {
            line_number: line,
            position: pos,
            move_type,
            start_position: (0.0, 0.0, 0.0),
            feed_rate: 1000.0,
            spindle_speed: 0.0,
        }
    }

    #[test]
    fn test_back_plotter_new() {
        let bp = BackPlotter::new();
        assert!(matches!(bp.state, BackPlotState::Idle));
        assert_eq!(bp.current_step, 0);
        assert_eq!(bp.step_count(), 0);
    }

    #[test]
    fn test_add_step() {
        let mut bp = BackPlotter::new();
        let step = create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed);
        bp.add_step(step);
        assert_eq!(bp.step_count(), 1);
    }

    #[test]
    fn test_start_empty_error() {
        let mut bp = BackPlotter::new();
        assert!(bp.start().is_err());
    }

    #[test]
    fn test_start_success() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        assert!(bp.start().is_ok());
        assert!(matches!(bp.state, BackPlotState::Running));
    }

    #[test]
    fn test_step_forward() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.add_step(create_test_step(2, (20.0, 30.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        assert!(bp.step_forward().is_ok());
        assert_eq!(bp.current_step, 1);
        assert_eq!(bp.current_position, (10.0, 20.0, 5.0));
    }

    #[test]
    fn test_step_backward() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.add_step(create_test_step(2, (20.0, 30.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        bp.step_forward().unwrap();
        assert!(bp.step_backward().is_ok());
        assert_eq!(bp.current_step, 0);
    }

    #[test]
    fn test_jump_to_step() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.add_step(create_test_step(2, (20.0, 30.0, 5.0), MoveType::Feed));
        bp.add_step(create_test_step(3, (30.0, 40.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        assert!(bp.jump_to_step(2).is_ok());
        assert_eq!(bp.current_step, 2);
    }

    #[test]
    fn test_pause_resume() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        bp.pause();
        assert!(matches!(bp.state, BackPlotState::Paused));

        bp.resume();
        assert!(matches!(bp.state, BackPlotState::Running));
    }

    #[test]
    fn test_stop_reset() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.start().unwrap();
        bp.step_forward().unwrap();

        bp.stop();
        assert!(matches!(bp.state, BackPlotState::Idle));
        assert_eq!(bp.current_step, 0);
    }

    #[test]
    fn test_progress_percent() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.add_step(create_test_step(2, (20.0, 30.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        assert_eq!(bp.get_progress_percent(), 0.0);
        bp.step_forward().unwrap();
        assert_eq!(bp.get_progress_percent(), 50.0);
        bp.step_forward().unwrap();
        assert_eq!(bp.get_progress_percent(), 100.0);
    }

    #[test]
    fn test_set_simulation_speed() {
        let mut bp = BackPlotter::new();
        assert!(bp.set_simulation_speed(2.0).is_ok());
        assert_eq!(bp.simulation_speed, 2.0);

        assert!(bp.set_simulation_speed(0.0).is_err());
        assert!(bp.set_simulation_speed(-1.0).is_err());
    }

    #[test]
    fn test_clear() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.start().unwrap();
        bp.step_forward().unwrap();

        bp.clear();
        assert_eq!(bp.step_count(), 0);
        assert_eq!(bp.current_step, 0);
        assert!(matches!(bp.state, BackPlotState::Idle));
    }

    #[test]
    fn test_get_current_step_data() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        assert!(bp.get_current_step_data().is_none());
        bp.step_forward().unwrap();
        assert!(bp.get_current_step_data().is_some());
    }

    #[test]
    fn test_is_complete() {
        let mut bp = BackPlotter::new();
        bp.add_step(create_test_step(1, (10.0, 20.0, 5.0), MoveType::Feed));
        bp.start().unwrap();

        assert!(!bp.is_complete());
        bp.step_forward().unwrap();
        assert!(bp.is_complete());
    }
}
