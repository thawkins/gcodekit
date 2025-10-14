use crate::communication::{CncController, ConnectionState};
use eframe::egui;

pub fn show_connection_widget(ui: &mut egui::Ui, communication: &mut dyn CncController) {
    ui.group(|ui| {
        ui.label("Connection");

        ui.horizontal(|ui| {
            // Port selection
            let mut selected_port = communication.get_selected_port().to_string();
            egui::ComboBox::from_id_salt("serial_port_combobox")
                .selected_text(&selected_port)
                .width(ui.available_width() - 30.0) // Leave space for the button
                .show_ui(ui, |ui| {
                    for port in communication.get_available_ports() {
                        ui.selectable_value(&mut selected_port, port.clone(), port);
                    }
                });
            if selected_port != communication.get_selected_port() {
                communication.set_port(selected_port);
            }

            // Refresh ports button
            if ui.button("🔄").clicked() {
                communication.refresh_ports();
            }
        });

        ui.horizontal(|ui| {
            let connect_enabled = !communication.get_selected_port().is_empty()
                && *communication.get_connection_state() != ConnectionState::Connected
                && *communication.get_connection_state() != ConnectionState::Connecting;

            let disconnect_enabled =
                *communication.get_connection_state() == ConnectionState::Connected;

            if ui
                .add_enabled(connect_enabled, egui::Button::new("Connect"))
                .clicked()
            {
                let _ = communication.connect();
            }

            if ui
                .add_enabled(disconnect_enabled, egui::Button::new("Disconnect"))
                .clicked()
            {
                communication.disconnect();
            }
        });

        // Status display
        let status_text = match *communication.get_connection_state() {
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::Connected => "Connected",
            ConnectionState::Error => "Connection Error",
            ConnectionState::Recovering => "Recovering...",
        };
        ui.colored_label(
            match *communication.get_connection_state() {
                ConnectionState::Connected => egui::Color32::GREEN,
                ConnectionState::Error => egui::Color32::RED,
                ConnectionState::Connecting => egui::Color32::YELLOW,
                _ => egui::Color32::GRAY,
            },
            format!("Status: {}", status_text),
        );

        if !communication.get_status_message().is_empty() {
            ui.label(communication.get_status_message());
        }

        // Recovery status
        if communication.is_recovering() {
            ui.separator();
            ui.label("🔄 Recovery Active");

            let recovery_state = communication.get_recovery_state();
            if let Some(last_error) = &recovery_state.last_error {
                ui.colored_label(egui::Color32::RED, format!("Last Error: {}", last_error));
            }

            ui.label(format!(
                "Reconnect Attempts: {}",
                recovery_state.reconnect_attempts
            ));
            ui.label(format!(
                "Command Retries: {}",
                recovery_state.command_retry_count
            ));

            if !recovery_state.recovery_actions_taken.is_empty() {
                ui.label("Recovery Actions:");
                for action in &recovery_state.recovery_actions_taken {
                    ui.label(format!("• {:?}", action));
                }
            }

            if ui.button("Reset Recovery").clicked() {
                communication.reset_recovery_state();
            }
        }
    });
}

