//! UI components for real-time status display.
//!
//! Provides egui-based widgets for displaying machine status, state indicators,
//! and status history in the main application UI.

use crate::communication::grbl_status::{MachineState, MachineStatus};
use egui::{Color32, RichText, Ui};
use std::time::Duration;

/// Color scheme for different machine states.
#[derive(Debug, Clone, Copy)]
pub struct StateColors {
    /// Idle state color (blue)
    pub idle: Color32,
    /// Running state color (green)
    pub running: Color32,
    /// Hold/pause state color (yellow)
    pub hold: Color32,
    /// Jog state color (cyan)
    pub jog: Color32,
    /// Alarm state color (red)
    pub alarm: Color32,
    /// Door open state color (orange)
    pub door: Color32,
    /// Check/sim state color (purple)
    pub check: Color32,
    /// Home state color (magenta)
    pub home: Color32,
    /// Sleep state color (gray)
    pub sleep: Color32,
}

impl Default for StateColors {
    fn default() -> Self {
        StateColors {
            idle: Color32::from_rgb(100, 150, 255),    // Blue
            running: Color32::from_rgb(100, 255, 100), // Green
            hold: Color32::from_rgb(255, 200, 0),      // Yellow
            jog: Color32::from_rgb(0, 255, 255),       // Cyan
            alarm: Color32::from_rgb(255, 50, 50),     // Red
            door: Color32::from_rgb(255, 150, 0),      // Orange
            check: Color32::from_rgb(200, 100, 255),   // Purple
            home: Color32::from_rgb(255, 0, 255),      // Magenta
            sleep: Color32::from_rgb(128, 128, 128),   // Gray
        }
    }
}

impl StateColors {
    /// Get color for a machine state.
    pub fn get_color(&self, state: MachineState) -> Color32 {
        match state {
            MachineState::Idle => self.idle,
            MachineState::Run => self.running,
            MachineState::Hold => self.hold,
            MachineState::Jog => self.jog,
            MachineState::Alarm => self.alarm,
            MachineState::Door => self.door,
            MachineState::Check => self.check,
            MachineState::Home => self.home,
            MachineState::Sleep => self.sleep,
            MachineState::Unknown => Color32::GRAY,
        }
    }
}

/// Display a machine state indicator with color and text.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `state` - Current machine state
/// * `colors` - Color scheme for states
///
/// # Example
/// ```ignore
/// display_state_indicator(ui, MachineState::Run, &StateColors::default());
/// ```
pub fn display_state_indicator(ui: &mut Ui, state: MachineState, colors: &StateColors) {
    let color = colors.get_color(state);
    let state_text = RichText::new(format!("{:?}", state))
        .size(18.0)
        .color(color)
        .strong();

    ui.label(state_text);
}

/// Display machine position with X, Y, Z coordinates.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
///
/// # Example
/// ```ignore
/// display_position(ui, &machine_status);
/// ```
pub fn display_position(ui: &mut Ui, status: &MachineStatus) {
    ui.heading("Position");
    ui.vertical(|ui| {
        ui.label(RichText::new(format!("X: {:.3} mm", status.machine_position.x)).monospace());
        ui.label(RichText::new(format!("Y: {:.3} mm", status.machine_position.y)).monospace());
        ui.label(RichText::new(format!("Z: {:.3} mm", status.machine_position.z)).monospace());

        if let (Some(a), Some(b), Some(c)) = (
            status.machine_position.a,
            status.machine_position.b,
            status.machine_position.c,
        ) {
            ui.separator();
            ui.label(RichText::new(format!("A: {:.3}°", a)).monospace());
            ui.label(RichText::new(format!("B: {:.3}°", b)).monospace());
            ui.label(RichText::new(format!("C: {:.3}°", c)).monospace());
        }
    });
}

