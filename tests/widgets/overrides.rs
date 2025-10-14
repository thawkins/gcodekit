// Integration test for overrides widget

use gcodekit::widgets::overrides::show_overrides_widget;
use gcodekit::GcodeKitApp;
use egui;

#[test]
fn test_show_overrides_widget_compiles() {
    // This test ensures the function compiles and has the expected signature
    // Full UI testing would require egui context mocking
    let _fn_exists = show_overrides_widget as fn(&mut egui::Ui, &mut GcodeKitApp);
}
