use gcodekit::widgets::gcode_loading::show_gcode_loading_widget;
use gcodekit::GcodeKitApp;
use eframe::egui;

#[test]
fn test_show_gcode_loading_widget_compiles() {
    // This test ensures the function compiles and has the expected signature
    // Full UI testing would require egui context mocking
    let _fn_exists = show_gcode_loading_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
}
