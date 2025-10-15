use crate::gcodeedit::vocabulary;

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

/// Collection of rules
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    pub grbl_version: String,
}

impl RuleSet {
    /// Default ruleset
    pub fn default() -> Self {
        Self {
            rules: vec![
                Rule { id: "unknown_code", description: "Unknown G/M code for selected GRBL version", severity: Severity::Error, enabled: true },
                Rule { id: "empty_line", description: "Empty or comment-only line", severity: Severity::Info, enabled: true },
            ],
            grbl_version: "1.1".to_string(),
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
        let mut diags = Vec::new();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('%') || trimmed.starts_with('(') || trimmed.starts_with(';') {
            if self.rule_enabled("empty_line") {
                diags.push(Diagnostic { line: line_no, severity: Severity::Info, message: "Empty or comment line".to_string() });
            }
            return diags;
        }

        // Split into tokens, check first code token (G or M)
        if let Some(first) = trimmed.split_whitespace().next() {
            let tok = first.to_uppercase();
            if tok.starts_with('G') || tok.starts_with('M') {
                // Normalize code: G38.2 etc. Accept dot codes as-is
                let code = tok;
                if self.rule_enabled("unknown_code") {
                    if !vocabulary::code_supported(&code, &self.grbl_version) {
                        diags.push(Diagnostic { line: line_no, severity: Severity::Error, message: format!("Code '{}' not supported in GRBL {}", code, self.grbl_version) });
                    }
                }
            }
        }

        diags
    }

    fn rule_enabled(&self, id: &str) -> bool {
        self.rules.iter().any(|r| r.id == id && r.enabled)
    }
}
