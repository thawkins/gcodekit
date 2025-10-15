//! Find and Replace functionality for G-code editor
//!
//! Provides comprehensive search and replace capabilities with
//! regex support, case sensitivity, and whole word matching.

use regex::Regex;

/// Options for find operations
#[derive(Debug, Clone)]
pub struct FindOptions {
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Use regular expressions
    pub use_regex: bool,
    /// Match whole words only
    pub whole_word: bool,
    /// Wrap around at end of document
    pub wrap_around: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            use_regex: false,
            whole_word: false,
            wrap_around: true,
        }
    }
}

/// A match result from find operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindMatch {
    /// Line number (0-indexed)
    pub line: usize,
    /// Start column in line
    pub start_col: usize,
    /// End column in line
    pub end_col: usize,
    /// Matched text
    pub text: String,
}

/// Find and Replace engine
#[derive(Clone, Debug)]
pub struct FindReplace {
    /// Current search query
    pub query: String,
    /// Replacement text
    pub replace_text: String,
    /// Search options
    pub options: FindOptions,
    /// All matches found
    pub matches: Vec<FindMatch>,
    /// Current match index
    pub current_match: usize,
}

impl Default for FindReplace {
    fn default() -> Self {
        Self::new()
    }
}

impl FindReplace {
    /// Create a new find/replace instance
    pub fn new() -> Self {
        Self {
            query: String::new(),
            replace_text: String::new(),
            options: FindOptions::default(),
            matches: Vec::new(),
            current_match: 0,
        }
    }

    /// Perform find operation on content
    ///
    /// # Arguments
    /// * `content` - The text content to search
    ///
    /// # Returns
    /// Number of matches found
    pub fn find(&mut self, content: &str) -> usize {
        self.matches.clear();
        self.current_match = 0;

        if self.query.is_empty() {
            return 0;
        }

        if self.options.use_regex {
            self.find_regex(content)
        } else {
            self.find_plain(content)
        }
    }

    /// Find using plain text search
    fn find_plain(&mut self, content: &str) -> usize {
        let lines: Vec<&str> = content.lines().collect();
        let query = if self.options.case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        for (line_idx, line) in lines.iter().enumerate() {
            let search_line = if self.options.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            let mut start_pos = 0;
            while let Some(pos) = search_line[start_pos..].find(&query) {
                let actual_pos = start_pos + pos;
                
                // Check whole word match if needed
                if self.options.whole_word && !self.is_whole_word(line, actual_pos, query.len()) {
                    start_pos = actual_pos + 1;
                    continue;
                }

                self.matches.push(FindMatch {
                    line: line_idx,
                    start_col: actual_pos,
                    end_col: actual_pos + query.len(),
                    text: line[actual_pos..actual_pos + query.len()].to_string(),
                });

                start_pos = actual_pos + 1;
            }
        }

        self.matches.len()
    }

    /// Find using regular expression
    fn find_regex(&mut self, content: &str) -> usize {
        let re = match self.build_regex() {
            Ok(r) => r,
            Err(_) => return 0,
        };

        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            for capture in re.captures_iter(line) {
                if let Some(mat) = capture.get(0) {
                    self.matches.push(FindMatch {
                        line: line_idx,
                        start_col: mat.start(),
                        end_col: mat.end(),
                        text: mat.as_str().to_string(),
                    });
                }
            }
        }

