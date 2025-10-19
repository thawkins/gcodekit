//! Status UI layout and integration.
//!
//! Provides the main UI panel that displays real-time status information,
//! integrating the status display widgets with the application layout.

use crate::communication::grbl_status::MachineStatus;
use crate::ui::{
    display_status_panel, display_status_bar, display_history_overview,
    display_position_trace, StateColors,
};
use egui::{Context, ScrollArea, Ui, Window};

/// Status panel configuration.
#[derive(Debug, Clone)]
pub struct StatusPanelConfig {
    /// Show status panel
    pub show_panel: bool,
    /// Show status bar at bottom
    pub show_bar: bool,
    /// Show history tabs
    pub show_history: bool,
    /// Show position trace
    pub show_trace: bool,
    /// Panel width as fraction (0.0-1.0)
    pub panel_width: f32,
    /// Use tabbed layout
    pub use_tabs: bool,
}

impl Default for StatusPanelConfig {
    fn default() -> Self {
        StatusPanelConfig {
            show_panel: true,
            show_bar: true,
            show_history: true,
            show_trace: true,
            panel_width: 0.25,
            use_tabs: true,
        }
    }
}

impl StatusPanelConfig {
    /// Create new config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to show status panel.
    pub fn with_panel(mut self, show: bool) -> Self {
        self.show_panel = show;
        self
    }

    /// Set whether to show status bar.
    pub fn with_bar(mut self, show: bool) -> Self {
        self.show_bar = show;
        self
    }

    /// Set whether to show history.
    pub fn with_history(mut self, show: bool) -> Self {
        self.show_history = show;
        self
    }

    /// Set whether to show trace.
    pub fn with_trace(mut self, show: bool) -> Self {
        self.show_trace = show;
        self
    }

    /// Set panel width as fraction.
    pub fn with_panel_width(mut self, width: f32) -> Self {
        self.panel_width = width.clamp(0.1, 0.5);
        self
    }

    /// Set whether to use tabbed layout.
    pub fn with_tabs(mut self, use_tabs: bool) -> Self {
        self.use_tabs = use_tabs;
        self
    }
}

/// Status panel state tracking.
#[derive(Debug, Clone)]
pub struct StatusPanelState {
    /// Currently active tab (0=status, 1=history, 2=trace)
    pub active_tab: usize,
    /// Scroll position in status panel
    pub scroll_position: f32,
}

impl Default for StatusPanelState {
    fn default() -> Self {
        StatusPanelState {
            active_tab: 0,
            scroll_position: 0.0,
        }
    }
}

/// Display status panel in UI.
///
/// # Arguments
/// * `ui` - egui Ui context
/// * `status` - Current machine status
/// * `config` - Panel configuration
/// * `state` - Panel state
/// * `colors` - State color scheme
pub fn display_status_panel_ui(
    ui: &mut Ui,
    status: &MachineStatus,
    config: &StatusPanelConfig,
    state: &mut StatusPanelState,
    colors: &StateColors,
) {
    ui.vertical(|ui| {
        if config.use_tabs {
            ui.horizontal(|ui| {
                if ui.selectable_label(state.active_tab == 0, "Status").clicked() {
                    state.active_tab = 0;
                }
                if ui.selectable_label(state.active_tab == 1, "Info").clicked() {
                    state.active_tab = 1;
                }
                if ui.selectable_label(state.active_tab == 2, "Details").clicked() {
                    state.active_tab = 2;
                }
            });
            ui.separator();
        }

        match state.active_tab {
            0 => {
                ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    display_status_panel(ui, status, colors);
                });
            }
            1 => {
                ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    display_position_info(ui, status);
                });
            }
            _ => {
                ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    display_advanced_info(ui, status);
                });
            }
        }
    });
}

/// Display position information.
fn display_position_info(ui: &mut Ui, status: &MachineStatus) {
    ui.group(|ui| {
        ui.heading("Position Information");

        ui.vertical(|ui| {
            ui.label("Machine Position (absolute):");
            ui.indent("machine_pos", |ui| {
                ui.monospace(format!(
                    "X: {:.4} mm\nY: {:.4} mm\nZ: {:.4} mm",
                    status.machine_position.x, status.machine_position.y, status.machine_position.z
                ));

                if let (Some(a), Some(b), Some(c)) =
                    (status.machine_position.a, status.machine_position.b, status.machine_position.c)
                {
                    ui.monospace(format!("A: {:.4}°\nB: {:.4}°\nC: {:.4}°", a, b, c));
                }
            });

            ui.separator();

            ui.label("Speeds and Rates:");
            ui.indent("speeds", |ui| {
                ui.monospace(format!(
                    "Feed Rate: {:.0} mm/min\nSpindle Speed: {:.0} RPM\nState: {:?}",
                    status.feed_speed.feed_rate, status.feed_speed.spindle_speed, status.state
                ));
            });
        });
    });
}

