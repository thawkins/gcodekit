/// Error Recovery UI Widget
///
/// Displays error recovery status, job resumption controls, and error notifications.
/// Provides visual indicators for recovery state, automatic retry status, and
/// detailed error messages with recovery options.

use crate::GcodeKitApp;
use eframe::egui;

/// Shows the error recovery widget with status display and recovery controls.
///
/// Features:
/// - Error status indicator with severity color coding
/// - Job resumption button for paused/interrupted jobs
/// - Auto-recovery toggle with status display
/// - Recovery action history
/// - Error message details with recommendations
pub fn show_error_recovery_widget(ui: &mut egui::Ui, app: &mut GcodeKitApp) {
    ui.group(|ui| {
        ui.heading("🔧 Error Recovery & Job Resumption");

        // Get recovery state
        let recovery_state = app.machine.communication.get_recovery_state().clone();
        let recovery_config = app.machine.communication.get_recovery_config().clone();

        // Recovery status section
        ui.horizontal(|ui| {
            ui.label("Recovery Status:");
            
            let status_color = if recovery_state.reconnect_attempts > 0 {
                egui::Color32::from_rgb(255, 165, 0) // Orange for recovering
            } else if recovery_state.last_error.is_some() {
                egui::Color32::from_rgb(220, 50, 50) // Red for errors
            } else {
                egui::Color32::from_rgb(50, 200, 50) // Green for OK
            };

            let status_text = if recovery_state.reconnect_attempts > 0 {
                "🔄 RECOVERING"
            } else if recovery_state.last_error.is_some() {
                "⚠️ ERROR"
            } else {
                "✓ OK"
            };

            ui.label(
                egui::RichText::new(status_text)
                    .color(status_color)
                    .size(16.0),
            );
        });

        ui.separator();

        // Auto-recovery toggle
        ui.horizontal(|ui| {
            ui.label("Auto-Recovery:");
            
            let mut auto_enabled = recovery_config.auto_recovery_enabled;
            ui.checkbox(&mut auto_enabled, "Enabled");
            if auto_enabled != recovery_config.auto_recovery_enabled {
                let mut config = recovery_config.clone();
                config.auto_recovery_enabled = auto_enabled;
                app.machine.communication.set_recovery_config(config);
            }
            
            ui.label(format!(
                "Retry Delay: {}ms | Max Reconnects: {}",
                recovery_config.reconnect_delay_ms,
                recovery_config.max_reconnect_attempts
            ));
        });

        ui.separator();

        // Error details section
        if let Some(error_msg) = &recovery_state.last_error {
            ui.group(|ui| {
                ui.label(egui::RichText::new("⚠️ Latest Error").size(14.0).strong());
                ui.text_edit_multiline(&mut error_msg.as_str());
            });

            ui.horizontal(|ui| {
                ui.label(format!(
                    "Reconnect Attempts: {}/{}",
                    recovery_state.reconnect_attempts, recovery_config.max_reconnect_attempts
                ));

                if let Some(last_attempt) = recovery_state.last_reconnect_attempt {
                    let duration_ms = last_attempt.elapsed().as_millis();
                    ui.label(format!("Last Attempt: {}ms ago", duration_ms));
                }
            });

            // Manual recovery button
            ui.horizontal(|ui| {
                if ui.button("🔄 Attempt Recovery Now").clicked() {
                    if let Ok(action) = app
                        .machine
                        .communication
                        .attempt_recovery(error_msg)
                    {
                        app.machine.status_message =
                            format!("Recovery action: {:?}", action);
                        app.log_console(&format!("Recovery initiated: {:?}", action));
                    }
                }

                if ui.button("🗑️ Discard Error").clicked() {
                    app.machine.communication.reset_recovery_state();
                    app.machine.status_message = "Error state cleared".to_string();
                    app.log_console("Error recovery state reset");
                }
            });
        } else {
            ui.label("✓ No errors recorded");
        }

        ui.separator();

        // Job resumption section
        if recovery_state.reconnect_attempts > 0 || recovery_state.last_error.is_some() {
            ui.group(|ui| {
                ui.label(
                    egui::RichText::new("📋 Job Resumption")
                        .size(14.0)
                        .strong(),
                );

                ui.label("Ready to resume job after recovery");

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            recovery_state.last_error.is_some(),
                            egui::Button::new(
                                egui::RichText::new("▶️ Resume Job")
                                    .size(12.0),
                            ),
                        )
                        .clicked()
                    {
                        // Resume job - send cycle start command
                        app.machine.communication.resume_job();
                        app.machine.status_message = "Job resumption initiated".to_string();
                        app.log_console("Job resumption initiated after recovery");
                    }

                    if ui
                        .add_enabled(
                            recovery_state.last_error.is_some(),
                            egui::Button::new(
                                egui::RichText::new("⏹️ Abort Job")
                                    .size(12.0),
                            ),
                        )
                        .clicked()
                    {
                        // Abort job
                        app.machine.communication.emergency_stop();
                        app.machine.communication.reset_recovery_state();
                        app.machine.status_message = "Job aborted".to_string();
                        app.log_console("Job aborted");
                    }
                });
            });

            ui.separator();
        }

        // Recovery actions history
        if !recovery_state.recovery_actions_taken.is_empty() {
            ui.group(|ui| {
                ui.label(
                    egui::RichText::new("📜 Recovery History")
                        .size(12.0)
                        .strong(),
                );

                ui.separator();

                for (idx, action) in recovery_state.recovery_actions_taken.iter().enumerate() {
                    ui.label(format!("{}. {:?}", idx + 1, action));
                }

                if ui.button("Clear History").clicked() {
                    app.machine.communication.reset_recovery_state();
                    app.log_console("Recovery history cleared");
                }
            });
        }

        ui.separator();

        // Settings
        ui.collapsing("⚙️ Recovery Settings", |ui| {
            ui.label("Configure automatic recovery behavior:");

            ui.horizontal(|ui| {
                ui.label("Reconnect Delay (milliseconds):");
                let mut delay = recovery_config.reconnect_delay_ms as i32;
                if ui.add(egui::Slider::new(&mut delay, 100..=10000)).changed() {
                    let mut config = recovery_config.clone();
                    config.reconnect_delay_ms = delay as u64;
                    app.machine.communication.set_recovery_config(config);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Max Reconnect Attempts:");
                let mut max_attempts = recovery_config.max_reconnect_attempts as i32;
                if ui.add(egui::Slider::new(&mut max_attempts, 1..=10)).changed() {
                    let mut config = recovery_config.clone();
                    config.max_reconnect_attempts = max_attempts as u32;
                    app.machine.communication.set_recovery_config(config);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Max Command Retries:");
                let mut max_retries = recovery_config.max_command_retries as i32;
                if ui.add(egui::Slider::new(&mut max_retries, 1..=10)).changed() {
                    let mut config = recovery_config.clone();
                    config.max_command_retries = max_retries as u32;
                    app.machine.communication.set_recovery_config(config);
                }
            });

            ui.horizontal(|ui| {
                let mut reset_critical = recovery_config.reset_on_critical_error;
                ui.checkbox(&mut reset_critical, "Reset on critical error");
                if reset_critical != recovery_config.reset_on_critical_error {
                    let mut config = recovery_config.clone();
                    config.reset_on_critical_error = reset_critical;
                    app.machine.communication.set_recovery_config(config);
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_error_recovery_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        let _fn_exists = show_error_recovery_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
    }
}
