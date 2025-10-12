use crate::communication::{ConnectionState, GrblCommunication};
use eframe::egui;

pub fn show_connection_widget(ui: &mut egui::Ui, communication: &mut GrblCommunication) {
    ui.group(|ui| {
        ui.label("Connection");

        // Refresh ports button
        if ui.button("🔄 Refresh Ports").clicked() {
            communication.refresh_ports();
        }

        // Port selection
        egui::ComboBox::from_label("Serial Port")
            .selected_text(&communication.selected_port)
            .show_ui(ui, |ui| {
                for port in &communication.available_ports {
                    ui.selectable_value(&mut communication.selected_port, port.clone(), port);
                }
            });

        ui.horizontal(|ui| {
            let connect_enabled = !communication.selected_port.is_empty()
                && communication.connection_state != ConnectionState::Connected
                && communication.connection_state != ConnectionState::Connecting;

            let disconnect_enabled = communication.connection_state == ConnectionState::Connected;

            if ui
                .add_enabled(connect_enabled, egui::Button::new("Connect"))
                .clicked()
            {
                communication.connect_to_device();
            }

            if ui
                .add_enabled(disconnect_enabled, egui::Button::new("Disconnect"))
                .clicked()
            {
                communication.disconnect_from_device();
            }
        });

        // Status display
        let status_text = match communication.connection_state {
            ConnectionState::Disconnected => "Disconnected",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::Connected => "Connected",
            ConnectionState::Error => "Connection Error",
        };
        ui.colored_label(
            match communication.connection_state {
                ConnectionState::Connected => egui::Color32::GREEN,
                ConnectionState::Error => egui::Color32::RED,
                ConnectionState::Connecting => egui::Color32::YELLOW,
                _ => egui::Color32::GRAY,
            },
            format!("Status: {}", status_text),
        );

        if !communication.status_message.is_empty() {
            ui.label(&communication.status_message);
        }
    });
}
