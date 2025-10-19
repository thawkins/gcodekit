//! Animated state indicator widget.
//!
//! Provides an animated indicator that pulses or blinks based on machine state,
//! useful for quick visual feedback of machine status.

use egui::{Color32, Painter, Rect, Sense, Ui, Vec2};
use std::time::{SystemTime, UNIX_EPOCH};

/// Animated state indicator with configurable appearance.
#[derive(Debug, Clone)]
pub struct AnimatedStateIndicator {
    /// Size of the indicator in pixels
    pub size: f32,
    /// Current color
    pub color: Color32,
    /// Whether to pulse (fade in/out)
    pub pulse: bool,
    /// Pulse frequency in Hz (pulses per second)
    pub pulse_frequency: f32,
    /// Whether to use a circular or square shape
    pub circular: bool,
}

impl Default for AnimatedStateIndicator {
    fn default() -> Self {
        AnimatedStateIndicator {
            size: 20.0,
            color: Color32::from_rgb(100, 255, 100),
            pulse: true,
            pulse_frequency: 2.0,
            circular: true,
        }
    }
}

impl AnimatedStateIndicator {
    /// Create a new indicator with default settings.
    pub fn new(color: Color32) -> Self {
        AnimatedStateIndicator {
            color,
            ..Default::default()
        }
    }

    /// Set the size of the indicator.
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set whether the indicator should pulse.
    pub fn with_pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    /// Set the pulse frequency.
    pub fn with_frequency(mut self, frequency: f32) -> Self {
        self.pulse_frequency = frequency.clamp(0.1, 10.0);
        self
    }

    /// Set whether to use circular shape.
    pub fn with_circular(mut self, circular: bool) -> Self {
        self.circular = circular;
        self
    }

    /// Get current opacity (0.0-1.0) based on pulse.
    fn get_opacity(&self) -> f32 {
        if !self.pulse {
            return 1.0;
        }

        // Get time in seconds since epoch
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f32();

        // Calculate pulse value (0.3 to 1.0)
        let pulse = (time * self.pulse_frequency * 2.0 * std::f32::consts::PI).sin();
        0.65 + pulse * 0.35 // Range: 0.3-1.0
    }

    /// Display the indicator in the UI.
    pub fn ui(self, ui: &mut Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(self.size, self.size), Sense::hover());

        let opacity = self.get_opacity();
        let color = Color32::from_rgba_unmultiplied(
            self.color.r(),
            self.color.g(),
            self.color.b(),
            (self.color.a() as f32 * opacity) as u8,
        );

        let painter = ui.painter();

        if self.circular {
            painter.circle_filled(rect.center(), self.size / 2.0, color);
            painter.circle_stroke(
                rect.center(),
                self.size / 2.0,
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 128)),
            );
        } else {
            painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
            painter.rect_stroke(
                rect,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 128)),
                egui::StrokeKind::Outside,
            );
        }

        response
    }
}

/// Status indicator with animation and labeling.
pub struct StatusIndicatorWidget {
    /// The animated indicator
    pub indicator: AnimatedStateIndicator,
    /// Label text
    pub label: String,
}

impl StatusIndicatorWidget {
    /// Create a new status indicator widget.
    pub fn new(label: impl Into<String>, color: Color32) -> Self {
        StatusIndicatorWidget {
            indicator: AnimatedStateIndicator::new(color),
            label: label.into(),
        }
    }

    /// Display the widget with label.
    pub fn ui(self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.indicator.ui(ui);
            ui.label(&self.label);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_default() {
        let indicator = AnimatedStateIndicator::default();
        assert_eq!(indicator.size, 20.0);
        assert!(indicator.pulse);
        assert!(indicator.circular);
    }

    #[test]
    fn test_indicator_with_size() {
        let indicator = AnimatedStateIndicator::default().with_size(30.0);
        assert_eq!(indicator.size, 30.0);
    }

    #[test]
    fn test_indicator_opacity_no_pulse() {
        let indicator = AnimatedStateIndicator::default().with_pulse(false);
        let opacity = indicator.get_opacity();
        assert_eq!(opacity, 1.0);
    }

    #[test]
    fn test_indicator_opacity_with_pulse() {
        let indicator = AnimatedStateIndicator::default().with_pulse(true);
        let opacity = indicator.get_opacity();
        assert!(opacity >= 0.3);
        assert!(opacity <= 1.0);
    }

    #[test]
    fn test_indicator_frequency_bounds() {
        let indicator = AnimatedStateIndicator::default()
            .with_frequency(0.05)
            .with_frequency(15.0);
        assert!(indicator.pulse_frequency >= 0.1);
        assert!(indicator.pulse_frequency <= 10.0);
    }

    #[test]
    fn test_status_indicator_widget() {
        let widget = StatusIndicatorWidget::new("Running", Color32::GREEN);
        assert_eq!(widget.label, "Running");
    }
}
