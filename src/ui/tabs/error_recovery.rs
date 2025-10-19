/// Error Recovery Tab
///
/// Displays error recovery status, job resumption controls, and error notifications.

use crate::widgets::error_recovery::show_error_recovery_widget;
use crate::GcodeKitApp;
use eframe::egui;

/// Shows the error recovery tab with all recovery controls and status.
pub fn show_error_recovery_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        show_error_recovery_widget(ui, app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_error_recovery_tab_compiles() {
        let _fn_exists = show_error_recovery_tab as fn(&mut GcodeKitApp, &mut egui::Ui);
    }
}
