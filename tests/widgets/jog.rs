use gcodekit::widgets::jog::show_jog_widget;
use gcodekit::GcodeKitApp;
use eframe::egui;

#[test]
fn test_show_jog_widget_compiles() {
    // This test ensures the function compiles and has the expected signature
    // Full UI testing would require egui context mocking
    let _fn_exists = show_jog_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
}
