//! Status history visualization.
//!
//! Provides widgets for displaying status history trends including position,
//! feed rate, spindle speed, and buffer fill as interactive charts.

use crate::communication::grbl_status::MachineStatus;
use egui::*;

/// Simple line chart for status trends.
#[derive(Debug, Clone)]
pub struct StatusTrendChart {
    /// Chart title
    pub title: String,
    /// Y-axis label
    pub y_label: String,
    /// Y-axis minimum value
    pub y_min: f32,
    /// Y-axis maximum value
    pub y_max: f32,
    /// Chart height in pixels
    pub height: f32,
    /// Whether to show grid
    pub show_grid: bool,
    /// Color for the line
    pub color: Color32,
}

impl Default for StatusTrendChart {
    fn default() -> Self {
        StatusTrendChart {
            title: "Trend".to_string(),
            y_label: "Value".to_string(),
            y_min: 0.0,
            y_max: 1000.0,
            height: 150.0,
            show_grid: true,
            color: Color32::from_rgb(100, 200, 255),
        }
    }
}

impl StatusTrendChart {
    /// Create new chart with title.
    pub fn new(title: impl Into<String>) -> Self {
        StatusTrendChart {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set Y-axis range.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.y_min = min;
        self.y_max = max;
        self
    }

    /// Set Y-axis label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = label.into();
        self
    }

    /// Set chart height.
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set line color.
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    /// Display the chart.
    pub fn ui(self, ui: &mut Ui, data: &[f32]) {
        ui.group(|ui| {
            ui.heading(&self.title);

            if data.is_empty() {
                ui.label("No data available");
                return;
            }

            let available_size = ui.available_size();
            let (rect, _response) =
                ui.allocate_exact_size(Vec2::new(available_size.x, self.height), Sense::hover());

            let painter = ui.painter();

            // Draw background
            painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 30));

            // Draw grid if enabled
            if self.show_grid {
                let grid_color = Color32::from_rgba_unmultiplied(100, 100, 100, 50);
                let grid_lines = 5;
                for i in 0..=grid_lines {
                    let y = rect.top() + (rect.height() / grid_lines as f32) * i as f32;
                    painter.line_segment(
                        [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                        Stroke::new(1.0, grid_color),
                    );
                }
            }

            // Draw data line
            if data.len() > 1 {
                let range = self.y_max - self.y_min;
                let point_spacing = rect.width() / (data.len() - 1) as f32;

                let mut points = Vec::new();
                for (i, &value) in data.iter().enumerate() {
                    let x = rect.left() + point_spacing * i as f32;
                    let normalized = (value - self.y_min) / range.max(0.001);
                    let clamped = normalized.clamp(0.0, 1.0);
                    let y = rect.bottom() - clamped * rect.height();
                    points.push(Pos2::new(x, y));
                }

                for i in 0..points.len() - 1 {
                    painter.line_segment([points[i], points[i + 1]], Stroke::new(2.0, self.color));
                }

                // Draw circles at data points
                let circle_size = 2.0;
                let start_idx = data.len().saturating_sub(10);
                for &point in &points[start_idx..] {
                    painter.circle_filled(point, circle_size, self.color);
                }
            }

            // Draw axis labels
            painter.text(
                Pos2::new(rect.left() - 30.0, rect.bottom()),
                Align2::RIGHT_BOTTOM,
                format!("{:.0}", self.y_min),
                egui::FontId::proportional(10.0),
                Color32::GRAY,
            );

            painter.text(
                Pos2::new(rect.left() - 30.0, rect.top()),
                Align2::RIGHT_TOP,
                format!("{:.0}", self.y_max),
                egui::FontId::proportional(10.0),
                Color32::GRAY,
            );
        });
    }
}