/// Display advanced information.
fn display_advanced_info(ui: &mut Ui, status: &MachineStatus) {
    ui.group(|ui| {
        ui.heading("Advanced Information");

        ui.vertical(|ui| {
            ui.label("Buffer State:");
            ui.indent("buffers", |ui| {
                ui.monospace(format!(
                    "Planner: {}/128 ({:.1}%)\nRX Buffer: {}/256 ({:.1}%)",
                    status.buffer_state.planner_buffer,
                    status.buffer_state.planner_fill_percent(),
                    status.buffer_state.rx_buffer,
                    status.buffer_state.rx_fill_percent()
                ));
            });

            ui.separator();

            ui.label("Override Status:");
            ui.indent("overrides", |ui| {
                ui.monospace(format!(
                    "Feed: {}%\nSpindle: {}%\nCoolant: {}%",
                    status.overrides.feed_override,
                    status.overrides.spindle_override,
                    status.overrides.coolant_override
                ));
            });

            ui.separator();

            ui.label("Pin States:");
            ui.indent("pins", |ui| {
                ui.monospace(format!(
                    "X Limit: {}\nY Limit: {}\nZ Limit: {}\nProbe: {}\nDoor: {}",
                    if status.pin_states.x_limit { "Active" } else { "Inactive" },
                    if status.pin_states.y_limit { "Active" } else { "Inactive" },
                    if status.pin_states.z_limit { "Active" } else { "Inactive" },
                    if status.pin_states.probe { "Active" } else { "Inactive" },
                    if status.pin_states.door_open { "Open" } else { "Closed" }
                ));
            });
        });
    });
}

/// Display history panel.
pub fn display_history_panel_ui(ui: &mut Ui, statuses: &[MachineStatus]) {
    if statuses.is_empty() {
        ui.label("No history available");
        return;
    }

    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
        display_history_overview(ui, statuses);
    });
}

/// Display position trace panel.
pub fn display_trace_panel_ui(ui: &mut Ui, statuses: &[MachineStatus]) {
    if statuses.is_empty() {
        ui.label("No position data available");
        return;
    }

    display_position_trace(ui, statuses);
}

/// Main status window.
pub fn display_status_window(
    ctx: &Context,
    open: &mut bool,
    status: Option<&MachineStatus>,
    history: &[MachineStatus],
    config: &StatusPanelConfig,
    state: &mut StatusPanelState,
) {
    if !*open {
        return;
    }

    let colors = StateColors::default();

    Window::new("Machine Status")
        .open(open)
        .default_size([400.0, 600.0])
        .show(ctx, |ui| {
            if let Some(status) = status {
                display_status_panel_ui(ui, status, config, state, &colors);
            } else {
                ui.label("Waiting for status...");
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = StatusPanelConfig::default();
        assert!(config.show_panel);
        assert!(config.show_bar);
        assert!(config.show_history);
        assert!(config.show_trace);
        assert_eq!(config.panel_width, 0.25);
        assert!(config.use_tabs);
    }

    #[test]
    fn test_config_with_panel() {
        let config = StatusPanelConfig::new().with_panel(false);
        assert!(!config.show_panel);
    }

    #[test]
    fn test_config_with_bar() {
        let config = StatusPanelConfig::new().with_bar(false);
        assert!(!config.show_bar);
    }

    #[test]
    fn test_config_with_history() {
        let config = StatusPanelConfig::new().with_history(false);
        assert!(!config.show_history);
    }

    #[test]
    fn test_config_with_trace() {
        let config = StatusPanelConfig::new().with_trace(false);
        assert!(!config.show_trace);
    }

    #[test]
    fn test_config_with_panel_width() {
        let config = StatusPanelConfig::new().with_panel_width(0.3);
        assert_eq!(config.panel_width, 0.3);
    }

    #[test]
    fn test_config_panel_width_bounds() {
        let config = StatusPanelConfig::new()
            .with_panel_width(0.05)
            .with_panel_width(0.8);
        assert!(config.panel_width >= 0.1);
        assert!(config.panel_width <= 0.5);
    }

    #[test]
    fn test_config_with_tabs() {
        let config = StatusPanelConfig::new().with_tabs(false);
        assert!(!config.use_tabs);
    }

    #[test]
    fn test_panel_state_default() {
        let state = StatusPanelState::default();
        assert_eq!(state.active_tab, 0);
        assert_eq!(state.scroll_position, 0.0);
    }
}