        self.matches.len()
    }

    /// Build regex pattern from query and options
    fn build_regex(&self) -> Result<Regex, regex::Error> {
        let mut pattern = if self.options.use_regex {
            self.query.clone()
        } else {
            regex::escape(&self.query)
        };

        if self.options.whole_word {
            pattern = format!(r"\b{}\b", pattern);
        }

        if self.options.case_sensitive {
            Regex::new(&pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        }
    }

    /// Check if match is a whole word
    fn is_whole_word(&self, line: &str, start: usize, length: usize) -> bool {
        let before_is_boundary = start == 0 || {
            let prev_char = line.chars().nth(start.saturating_sub(1));
            prev_char.is_none() || !prev_char.unwrap().is_alphanumeric() && prev_char.unwrap() != '_'
        };

        let after_is_boundary = start + length >= line.len() || {
            let next_char = line.chars().nth(start + length);
            next_char.is_none() || !next_char.unwrap().is_alphanumeric() && next_char.unwrap() != '_'
        };

        before_is_boundary && after_is_boundary
    }

    /// Move to next match
    pub fn next_match(&mut self) -> Option<&FindMatch> {
        if self.matches.is_empty() {
            return None;
        }

        if self.current_match < self.matches.len() - 1 {
            self.current_match += 1;
        } else if self.options.wrap_around {
            self.current_match = 0;
        }

        Some(&self.matches[self.current_match])
    }

    /// Move to previous match
    pub fn prev_match(&mut self) -> Option<&FindMatch> {
        if self.matches.is_empty() {
            return None;
        }

        if self.current_match > 0 {
            self.current_match -= 1;
        } else if self.options.wrap_around {
            self.current_match = self.matches.len() - 1;
        }

        Some(&self.matches[self.current_match])
    }

    /// Get current match
    pub fn current(&self) -> Option<&FindMatch> {
        self.matches.get(self.current_match)
    }

    /// Replace current match
    ///
    /// # Arguments
    /// * `content` - Mutable reference to content
    ///
    /// # Returns
    /// Updated content with replacement
    pub fn replace_current(&mut self, content: &str) -> String {
        if self.matches.is_empty() {
            return content.to_string();
        }

        let mat = &self.matches[self.current_match];
        self.replace_match(content, mat)
    }

    /// Replace all matches
    ///
    /// # Arguments
    /// * `content` - The content to perform replacements on
    ///
    /// # Returns
    /// Tuple of (updated_content, number_of_replacements)
    pub fn replace_all(&self, content: &str) -> (String, usize) {
        if self.matches.is_empty() {
            return (content.to_string(), 0);
        }

        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut result = lines;
        let mut replacements = 0;

        // Process matches in reverse order to maintain indices
        for mat in self.matches.iter().rev() {
            if mat.line < result.len() {
                let line = &result[mat.line];
                let before = &line[..mat.start_col];
                let after = &line[mat.end_col..];
                result[mat.line] = format!("{}{}{}", before, self.replace_text, after);
                replacements += 1;
            }
        }

        (result.join("\n"), replacements)
    }

    /// Replace a single match
    fn replace_match(&self, content: &str, mat: &FindMatch) -> String {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut result = lines;

        if mat.line < result.len() {
            let line = &result[mat.line];
            let before = &line[..mat.start_col];
            let after = &line[mat.end_col..];
            result[mat.line] = format!("{}{}{}", before, self.replace_text, after);
        }

        result.join("\n")
    }

    /// Get match count
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get current match number (1-indexed)
    pub fn current_match_number(&self) -> usize {
        if self.matches.is_empty() {
            0
        } else {
            self.current_match + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_plain() {
        let mut fr = FindReplace::new();
        fr.query = "G0".to_string();
        
        let content = "G0 X10\nG1 Y20\nG0 Z30";
        let count = fr.find(content);
        
        assert_eq!(count, 2);
        assert_eq!(fr.matches[0].line, 0);
        assert_eq!(fr.matches[1].line, 2);
    }

    #[test]
    fn test_find_case_sensitive() {
        let mut fr = FindReplace::new();
        fr.query = "G0".to_string();
        fr.options.case_sensitive = true;
        
        let content = "G0 X10\ng0 Y20";
        let count = fr.find(content);
        
        assert_eq!(count, 1);
        assert_eq!(fr.matches[0].line, 0);
    }

    #[test]
    fn test_find_case_insensitive() {
        let mut fr = FindReplace::new();
        fr.query = "G0".to_string();
        fr.options.case_sensitive = false;
        
        let content = "G0 X10\ng0 Y20";
        let count = fr.find(content);
        
        assert_eq!(count, 2);
    }

    #[test]
    fn test_find_whole_word() {
        let mut fr = FindReplace::new();
        fr.query = "G0".to_string();
        fr.options.whole_word = true;
        
        let content = "G0 X10\nG01 Y20";
        let count = fr.find(content);
        
        assert_eq!(count, 1);
        assert_eq!(fr.matches[0].line, 0);
    }

    #[test]
    fn test_find_regex() {
        let mut fr = FindReplace::new();
        fr.query = r"G\d+".to_string();
        fr.options.use_regex = true;
        
        let content = "G0 X10\nG1 Y20\nM3 S1000";
        let count = fr.find(content);
        
        assert_eq!(count, 2);
    }

    #[test]
    fn test_next_match() {
        let mut fr = FindReplace::new();
        fr.query = "G".to_string();
        
        let content = "G0\nG1\nG2";
        fr.find(content);
        
        assert_eq!(fr.current_match, 0);
        
        fr.next_match();
        assert_eq!(fr.current_match, 1);
        
        fr.next_match();
        assert_eq!(fr.current_match, 2);
    }

    #[test]
    fn test_prev_match() {
        let mut fr = FindReplace::new();
        fr.query = "G".to_string();
        
        let content = "G0\nG1\nG2";
        fr.find(content);
        fr.current_match = 2;
        
        fr.prev_match();
        assert_eq!(fr.current_match, 1);
        
        fr.prev_match();
        assert_eq!(fr.current_match, 0);
    }

    #[test]
    fn test_replace_current() {
        let mut fr = FindReplace::new();
        fr.query = "G0".to_string();
        fr.replace_text = "G1".to_string();
        
        let content = "G0 X10\nG0 Y20";
        fr.find(content);
        
        let result = fr.replace_current(content);
        assert_eq!(result, "G1 X10\nG0 Y20");
    }

    #[test]
    fn test_replace_all() {
        let fr = FindReplace {
            query: "G0".to_string(),
            replace_text: "G1".to_string(),
            ..Default::default()
        };
        
        let content = "G0 X10\nG0 Y20\nG0 Z30";
        let mut fr_mut = fr.clone();
        fr_mut.find(content);
        
        let (result, count) = fr_mut.replace_all(content);
        assert_eq!(count, 3);
        assert!(result.contains("G1 X10"));
        assert!(result.contains("G1 Y20"));
        assert!(result.contains("G1 Z30"));
    }

    #[test]
    fn test_wrap_around() {
        let mut fr = FindReplace::new();
        fr.query = "G".to_string();
        fr.options.wrap_around = true;
        
        let content = "G0\nG1";
        fr.find(content);
        fr.current_match = 1;
        
        fr.next_match();
        assert_eq!(fr.current_match, 0); // Wrapped around
    }
}
