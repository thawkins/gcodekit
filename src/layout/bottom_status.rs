use crate::app::GcodeKitApp;
use crate::communication::{ConnectionState, grbl_status::MachineState, ControllerType};
use egui;

/// Renders the bottom status bar showing connection status, machine state,
/// controller type, current position, feed rate, spindle speed, and version information.
pub fn show_bottom_status(app: &mut GcodeKitApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Connection status with color
            let status_text = match app.machine.connection_state {
                ConnectionState::Disconnected => "Disconnected",
                ConnectionState::Connecting => "Connecting...",
                ConnectionState::Connected => "Connected",
                ConnectionState::Error => "Error",
                ConnectionState::Recovering => "Recovering...",
            };
            ui.colored_label(
                match app.machine.connection_state {
                    ConnectionState::Connected => egui::Color32::GREEN,
                    ConnectionState::Error => egui::Color32::RED,
                    ConnectionState::Connecting => egui::Color32::YELLOW,
                    _ => egui::Color32::GRAY,
                },
                format!("● {}", status_text),
            );

            ui.separator();

            // Machine state with color coding
            let (state_text, state_color) = match app.machine.realtime_status.state {
                MachineState::Idle => ("Idle", egui::Color32::GREEN),
                MachineState::Run => ("Running", egui::Color32::LIGHT_BLUE),
                MachineState::Hold => ("Hold", egui::Color32::YELLOW),
                MachineState::Jog => ("Jogging", egui::Color32::LIGHT_BLUE),
                MachineState::Alarm => ("Alarm", egui::Color32::RED),
                MachineState::Door => ("Door Open", egui::Color32::YELLOW),
                MachineState::Check => ("Check", egui::Color32::GRAY),
                MachineState::Home => ("Homing", egui::Color32::LIGHT_BLUE),
                MachineState::Sleep => ("Sleep", egui::Color32::GRAY),
                MachineState::Unknown => ("Unknown", egui::Color32::GRAY),
            };
            ui.colored_label(state_color, format!("🔧 {}", state_text));

            ui.separator();

            // Controller type
            match app.machine.controller_type {
                ControllerType::Grbl => ui.label("GRBL"),
            };

            ui.separator();

            // Real-time machine position (MPos)
            ui.label(format!(
                "MPos: X:{:.2} Y:{:.2} Z:{:.2}",
                app.machine.realtime_status.machine_position.x,
                app.machine.realtime_status.machine_position.y,
                app.machine.realtime_status.machine_position.z
            ));

            ui.separator();

            // Work position (WPos) if available
            if let Some(wpos) = app.machine.realtime_status.work_position {
                ui.label(format!(
                    "WPos: X:{:.2} Y:{:.2} Z:{:.2}",
                    wpos.x, wpos.y, wpos.z
                ));
            } else {
                ui.label("WPos: -");
            }

            ui.separator();

            // Feed rate
            if app.machine.realtime_status.feed_speed.feed_rate > 0.0 {
                ui.label(format!(
                    "F:{:.0} mm/min",
                    app.machine.realtime_status.feed_speed.feed_rate
                ));
            } else {
                ui.label("F: -");
            }

            ui.separator();

            // Spindle speed
            if app.machine.realtime_status.feed_speed.spindle_speed > 0.0 {
                ui.label(format!(
                    "S:{:.0} RPM",
                    app.machine.realtime_status.feed_speed.spindle_speed
                ));
            } else {
                ui.label("S: -");
            }

            ui.separator();

            // Port
            if !app.machine.selected_port.is_empty() {
                ui.label(format!("📍 {}", app.machine.selected_port));
            }

            // Version info on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("gcodekit v0.1.0");
            });
        });
    });
}
