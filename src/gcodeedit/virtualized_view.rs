//! Virtualized line rendering for large G-code files
//!
//! This module provides efficient rendering for large files by only rendering
//! visible lines, significantly improving performance for files with thousands of lines.

use egui::{Color32, Pos2, Rect, Response, Sense, Ui, Vec2};

/// Configuration for virtualized rendering
#[derive(Debug, Clone)]
pub struct VirtualizedConfig {
    /// Height of each line in pixels
    pub line_height: f32,
    /// Number of lines to render beyond visible area (for smooth scrolling)
    pub overscan_lines: usize,
    /// Maximum lines to render in one frame
    pub max_rendered_lines: usize,
}

impl Default for VirtualizedConfig {
    fn default() -> Self {
        Self {
            line_height: 14.0,
            overscan_lines: 10,
            max_rendered_lines: 1000,
        }
    }
}

/// State for virtualized scrolling
#[derive(Debug, Clone)]
pub struct VirtualizedState {
    /// Current scroll offset in pixels
    pub scroll_offset: f32,
    /// Total content height
    pub total_height: f32,
    /// First visible line index
    pub first_visible_line: usize,
    /// Last visible line index
    pub last_visible_line: usize,
    /// Viewport height
    pub viewport_height: f32,
}

impl Default for VirtualizedState {
    fn default() -> Self {
        Self {
            scroll_offset: 0.0,
            total_height: 0.0,
            first_visible_line: 0,
            last_visible_line: 0,
            viewport_height: 0.0,
        }
    }
}

impl VirtualizedState {
    /// Update state based on scroll position and total line count
    pub fn update(
        &mut self,
        scroll_offset: f32,
        viewport_height: f32,
        total_lines: usize,
        config: &VirtualizedConfig,
    ) {
        self.scroll_offset = scroll_offset;
        self.viewport_height = viewport_height;
        self.total_height = total_lines as f32 * config.line_height;

        // Calculate visible range with overscan
        let first_visible = (scroll_offset / config.line_height).floor() as usize;
        let last_visible = ((scroll_offset + viewport_height) / config.line_height).ceil() as usize;

        self.first_visible_line = first_visible.saturating_sub(config.overscan_lines);
        self.last_visible_line =
            (last_visible + config.overscan_lines).min(total_lines.saturating_sub(1));
    }

    /// Get the range of lines to render
    pub fn visible_range(&self) -> std::ops::Range<usize> {
        self.first_visible_line..self.last_visible_line.saturating_add(1)
    }

    /// Check if a line is currently visible
    pub fn is_line_visible(&self, line: usize) -> bool {
        line >= self.first_visible_line && line <= self.last_visible_line
    }

    /// Get Y position for a line
    pub fn line_y_position(&self, line: usize, config: &VirtualizedConfig) -> f32 {
        line as f32 * config.line_height
    }

    /// Scroll to make a line visible
    pub fn scroll_to_line(&mut self, line: usize, config: &VirtualizedConfig) -> f32 {
        let line_y = self.line_y_position(line, config);

        // Check if line is already visible
        if line_y < self.scroll_offset {
            // Line is above viewport
            self.scroll_offset = line_y;
        } else if line_y + config.line_height > self.scroll_offset + self.viewport_height {
            // Line is below viewport
            self.scroll_offset = line_y + config.line_height - self.viewport_height;
        }

        self.scroll_offset
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Number of lines rendered in last frame
    pub lines_rendered: usize,
    /// Time spent rendering (microseconds)
    pub render_time_us: u64,
    /// Total lines in file
    pub total_lines: usize,
    /// Memory used for visible lines (estimate in bytes)
    pub memory_used: usize,
}

impl PerformanceMetrics {
    /// Update metrics
    pub fn update(&mut self, lines_rendered: usize, render_time_us: u64, total_lines: usize) {
        self.lines_rendered = lines_rendered;
        self.render_time_us = render_time_us;
        self.total_lines = total_lines;
        // Estimate ~200 bytes per rendered line (rough estimate)
        self.memory_used = lines_rendered * 200;
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "Rendered {}/{} lines in {}μs (~{}KB)",
            self.lines_rendered,
            self.total_lines,
            self.render_time_us,
            self.memory_used / 1024
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtualized_state_update() {
        let mut state = VirtualizedState::default();
        let config = VirtualizedConfig::default();

        state.update(0.0, 400.0, 1000, &config);

        assert_eq!(state.first_visible_line, 0);
        assert!(state.last_visible_line > 0);
        assert!(state.last_visible_line < 50); // Should not render all lines
    }

    #[test]
    fn test_scroll_to_line() {
        let mut state = VirtualizedState::default();
        let config = VirtualizedConfig::default();

        state.update(0.0, 400.0, 1000, &config);

        // Scroll to line that's below viewport
        state.scroll_to_line(100, &config);
        assert!(state.scroll_offset > 0.0);
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::default();
        metrics.update(100, 5000, 10000);

        assert_eq!(metrics.lines_rendered, 100);
        assert_eq!(metrics.render_time_us, 5000);
        assert_eq!(metrics.total_lines, 10000);

        let summary = metrics.summary();
        assert!(summary.contains("100"));
        assert!(summary.contains("10000"));
    }
}
