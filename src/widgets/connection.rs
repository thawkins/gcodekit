use eframe::egui;
use crate::GcodeKitApp;

pub fn show_connection_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.label("Connection");

        // Refresh ports button
        if ui.button("🔄 Refresh Ports").clicked() {
            app.refresh_ports();
        }

        // Port selection
        egui::ComboBox::from_label("Serial Port")
            .selected_text(&app.selected_port)
            .show_ui(ui, |ui| {
                for port in &app.available_ports {
                    ui.selectable_value(&mut app.selected_port, port.clone(), port);
                }
            });

        ui.horizontal(|ui| {
            let connect_enabled = !app.selected_port.is_empty()
                && app.connection_state != crate::ConnectionState::Connected
                && app.connection_state != crate::ConnectionState::Connecting;

            let disconnect_enabled = app.connection_state == crate::ConnectionState::Connected;

            if ui.add_enabled(connect_enabled, egui::Button::new("Connect")).clicked() {
                app.connect_to_device();
            }

            if ui.add_enabled(disconnect_enabled, egui::Button::new("Disconnect")).clicked() {
                app.disconnect_from_device();
            }
        });

        // Status display
        let status_text = match app.connection_state {
            crate::ConnectionState::Disconnected => "Disconnected",
            crate::ConnectionState::Connecting => "Connecting...",
            crate::ConnectionState::Connected => "Connected",
            crate::ConnectionState::Error => "Connection Error",
        };
        ui.colored_label(
            match app.connection_state {
                crate::ConnectionState::Connected => egui::Color32::GREEN,
                crate::ConnectionState::Error => egui::Color32::RED,
                crate::ConnectionState::Connecting => egui::Color32::YELLOW,
                _ => egui::Color32::GRAY,
            },
            format!("Status: {}", status_text)
        );

        if !app.status_message.is_empty() {
            ui.label(&app.status_message);
        }
    });
}