use gcodekit::communication::CncController;
use eframe::egui;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_connection_widget_compiles() {
        // This test ensures the function compiles and has the expected signature
        // Full UI testing would require egui context mocking
        let _fn_exists = gcodekit::widgets::connection::show_connection_widget as fn(&mut egui::Ui, &mut dyn CncController);
    }
}