/// Display feed rate and spindle speed.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
///
/// # Example
/// ```ignore
/// display_feed_and_speed(ui, &machine_status);
/// ```
pub fn display_feed_and_speed(ui: &mut Ui, status: &MachineStatus) {
    ui.heading("Feed & Speed");
    ui.vertical(|ui| {
        let feed_bar = (status.feed_speed.feed_rate / 5000.0_f32).min(1.0);
        ui.label(format!(
            "Feed Rate: {:.0} mm/min",
            status.feed_speed.feed_rate
        ));
        ui.add(egui::ProgressBar::new(feed_bar).desired_width(f32::INFINITY));

        ui.separator();

        let spindle_bar = (status.feed_speed.spindle_speed / 20000.0_f32).min(1.0);
        ui.label(format!(
            "Spindle Speed: {:.0} RPM",
            status.feed_speed.spindle_speed
        ));
        ui.add(egui::ProgressBar::new(spindle_bar).desired_width(f32::INFINITY));
    });
}

/// Display override percentages (feed, spindle, coolant).
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
///
/// # Example
/// ```ignore
/// display_overrides(ui, &machine_status);
/// ```
pub fn display_overrides(ui: &mut Ui, status: &MachineStatus) {
    ui.heading("Overrides");
    ui.vertical(|ui| {
        let feed_bar = status.overrides.feed_override as f32 / 200.0_f32;
        ui.label(format!(
            "Feed Override: {}%",
            status.overrides.feed_override
        ));
        ui.add(egui::ProgressBar::new(feed_bar).desired_width(f32::INFINITY));

        ui.separator();

        let spindle_bar = status.overrides.spindle_override as f32 / 200.0_f32;
        ui.label(format!(
            "Spindle Override: {}%",
            status.overrides.spindle_override
        ));
        ui.add(egui::ProgressBar::new(spindle_bar).desired_width(f32::INFINITY));

        ui.separator();

        let coolant_bar = status.overrides.coolant_override as f32 / 200.0_f32;
        ui.label(format!(
            "Coolant Override: {}%",
            status.overrides.coolant_override
        ));
        ui.add(egui::ProgressBar::new(coolant_bar).desired_width(f32::INFINITY));
    });
}

/// Display buffer status (planner and RX).
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
///
/// # Example
/// ```ignore
/// display_buffer_status(ui, &machine_status);
/// ```
pub fn display_buffer_status(ui: &mut Ui, status: &MachineStatus) {
    ui.heading("Buffers");
    ui.vertical(|ui| {
        let planner_percent = status.buffer_state.planner_fill_percent() as f32 / 100.0_f32;
        ui.label(format!(
            "Planner: {} / 128 ({:.0}%)",
            status.buffer_state.planner_buffer,
            planner_percent * 100.0
        ));
        ui.add(egui::ProgressBar::new(planner_percent).desired_width(f32::INFINITY));

        ui.separator();

        let rx_percent = status.buffer_state.rx_fill_percent() as f32 / 100.0_f32;
        ui.label(format!(
            "RX Buffer: {} / 256 ({:.0}%)",
            status.buffer_state.rx_buffer,
            rx_percent * 100.0
        ));
        ui.add(egui::ProgressBar::new(rx_percent).desired_width(f32::INFINITY));
    });
}

/// Display pin states (limits, probe, door, etc.).
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
///
/// # Example
/// ```ignore
/// display_pin_states(ui, &machine_status);
/// ```
pub fn display_pin_states(ui: &mut Ui, status: &MachineStatus) {
    ui.heading("Pin States");
    ui.vertical(|ui| {
        let active_color = Color32::from_rgb(255, 100, 100); // Red for active
        let inactive_color = Color32::from_rgb(100, 100, 100); // Gray for inactive

        let x_color = if status.pin_states.x_limit {
            active_color
        } else {
            inactive_color
        };
        ui.label(RichText::new("✓ X Limit").color(x_color));

        let y_color = if status.pin_states.y_limit {
            active_color
        } else {
            inactive_color
        };
        ui.label(RichText::new("✓ Y Limit").color(y_color));

        let z_color = if status.pin_states.z_limit {
            active_color
        } else {
            inactive_color
        };
        ui.label(RichText::new("✓ Z Limit").color(z_color));

        ui.separator();

        let probe_color = if status.pin_states.probe {
            active_color
        } else {
            inactive_color
        };
        ui.label(RichText::new("◆ Probe").color(probe_color));

        let door_color = if status.pin_states.door_open {
            active_color
        } else {
            inactive_color
        };
        ui.label(RichText::new("⚠ Door").color(door_color));
    });
}

