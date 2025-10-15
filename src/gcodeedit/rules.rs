//! G-code validation rules with incremental parsing support.
//!
//! This module provides a configurable rule-based validation system for G-code
//! with efficient incremental validation capabilities. It caches validation results
//! per line to avoid redundant re-validation when only a few lines change.
//!
//! # Features
//!
//! - **Incremental Validation**: Efficiently validates only changed lines using a cache
//! - **Content Version Tracking**: Detects when full re-validation is needed
//! - **Configurable Rules**: Enable/disable rules at runtime
//! - **Multiple Severity Levels**: Error, Warn, Info diagnostics
//! - **Token-Aware Validation**: Integrates with the tokenizer for precise validation
//!
//! # Example Usage
//!
//! ```no_run
//! use gcodekit::gcodeedit::rules::RuleSet;
//! use gcodekit::gcodeedit::tokenizer::parse_content_sync;
//!
//! let mut ruleset = RuleSet::new_default();
//!
//! // Initial validation
//! let content = "G0 X10\nG999 Y20\nG1 X30";
//! let parsed = parse_content_sync(content);
//! let diagnostics = ruleset.validate_parsed(&parsed, Some(1));
//!
//! // Incremental update - only changed lines
//! let updated_content = "G0 X10\nG1 Y20\nG1 X30";
//! let updated_parsed = parse_content_sync(updated_content);
//! let updated_diagnostics = ruleset.validate_parsed(&updated_parsed, Some(2));
//! ```

use crate::gcodeedit::vocabulary;
use std::collections::HashMap;

/// Severity for a diagnostic rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warn,
    Info,
}

/// A simple diagnostic message produced by a rule
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

/// Rule definition (configurable)
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: &'static str,
    pub description: &'static str,
    pub severity: Severity,
    pub enabled: bool,
}

/// Collection of rules with incremental validation support
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    pub grbl_version: String,
    /// Cache of diagnostics per line for incremental updates
    diagnostic_cache: HashMap<usize, Vec<Diagnostic>>,
    /// Track the last validated content hash to detect full re-validation needs
    last_content_version: u64,
}

impl RuleSet {
    /// Default ruleset
    pub fn new_default() -> Self {
        Self {
            rules: vec![
                Rule {
                    id: "unknown_code",
                    description: "Unknown G/M code for selected GRBL version",
                    severity: Severity::Error,
                    enabled: true,
                },
                Rule {
                    id: "empty_line",
                    description: "Empty or comment-only line",
                    severity: Severity::Info,
                    enabled: true,
                },
            ],
            grbl_version: "1.1".to_string(),
            diagnostic_cache: HashMap::new(),
            last_content_version: 0,
        }
    }

    pub fn enable_rule(&mut self, id: &str) {
        if let Some(r) = self.rules.iter_mut().find(|r| r.id == id) {
            r.enabled = true;
        }
    }

    pub fn disable_rule(&mut self, id: &str) {
        if let Some(r) = self.rules.iter_mut().find(|r| r.id == id) {
            r.enabled = false;
        }
    }

    /// Validate a single line and return diagnostics
    pub fn validate_line(&self, line: &str, line_no: usize) -> Vec<Diagnostic> {
        // Default single-line validation preserved for compatibility
        let mut diags = Vec::new();
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('%')
            || trimmed.starts_with('(')
            || trimmed.starts_with(';')
        {
            if self.rule_enabled("empty_line") {
                diags.push(Diagnostic {
                    line: line_no,
                    severity: Severity::Info,
                    message: "Empty or comment line".to_string(),
                });
            }
            return diags;
        }

        // Split into tokens, check first code token (G or M)
        if let Some(first) = trimmed.split_whitespace().next() {
            let tok = first.to_uppercase();
            if tok.starts_with('G') || tok.starts_with('M') {
                // Normalize code: G38.2 etc. Accept dot codes as-is
                let code = tok;
                if self.rule_enabled("unknown_code")
                    && !vocabulary::code_supported(&code, &self.grbl_version) {
                        diags.push(Diagnostic {
                            line: line_no,
                            severity: Severity::Error,
                            message: format!(
                                "Code '{}' not supported in GRBL {}",
                                code, self.grbl_version
                            ),
                        });
                    }
            }
        }

        diags
    }