/// Display status history overview.
pub fn display_history_overview(ui: &mut Ui, statuses: &[MachineStatus]) {
    if statuses.is_empty() {
        ui.label("No status history available");
        return;
    }

    ui.group(|ui| {
        ui.heading("Status History");

        ui.columns(2, |columns| {
            // Feed rate trend
            let feedrates: Vec<f32> = statuses.iter().map(|s| s.feed_speed.feed_rate).collect();
            let max_feed = feedrates.iter().copied().fold(0.0, f32::max);
            StatusTrendChart::new("Feed Rate")
                .with_range(0.0, max_feed.max(1000.0))
                .with_label("mm/min")
                .with_height(120.0)
                .with_color(Color32::from_rgb(100, 255, 100))
                .ui(&mut columns[0], &feedrates);

            // Spindle speed trend
            let spindle_speeds: Vec<f32> = statuses
                .iter()
                .map(|s| s.feed_speed.spindle_speed)
                .collect();
            let max_spindle = spindle_speeds.iter().copied().fold(0.0, f32::max);
            StatusTrendChart::new("Spindle Speed")
                .with_range(0.0, max_spindle.max(5000.0))
                .with_label("RPM")
                .with_height(120.0)
                .with_color(Color32::from_rgb(255, 200, 100))
                .ui(&mut columns[1], &spindle_speeds);
        });

        ui.separator();

        ui.columns(2, |columns| {
            // Buffer fill trend
            let buffer_fills: Vec<f32> = statuses
                .iter()
                .map(|s| s.buffer_state.planner_buffer as f32)
                .collect();
            StatusTrendChart::new("Planner Buffer")
                .with_range(0.0, 128.0)
                .with_label("Blocks")
                .with_height(120.0)
                .with_color(Color32::from_rgb(100, 150, 255))
                .ui(&mut columns[0], &buffer_fills);

            // RX buffer trend
            let rx_fills: Vec<f32> = statuses
                .iter()
                .map(|s| s.buffer_state.rx_buffer as f32)
                .collect();
            StatusTrendChart::new("RX Buffer")
                .with_range(0.0, 256.0)
                .with_label("Bytes")
                .with_height(120.0)
                .with_color(Color32::from_rgb(255, 100, 200))
                .ui(&mut columns[1], &rx_fills);
        });
    });
}

/// Display position trace (XY plane).
pub fn display_position_trace(ui: &mut Ui, statuses: &[MachineStatus]) {
    if statuses.is_empty() {
        ui.label("No position history available");
        return;
    }

    ui.group(|ui| {
        ui.heading("Position Trace (XY Plane)");

        let available_size = ui.available_size();
        let height = (300.0_f32).min(available_size.y);
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(available_size.x, height), Sense::hover());

        let painter = ui.painter();

        // Draw background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 30));

        // Find bounds
        let x_values: Vec<f32> = statuses.iter().map(|s| s.machine_position.x).collect();
        let y_values: Vec<f32> = statuses.iter().map(|s| s.machine_position.y).collect();

        let x_min = x_values.iter().copied().fold(f32::INFINITY, f32::min);
        let x_max = x_values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let y_min = y_values.iter().copied().fold(f32::INFINITY, f32::min);
        let y_max = y_values.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        let x_range = (x_max - x_min).max(1.0);
        let y_range = (y_max - y_min).max(1.0);

        // Draw grid
        let grid_color = Color32::from_rgba_unmultiplied(100, 100, 100, 50);
        for i in 0..=4 {
            let x = rect.left() + (rect.width() / 4.0) * i as f32;
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, grid_color),
            );

            let y = rect.top() + (rect.height() / 4.0) * i as f32;
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, grid_color),
            );
        }

        // Draw trace
        let trace_color = Color32::from_rgb(100, 255, 100);
        for i in 0..statuses.len() {
            let norm_x = (statuses[i].machine_position.x - x_min) / x_range;
            let norm_y = (statuses[i].machine_position.y - y_min) / y_range;

            let screen_x = rect.left() + norm_x * rect.width();
            let screen_y = rect.bottom() - norm_y * rect.height();
            let point = Pos2::new(screen_x, screen_y);

            if i > 0 {
                let prev_norm_x = (statuses[i - 1].machine_position.x - x_min) / x_range;
                let prev_norm_y = (statuses[i - 1].machine_position.y - y_min) / y_range;

                let prev_screen_x = rect.left() + prev_norm_x * rect.width();
                let prev_screen_y = rect.bottom() - prev_norm_y * rect.height();
                let prev_point = Pos2::new(prev_screen_x, prev_screen_y);

                painter.line_segment([prev_point, point], Stroke::new(1.5, trace_color));
            }

            painter.circle_filled(point, 2.0, trace_color);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_chart_default() {
        let chart = StatusTrendChart::default();
        assert_eq!(chart.y_min, 0.0);
        assert_eq!(chart.y_max, 1000.0);
        assert_eq!(chart.height, 150.0);
        assert!(chart.show_grid);
    }

    #[test]
    fn test_trend_chart_with_range() {
        let chart = StatusTrendChart::new("Test").with_range(100.0, 500.0);
        assert_eq!(chart.y_min, 100.0);
        assert_eq!(chart.y_max, 500.0);
    }

    #[test]
    fn test_trend_chart_with_height() {
        let chart = StatusTrendChart::new("Test").with_height(200.0);
        assert_eq!(chart.height, 200.0);
    }

    #[test]
    fn test_trend_chart_with_color() {
        let chart = StatusTrendChart::new("Test").with_color(Color32::RED);
        assert_eq!(chart.color, Color32::RED);
    }
}
