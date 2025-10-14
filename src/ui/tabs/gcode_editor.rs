use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::text_edit::TextBuffer;

use crate::GcodeKitApp;

/// Shows the G-code editor tab
pub fn show_gcode_editor_tab(app: &mut GcodeKitApp, ui: &mut egui::Ui) {
    if app.gcode.gcode_content.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("No G-code file loaded. Use 'Load File' in the left panel.");
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut app.gcode.gcode_content)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(20)
                    .layouter(&mut |ui: &egui::Ui, string: &dyn TextBuffer, _wrap_width| {
                        let mut job = LayoutJob::default();
                        for (i, line) in string.as_str().lines().enumerate() {
                            // Line number
                            job.append(
                                &format!("{:05}: ", i + 1),
                                0.0,
                                TextFormat {
                                    font_id: egui::FontId::monospace(12.0),
                                    color: egui::Color32::DARK_GRAY,
                                    ..Default::default()
                                },
                            );
                            // Parse line for highlighting
                            let words: Vec<&str> = line.split_whitespace().collect();
                            for (j, word) in words.iter().enumerate() {
                                let color = if word.starts_with('G')
                                    && word.len() > 1
                                    && word[1..].chars().all(|c| c.is_ascii_digit())
                                {
                                    egui::Color32::BLUE
                                } else if word.starts_with('M')
                                    && word.len() > 1
                                    && word[1..].chars().all(|c| c.is_ascii_digit())
                                {
                                    egui::Color32::GREEN
                                } else if word.starts_with('X')
                                    || word.starts_with('Y')
                                    || word.starts_with('Z')
                                    || word.starts_with('I')
                                    || word.starts_with('J')
                                    || word.starts_with('K')
                                    || word.starts_with('F')
                                    || word.starts_with('S')
                                {
                                    egui::Color32::RED
                                } else if word.starts_with(';') {
                                    egui::Color32::GRAY
                                } else {
                                    egui::Color32::BLACK
                                };
                                job.append(
                                    word,
                                    0.0,
                                    TextFormat {
                                        font_id: egui::FontId::monospace(12.0),
                                        color,
                                        ..Default::default()
                                    },
                                );
                                if j < words.len() - 1 {
                                    job.append(" ", 0.0, TextFormat::default());
                                }
                            }
                            job.append("\n", 0.0, TextFormat::default());
                        }
                        ui.fonts_mut(|fonts| fonts.layout_job(job))
                    }),
            );
            if response.changed() {
                app.parse_gcode();
            }
        });
    }
}