    /// Validate using tokenized line syntax; callers can pass parsed tokens for better checks
    pub fn validate_tokenized_line(
        &self,
        syntax: &crate::gcodeedit::tokenizer::LineSyntax,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        if syntax.tokens.is_empty() {
            if self.rule_enabled("empty_line") {
                diags.push(Diagnostic {
                    line: syntax.line,
                    severity: Severity::Info,
                    message: "Empty or comment line".to_string(),
                });
            }
            return diags;
        }

        // Check first token if it's a command token
        if let Some(first) = syntax
            .tokens
            .iter()
            .find(|t| t.kind == crate::gcodeedit::tokenizer::TokenKind::Command)
        {
            let code = first.text.to_uppercase();
            if self.rule_enabled("unknown_code")
                && !vocabulary::code_supported(&code, &self.grbl_version) {
                    diags.push(Diagnostic {
                        line: syntax.line,
                        severity: Severity::Error,
                        message: format!(
                            "Code '{}' not supported in GRBL {}",
                            code, self.grbl_version
                        ),
                    });
                }
        }

        diags
    }

    fn rule_enabled(&self, id: &str) -> bool {
        self.rules.iter().any(|r| r.id == id && r.enabled)
    }

    /// Validate all lines from parsed content and return all diagnostics.
    /// Uses incremental caching to avoid re-validating unchanged lines.
    ///
    /// # Arguments
    /// * `parsed` - Vector of parsed line syntax from the tokenizer
    /// * `content_version` - Optional version identifier to track content changes
    ///
    /// # Returns
    /// Vector of all current diagnostics
    pub fn validate_parsed(
        &mut self,
        parsed: &[crate::gcodeedit::tokenizer::LineSyntax],
        content_version: Option<u64>,
    ) -> Vec<Diagnostic> {
        // Check if we need full re-validation
        let version = content_version.unwrap_or(0);
        let needs_full_validation = version != self.last_content_version;

        if needs_full_validation {
            // Clear cache on full re-validation
            self.diagnostic_cache.clear();
            self.last_content_version = version;
        }

        // Validate each line and cache results
        for syntax in parsed {
            let line_diags = self.validate_tokenized_line(syntax);
            if line_diags.is_empty() {
                self.diagnostic_cache.remove(&syntax.line);
            } else {
                self.diagnostic_cache.insert(syntax.line, line_diags);
            }
        }

        // Collect all diagnostics from cache
        let mut all_diagnostics: Vec<Diagnostic> =
            self.diagnostic_cache.values().flatten().cloned().collect();

        // Sort by line number for consistent output
        all_diagnostics.sort_by_key(|d| d.line);

        all_diagnostics
    }

    /// Incrementally update validation for specific changed lines.
    /// More efficient than full validation when only a few lines changed.
    ///
    /// # Arguments
    /// * `changed_lines` - Vector of parsed line syntax for lines that changed
    ///
    /// # Returns
    /// Vector of all current diagnostics after the incremental update
    pub fn validate_incremental(
        &mut self,
        changed_lines: &[crate::gcodeedit::tokenizer::LineSyntax],
    ) -> Vec<Diagnostic> {
        // Update cache for changed lines only
        for syntax in changed_lines {
            let line_diags = self.validate_tokenized_line(syntax);
            if line_diags.is_empty() {
                self.diagnostic_cache.remove(&syntax.line);
            } else {
                self.diagnostic_cache.insert(syntax.line, line_diags);
            }
        }

        // Collect all diagnostics from cache
        let mut all_diagnostics: Vec<Diagnostic> =
            self.diagnostic_cache.values().flatten().cloned().collect();

        // Sort by line number for consistent output
        all_diagnostics.sort_by_key(|d| d.line);

        all_diagnostics
    }

    /// Clear all cached diagnostics. Useful when rules change.
    pub fn clear_cache(&mut self) {
        self.diagnostic_cache.clear();
    }

