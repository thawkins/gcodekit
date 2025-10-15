//! Incremental G-code tokenizer and parser service
//!
//! This module provides a simple, debounced background tokenizer that converts
//! the editor buffer into tokens and a minimal parse structure suitable for
//! validation and editor features (folding, breadcrumbs, autocompletion).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

/// Token kinds for a G-code line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Command,   // e.g., G0, G1, M3
    Parameter, // e.g., X10.0, F100
    Comment,   // ; comment
    Unknown,
}

/// A single token with kind, span and text.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

/// Parsed representation for a line.
#[derive(Debug, Clone)]
pub struct LineSyntax {
    pub line: usize,
    pub tokens: Vec<Token>,
}

/// The tokenizer service holds the latest content snapshot and parsed lines.
pub struct TokenizerService {
    content: Arc<Mutex<String>>,
    parsed: Arc<Mutex<Vec<LineSyntax>>>,
    debounce_ms: u64,
    last_update: Arc<Mutex<Instant>>,
}

impl TokenizerService {
    /// Create a new tokenizer service with given debounce milliseconds.
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            content: Arc::new(Mutex::new(String::new())),
            parsed: Arc::new(Mutex::new(Vec::new())),
            debounce_ms,
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Submit new content snapshot for parsing. Parsing will run after the debounce delay.
    pub fn submit_content(&self, content: &str) {
        if let Ok(mut c) = self.content.lock() {
            *c = content.to_string();
        }
        if let Ok(mut t) = self.last_update.lock() {
            *t = Instant::now();
        }
    }

    /// Start background worker thread that performs debounced parsing.
    pub fn start_worker(&self) -> tokio::task::JoinHandle<()> {
        let content = self.content.clone();
        let parsed = self.parsed.clone();
        let debounce = self.debounce_ms;
        let last_update = self.last_update.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(debounce)).await;
                let since = { *last_update.lock().unwrap() };
                if since.elapsed() >= Duration::from_millis(debounce) {
                    let snapshot = { content.lock().unwrap().clone() };
                    let mut out = Vec::new();
                    for (i, line) in snapshot.lines().enumerate() {
                        let mut tokens = Vec::new();
                        let mut col = 0usize;
                        let s = line.trim_end();
                        if s.is_empty() {
                            out.push(LineSyntax { line: i, tokens });
                            continue;
                        }
                        if let Some(pos) = s.find(';') {
                            let comment = &s[pos..];
                            if pos > 0 {
                                let before = &s[..pos].trim();
                                for part in before.split_whitespace() {
                                    let kind = if part.starts_with('G') || part.starts_with('M') { TokenKind::Command } else { TokenKind::Parameter };
                                    tokens.push(Token { kind, text: part.to_string(), line: i, start_col: col, end_col: col + part.len() });
                                    col += part.len() + 1;
                                }
                            }
                            tokens.push(Token { kind: TokenKind::Comment, text: comment.to_string(), line: i, start_col: pos, end_col: s.len() });
                        } else {
                            for part in s.split_whitespace() {
                                let kind = if part.starts_with('G') || part.starts_with('M') { TokenKind::Command } else { TokenKind::Parameter };
                                tokens.push(Token { kind, text: part.to_string(), line: i, start_col: col, end_col: col + part.len() });
                                col += part.len() + 1;
                            }
                        }
                        out.push(LineSyntax { line: i, tokens });
                    }
                    if let Ok(mut p) = parsed.lock() {
                        *p = out;
                    }
                }
            }
        })
    }

    /// Get the latest parsed snapshot
    pub fn get_parsed(&self) -> Vec<LineSyntax> {
        self.parsed.lock().unwrap().clone()
    }
}

/// Synchronous parser for content; useful for immediate validation without background worker.
pub fn parse_content_sync(content: &str) -> Vec<LineSyntax> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let mut tokens = Vec::new();
        let mut col = 0usize;
        let s = line.trim_end();
        if s.is_empty() {
            out.push(LineSyntax { line: i, tokens });
            continue;
        }
        if let Some(pos) = s.find(';') {
            let comment = &s[pos..];
            if pos > 0 {
                let before = &s[..pos].trim();
                for part in before.split_whitespace() {
                    let kind = if part.starts_with('G') || part.starts_with('M') { TokenKind::Command } else { TokenKind::Parameter };
                    tokens.push(Token { kind, text: part.to_string(), line: i, start_col: col, end_col: col + part.len() });
                    col += part.len() + 1;
                }
            }
            tokens.push(Token { kind: TokenKind::Comment, text: comment.to_string(), line: i, start_col: pos, end_col: s.len() });
        } else {
            for part in s.split_whitespace() {
                let kind = if part.starts_with('G') || part.starts_with('M') { TokenKind::Command } else { TokenKind::Parameter };
                tokens.push(Token { kind, text: part.to_string(), line: i, start_col: col, end_col: col + part.len() });
                col += part.len() + 1;
            }
        }
        out.push(LineSyntax { line: i, tokens });
    }
    out
}