/// Display job progress information.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
///
/// # Example
/// ```ignore
/// display_job_progress(ui, &machine_status);
/// ```
pub fn display_job_progress(ui: &mut Ui, status: &MachineStatus) {
    let total = status.feedback.total_lines();
    if total == 0 {
        ui.label("No job loaded");
        return;
    }

    ui.heading("Job Progress");
    ui.vertical(|ui| {
        let progress = status.feedback.progress_percent() as f32 / 100.0_f32;
        ui.label(format!(
            "{} / {} lines ({:.0}%)",
            status.feedback.lines_completed,
            total,
            progress * 100.0
        ));
        ui.add(egui::ProgressBar::new(progress).desired_width(f32::INFINITY));

        if status.feedback.lines_remaining > 0 {
            ui.separator();
            ui.label(format!(
                "Remaining: {} lines",
                status.feedback.lines_remaining
            ));
        }
    });
}

/// Display complete status panel with all information.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
/// * `colors` - Color scheme for states
///
/// # Example
/// ```ignore
/// display_status_panel(ui, &machine_status, &StateColors::default());
/// ```
pub fn display_status_panel(ui: &mut Ui, status: &MachineStatus, colors: &StateColors) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.heading("Machine Status");
            ui.separator();
            display_state_indicator(ui, status.state, colors);
        });

        ui.separator();

        ui.columns(2, |columns| {
            columns[0].vertical(|ui| {
                display_position(ui, status);
                ui.separator();
                display_feed_and_speed(ui, status);
            });

            columns[1].vertical(|ui| {
                display_overrides(ui, status);
                ui.separator();
                display_buffer_status(ui, status);
            });
        });

        ui.separator();

        ui.columns(2, |columns| {
            columns[0].vertical(|ui| {
                display_pin_states(ui, status);
            });

            columns[1].vertical(|ui| {
                display_job_progress(ui, status);
            });
        });
    });
}

/// Display a compact status bar at the bottom of the UI.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
/// * `colors` - Color scheme for states
///
/// # Example
/// ```ignore
/// display_status_bar(ui, &machine_status, &StateColors::default());
/// ```
pub fn display_status_bar(ui: &mut Ui, status: &MachineStatus, colors: &StateColors) {
    ui.horizontal(|ui| {
        let color = colors.get_color(status.state);
        ui.label(
            RichText::new(format!("  {}  ", status.state))
                .color(color)
                .strong(),
        );

        ui.separator();

        ui.label(format!(
            "Pos: X{:.2} Y{:.2} Z{:.2}",
            status.machine_position.x, status.machine_position.y, status.machine_position.z
        ));

        ui.separator();

        ui.label(format!(
            "Feed: {:.0} mm/min | Speed: {:.0} RPM",
            status.feed_speed.feed_rate, status.feed_speed.spindle_speed
        ));

        ui.separator();

        let progress = status.feedback.progress_percent();
        ui.label(format!("Progress: {}%", progress));

        if status.pin_states.has_alarm() {
            ui.separator();
            ui.label(RichText::new("⚠ ALARM").color(Color32::RED).strong());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_colors_defaults() {
        let colors = StateColors::default();
        assert_eq!(colors.idle, Color32::from_rgb(100, 150, 255));
        assert_eq!(colors.running, Color32::from_rgb(100, 255, 100));
        assert_eq!(colors.alarm, Color32::from_rgb(255, 50, 50));
    }

    #[test]
    fn test_state_colors_get_color() {
        let colors = StateColors::default();
        assert_eq!(colors.get_color(MachineState::Idle), colors.idle);
        assert_eq!(colors.get_color(MachineState::Run), colors.running);
        assert_eq!(colors.get_color(MachineState::Alarm), colors.alarm);
    }

    #[test]
    fn test_state_colors_all_states() {
        let colors = StateColors::default();
        assert_ne!(colors.get_color(MachineState::Idle), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Run), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Hold), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Jog), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Alarm), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Door), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Check), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Home), Color32::TRANSPARENT);
        assert_ne!(colors.get_color(MachineState::Sleep), Color32::TRANSPARENT);
        assert_ne!(
            colors.get_color(MachineState::Unknown),
            Color32::TRANSPARENT
        );
    }
}
