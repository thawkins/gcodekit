//! G-code editor core: buffer, cursor, selection, and undo/redo
//!
//! Provides a lightweight in-memory text buffer tailored for G-code editing and
//! integrates with the higher-level GcodeEditorState for rendering and validation.

use std::cmp::{max, min};

/// Represents a position in the text buffer as (line, column), zero-indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

impl Cursor {
    /// Create a new cursor at specified line and column.
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// Selection with inclusive start and exclusive end positions.
#[derive(Debug, Clone)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

impl Selection {
    pub fn new(start: Cursor, end: Cursor) -> Self {
        Self { start, end }
    }

    pub fn normalize(&mut self) {
        if (self.start.line > self.end.line)
            || (self.start.line == self.end.line && self.start.col > self.end.col)
        {
            std::mem::swap(&mut self.start, &mut self.end);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Simple edit operation for undo/redo
#[derive(Debug, Clone)]
pub enum EditOp {
    Insert {
        pos: Cursor,
        text: String,
    },
    Delete {
        start: Cursor,
        end: Cursor,
        deleted: String,
    },
}

/// In-memory text buffer specialized for G-code
#[derive(Clone, Debug)]
pub struct TextBufferCore {
    lines: Vec<String>,
    pub cursor: Cursor,
    pub selection: Option<Selection>,
    undo_stack: Vec<EditOp>,
    redo_stack: Vec<EditOp>,
}

impl Default for TextBufferCore {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBufferCore {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: Cursor::new(0, 0),
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Load content into buffer, resetting undo/redo stacks
    pub fn set_content(&mut self, content: &str) {
        self.lines = content.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor = Cursor::new(0, 0);
        self.selection = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Return a String with current content
    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    /// Insert text at the cursor position
    pub fn insert_text(&mut self, text: &str) {
        let line = &mut self.lines[self.cursor.line];
        let head = line[..min(self.cursor.col, line.len())].to_string();
        let tail = line[min(self.cursor.col, line.len())..].to_string();
        let mut new_lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        if new_lines.is_empty() {
            new_lines.push(String::new());
        }
        // Build replacement
        new_lines[0] = format!("{}{}", head, new_lines[0]);
        let last_idx = new_lines.len() - 1;
        new_lines[last_idx] = format!("{}{}", new_lines[last_idx], tail);

        // Replace current line with new lines
        self.lines
            .splice(self.cursor.line..=self.cursor.line, new_lines.clone());

        // Push undo op
        let end_cursor = Cursor::new(
            self.cursor.line + last_idx,
            new_lines[last_idx].len() - tail.len(),
        );
        self.undo_stack.push(EditOp::Insert {
            pos: self.cursor,
            text: text.to_string(),
        });
        self.redo_stack.clear();

        // Advance cursor
        self.cursor = end_cursor;
    }

    /// Delete range between start (inclusive) and end (exclusive)
    pub fn delete_range(&mut self, start: Cursor, end: Cursor) {
        let mut s = start;
        let mut e = end;
        // normalize
        if (s.line > e.line) || (s.line == e.line && s.col > e.col) {
            std::mem::swap(&mut s, &mut e);
        }

        // Extract deleted text
        if s.line == e.line {
            let line = &mut self.lines[s.line];
            let deleted = line[s.col..e.col].to_string();
            let new_line = format!("{}{}", &line[..s.col], &line[e.col..]);
            self.lines[s.line] = new_line;
            self.undo_stack.push(EditOp::Delete {
                start: s,
                end: e,
                deleted,
            });
        } else {
            let mut deleted_parts = Vec::new();
            deleted_parts.push(self.lines[s.line][s.col..].to_string());
            for ln in (s.line + 1)..e.line {
                deleted_parts.push(self.lines[ln].clone());
            }
            deleted_parts.push(self.lines[e.line][..e.col].to_string());
            let deleted = deleted_parts.join("\n");

            let prefix = self.lines[s.line][..s.col].to_string();
            let suffix = self.lines[e.line][e.col..].to_string();

            // Replace range with single merged line
            self.lines
                .splice(s.line..=e.line, vec![format!("{}{}", prefix, suffix)]);
            self.undo_stack.push(EditOp::Delete {
                start: s,
                end: e,
                deleted,
            });
        }

        self.redo_stack.clear();
        self.cursor = s;
    }

    /// Undo last edit
    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            match &op {
                EditOp::Insert { pos, text } => {
                    // delete inserted text
                    let mut end = *pos;
                    let lines: Vec<&str> = text.split('\n').collect();
                    if lines.len() == 1 {
                        end.col = pos.col + lines[0].len();
                    } else {
                        end.line = pos.line + lines.len() - 1;
                        end.col = lines.last().unwrap().len();
                    }
                    self.delete_range(*pos, end);
                }
                EditOp::Delete {
                    start,
                    end,
                    deleted,
                } => {
                    // re-insert deleted text
                    self.cursor = *start;
                    self.insert_text(deleted);
                }
            }
            self.redo_stack.push(op);
            true
        } else {
            false
        }
    }

    /// Redo last undone edit
    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            match &op {
                EditOp::Insert { pos, text } => {
                    self.cursor = *pos;
                    self.insert_text(text);
                }
                EditOp::Delete { start, end, .. } => {
                    self.delete_range(*start, *end);
                }
            }
            self.undo_stack.push(op);
            true
        } else {
            false
        }
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get line at index
    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.lines.get(index)
    }

    /// Get all lines
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}
