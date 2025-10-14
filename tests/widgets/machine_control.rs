use gcodekit;
use egui;

#[test]
fn test_show_machine_control_widget_compiles() {
    // This test ensures the function compiles and has the expected signature
    // Full UI testing would require egui context mocking
    let _fn_exists = gcodekit::widgets::machine_control::show_machine_control_widget as fn(&mut egui::Ui, &mut gcodekit::GcodeKitApp);
}
