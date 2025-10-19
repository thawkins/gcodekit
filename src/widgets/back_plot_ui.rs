//! Back Plot Visualization Widget
//!
//! Provides UI for step-through G-code visualization and back plot controls.
//! Integrates with the 3D visualizer to show real-time tool path stepping.

use crate::gcodeedit::{BackPlotState, BackPlotter};
use crate::GcodeKitApp;
use eframe::egui;

/// Configuration for back plot UI display
#[derive(Clone, Debug)]
pub struct BackPlotUiConfig {
    /// Show step counter
    pub show_step_counter: bool,
    /// Show progress bar
    pub show_progress_bar: bool,
    /// Button width
    pub button_width: f32,
    /// Auto-play speed (steps per second, 0 = no auto)
    pub auto_play_speed: f32,
}

impl Default for BackPlotUiConfig {
    fn default() -> Self {
        Self {
            show_step_counter: true,
            show_progress_bar: true,
            button_width: 80.0,
            auto_play_speed: 0.0,
        }
    }
}

impl BackPlotUiConfig {
    /// Create with auto-play enabled
    pub fn with_auto_play(speed: f32) -> Self {
        Self {
            auto_play_speed: speed,
            ..Default::default()
        }
    }
}

/// Display the back plot control panel
///
/// Provides controls for stepping forward/backward through G-code,
/// jumping to specific steps, and adjusting simulation speed.
pub fn show_back_plot_panel(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.heading("🎬 Back Plot Simulator");

        // Control buttons row
        ui.horizontal(|ui| {
            let button_width = 70.0;

            if app.back_plotter.step_count() == 0 {
                ui.label("No G-code loaded for simulation");
                return;
            }

            // State info
            let state_text = match &app.back_plotter.state {
                BackPlotState::Idle => "Idle",
                BackPlotState::Running => "Running",
                BackPlotState::Paused => "Paused",
                BackPlotState::Completed => "Completed",
            };
            let state_color = match &app.back_plotter.state {
                BackPlotState::Idle => egui::Color32::GRAY,
                BackPlotState::Running => egui::Color32::GREEN,
                BackPlotState::Paused => egui::Color32::YELLOW,
                BackPlotState::Completed => egui::Color32::BLUE,
            };

            ui.colored_label(state_color, format!("State: {}", state_text));

            ui.separator();

            // Start button
            if ui
                .add_sized([button_width, 30.0], egui::Button::new("▶ Start"))
                .clicked()
            {
                let _ = app.back_plotter.start();
            }

            // Pause button
            if ui
                .add_sized([button_width, 30.0], egui::Button::new("⏸ Pause"))
                .clicked()
            {
                app.back_plotter.pause();
            }

            // Resume button
            if ui
                .add_sized([button_width, 30.0], egui::Button::new("▶ Resume"))
                .clicked()
            {
                app.back_plotter.resume();
            }

            // Stop button
            if ui
                .add_sized([button_width, 30.0], egui::Button::new("⏹ Stop"))
                .clicked()
            {
                app.back_plotter.stop();
            }
        });

        ui.separator();

        // Step controls row
        ui.horizontal(|ui| {
            let button_width = 70.0;

            // Step back button
            if ui
                .add_sized(
                    [button_width, 30.0],
                    egui::Button::new("⬅ Step Back"),
                )
                .clicked()
            {
                let _ = app.back_plotter.step_backward();
            }

            // Step forward button
            if ui
                .add_sized(
                    [button_width, 30.0],
                    egui::Button::new("Step ➡"),
                )
                .clicked()
            {
                let _ = app.back_plotter.step_forward();
            }

            ui.separator();

            // Step counter
            ui.label(format!(
                "Step: {}/{}",
                app.back_plotter.current_step, app.back_plotter.step_count()
            ));
        });

        ui.separator();

        // Progress bar and percentage
        let progress = app.back_plotter.get_progress_percent() / 100.0;
        ui.add(
            egui::ProgressBar::new(progress)
                .show_percentage()
                .text(format!("{}%", (progress * 100.0) as i32)),
        );

        ui.separator();

        // Jump to step
        ui.horizontal(|ui| {
            ui.label("Jump to step:");
            let mut jump_step = app.back_plotter.current_step;
            if ui
                .add(
                    egui::Slider::new(&mut jump_step, 0..=app.back_plotter.step_count())
                        .show_value(true),
                )
                .changed()
            {
                let _ = app.back_plotter.jump_to_step(jump_step);
            }
        });

        ui.separator();

        // Simulation speed
        ui.horizontal(|ui| {
            ui.label("Speed:");
            let mut speed = app.back_plotter.simulation_speed;
            if ui
                .add(
                    egui::Slider::new(&mut speed, 0.1..=5.0)
                        .step_by(0.1)
                        .show_value(true),
                )
                .changed()
            {
                let _ = app.back_plotter.set_simulation_speed(speed);
            }
        });

        ui.separator();

        // Current step info
        if let Some(step_data) = app.back_plotter.get_current_step_data() {
            ui.group(|ui| {
                ui.heading("Current Step Info");
                ui.label(format!("Line: {}", step_data.line_number));
                ui.label(format!(
                    "Position: X={:.2}, Y={:.2}, Z={:.2}",
                    step_data.position.0, step_data.position.1, step_data.position.2
                ));
                ui.label(format!("Feed Rate: {:.0} mm/min", step_data.feed_rate));
                ui.label(format!("Spindle Speed: {:.0} RPM", step_data.spindle_speed));
                ui.label(format!(
                    "Move Type: {}",
                    match step_data.move_type {
                        crate::types::MoveType::Rapid => "Rapid (G0)",
                        crate::types::MoveType::Feed => "Feed (G1)",
                        crate::types::MoveType::Arc => "Arc (G2/G3)",
                    }
                ));
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_back_plot_ui_config_default() {
        let config = BackPlotUiConfig::default();
        assert!(config.show_step_counter);
        assert!(config.show_progress_bar);
        assert_eq!(config.auto_play_speed, 0.0);
    }

    #[test]
    fn test_back_plot_ui_config_with_auto_play() {
        let config = BackPlotUiConfig::with_auto_play(2.0);
        assert_eq!(config.auto_play_speed, 2.0);
    }

    #[test]
    fn test_show_back_plot_panel_compiles() {
        // This test just ensures the function compiles
        // Full integration tests would require egui test harness
    }
}
