use crate::app::GcodeKitApp;
use crate::communication::{grbl, ConnectionState};
use egui;
#[allow(unused_imports)]
use std::any::Any;

/// Renders the bottom status bar showing connection status, machine state,
/// controller type, current position, and version information.
pub fn show_bottom_status(app: &mut GcodeKitApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Connection status
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
                format!("Status: {}", status_text),
            );

            ui.separator();

            // Device state (machine state from GRBL)
            let machine_state = if let Some(grbl_comm) = app
                .machine
                .communication
                .as_any()
                .downcast_ref::<grbl::GrblCommunication>()
            {
                format!("State: {:?}", grbl_comm.current_status.machine_state)
            } else {
                "State: Unknown".to_string()
            };
            ui.label(machine_state);

            ui.separator();

            // Controller type
            ui.label(format!("Controller: {:?}", app.machine.controller_type));

            ui.separator();

            // Current position
            ui.label(format!(
                "Position: {}",
                app.machine.current_position.format()
            ));

            ui.separator();

            // Version - TODO: get from thread
            // Selected port
            if !app.machine.selected_port.is_empty() {
                ui.label(format!("Port: {}", app.machine.selected_port));
            }

            // Version info on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("gcodekit v0.1.0");
            });
        });
    });
}