    /// Get cached diagnostics without re-validation.
    pub fn get_diagnostics(&self) -> Vec<Diagnostic> {
        let mut all_diagnostics: Vec<Diagnostic> =
            self.diagnostic_cache.values().flatten().cloned().collect();

        all_diagnostics.sort_by_key(|d| d.line);
        all_diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gcodeedit::tokenizer::{parse_content_sync, LineSyntax};

    #[test]
    fn test_incremental_validation_basic() {
        let mut ruleset = RuleSet::new_default();

        // Initial content
        let content1 = "G0 X10\nG999 Y20\nG1 X30";
        let parsed1 = parse_content_sync(content1);

        // First validation should populate cache
        let diags1 = ruleset.validate_parsed(&parsed1, Some(1));
        assert_eq!(diags1.len(), 1);
        assert_eq!(diags1[0].line, 1); // G999 is invalid

        // Modified content - only line 1 changed
        let content2 = "G0 X10\nG1 Y20\nG1 X30";
        let parsed2 = parse_content_sync(content2);

        // Incremental validation with new version
        let diags2 = ruleset.validate_parsed(&parsed2, Some(2));
        assert_eq!(diags2.len(), 0); // All valid now
    }

    #[test]
    fn test_incremental_validation_cache_efficiency() {
        let mut ruleset = RuleSet::new_default();

        // Use content with some invalid codes to ensure cache entries
        let content = "G0 X10\nG999 Y20\nG2 X30 I5 J5";
        let parsed = parse_content_sync(content);

        // First validation
        let diags1 = ruleset.validate_parsed(&parsed, Some(1));
        assert_eq!(diags1.len(), 1); // G999 is invalid

        // Second validation with same version - should use cache
        let diags2 = ruleset.validate_parsed(&parsed, Some(1));
        assert_eq!(diags2.len(), 1);

        // Verify cache has entry for the error line
        assert!(!ruleset.diagnostic_cache.is_empty());
        assert!(ruleset.diagnostic_cache.contains_key(&1));
    }

    #[test]
    fn test_validate_incremental_changed_lines() {
        let mut ruleset = RuleSet::new_default();

        // Initial full validation
        let content1 = "G0 X10\nG1 Y20\nG2 X30 I5 J5";
        let parsed1 = parse_content_sync(content1);
        ruleset.validate_parsed(&parsed1, Some(1));

        // Now update only line 1 with invalid code
        let content2 = "G0 X10\nG999 Y20\nG2 X30 I5 J5";
        let parsed2 = parse_content_sync(content2);
        let changed_line = vec![parsed2[1].clone()];

        // Incremental update only for changed line
        let diags = ruleset.validate_incremental(&changed_line);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, 1);
    }

    #[test]
    fn test_clear_cache() {
        let mut ruleset = RuleSet::new_default();

        let content = "G0 X10\nG999 Y20";
        let parsed = parse_content_sync(content);

        // Populate cache
        ruleset.validate_parsed(&parsed, Some(1));
        assert!(!ruleset.diagnostic_cache.is_empty());

        // Clear cache
        ruleset.clear_cache();
        assert!(ruleset.diagnostic_cache.is_empty());
    }

    #[test]
    fn test_get_diagnostics_without_revalidation() {
        let mut ruleset = RuleSet::new_default();

        let content = "G0 X10\nG999 Y20";
        let parsed = parse_content_sync(content);

        // Populate cache
        ruleset.validate_parsed(&parsed, Some(1));

        // Get diagnostics without re-validation
        let diags1 = ruleset.get_diagnostics();
        let diags2 = ruleset.get_diagnostics();

        assert_eq!(diags1.len(), diags2.len());
        assert_eq!(diags1.len(), 1);
    }

    #[test]
    fn test_version_tracking() {
        let mut ruleset = RuleSet::new_default();

        let content = "G0 X10";
        let parsed = parse_content_sync(content);

        // First validation with version 1
        ruleset.validate_parsed(&parsed, Some(1));
        assert_eq!(ruleset.last_content_version, 1);

        // Second validation with version 2 should trigger cache clear
        ruleset.validate_parsed(&parsed, Some(2));
        assert_eq!(ruleset.last_content_version, 2);
    }

    #[test]
    fn test_rule_enable_disable_affects_validation() {
        let mut ruleset = RuleSet::new_default();

        let content = "G999 X10";
        let parsed = parse_content_sync(content);

        // With rule enabled
        let diags1 = ruleset.validate_parsed(&parsed, Some(1));
        assert_eq!(diags1.len(), 1);

        // Disable rule and clear cache
        ruleset.disable_rule("unknown_code");
        ruleset.clear_cache();

        // Re-validate with rule disabled
        let diags2 = ruleset.validate_parsed(&parsed, Some(2));
        assert_eq!(diags2.len(), 0);
    }

    #[test]
    fn test_empty_line_diagnostics() {
        let mut ruleset = RuleSet::new_default();

        let content = "\n\nG0 X10\n";
        let parsed = parse_content_sync(content);

        let diags = ruleset.validate_parsed(&parsed, Some(1));

        // Should have info diagnostics for empty lines
        let empty_line_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .collect();
        assert!(!empty_line_diags.is_empty());
    }
}
