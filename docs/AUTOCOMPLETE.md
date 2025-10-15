# G-code Auto-completion Feature

## Overview

The G-code editor now includes a comprehensive auto-completion system that provides intelligent, context-aware suggestions for G/M codes, parameters, and values.

## Features

### 1. **Command Completion (G/M Codes)**
- Suggests all valid G and M codes for the configured GRBL version
- Filters suggestions based on what you've typed (e.g., typing "G9" shows G90, G91, G92)
- Prioritizes commonly used codes (G0, G1, G90, M3, etc.)
- Shows descriptions for each code

### 2. **Parameter Completion**
- Context-aware parameter suggestions based on the current command
- For G0/G1: suggests X, Y, Z, F parameters
- For G2/G3 (arcs): suggests X, Y, Z, I, J, K, R, F parameters
- For G4 (dwell): suggests P parameter
- For M3/M4 (spindle): suggests S parameter

### 3. **Value Completion**
- Provides common value suggestions for certain parameters
- Feed rate (F) suggestions: 100, 200, 300, 500, 1000, 2000 mm/min
- Spindle speed (S) suggestions: 1000, 5000, 10000, 12000, 15000, 20000 RPM
- Can be extended to provide machine-specific limits and preferences

### 4. **GRBL Version Awareness**
- Automatically filters suggestions based on GRBL version (1.0, 1.1, 1.2)
- Only shows codes supported by your configured version
- Example: G38.2 (probe) is only shown for GRBL 1.1+

## Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Space` | Trigger auto-completion at cursor position |
| `↑` / `↓` | Navigate through suggestions |
| `Enter` | Accept the selected suggestion |
| `Esc` | Cancel auto-completion |

### How It Works

1. **Automatic Trigger**: Type any G or M code prefix, and auto-completion can be triggered
2. **Context Detection**: The system analyzes your cursor position and line content to determine what type of completion to provide
3. **Smart Filtering**: Suggestions are filtered in real-time based on what you've typed
4. **One-Click Accept**: Click on any suggestion to insert it immediately

## Examples

### Example 1: Command Completion
```
Type: G
Press: Ctrl+Space
See: G0, G1, G2, G3, G4, G10, G17, G18, G19, G20, G21, etc.
```

### Example 2: Parameter Completion
```
Type: G1 
Press: Ctrl+Space
See: X (X-axis position), Y (Y-axis position), Z (Z-axis position), F (Feed rate)
```

### Example 3: Arc Parameter Completion
```
Type: G2 
Press: Ctrl+Space
See: X, Y, Z, I (Arc center X offset), J (Arc center Y offset), K, R (Arc radius), F
```

### Example 4: Value Completion
```
Type: G1 X10 F
Press: Ctrl+Space
See: F100, F200, F300, F500, F1000, F2000
```

## Architecture

### Components

1. **AutoCompleter** (`src/gcodeedit/autocomplete.rs`)
   - Core completion engine
   - Analyzes context and provides suggestions
   - Filters by GRBL version

2. **CompletionContext** 
   - Tracks line content before/after cursor
   - Records recent commands for smart suggestions
   - Maintains modal state (future enhancement)

3. **CompletionItem**
   - Represents a single suggestion
   - Contains insert text, label, description, and priority

4. **UI Integration** (`src/gcodeedit/mod.rs`)
   - Popup window showing suggestions
   - Keyboard navigation
   - Visual feedback for selected item

### Completion Type Detection

The system automatically detects what type of completion is needed:

- **Command**: Beginning of line or after command
- **Parameter**: After a G/M code command
- **Value**: After a parameter letter (e.g., "F", "S")

## Testing

The auto-completion system includes comprehensive unit tests:

- ✅ Command completion (G and M codes)
- ✅ Parameter completion (context-aware)
- ✅ Value completion (F and S parameters)
- ✅ Partial matching
- ✅ GRBL version filtering
- ✅ Arc-specific parameters
- ✅ Completion type detection

Run tests with:
```bash
cargo test --lib gcodeedit::autocomplete::tests
```

## Future Enhancements

### Planned Features

1. **Modal State Tracking**
   - Track current positioning mode (G90/G91)
   - Track current motion mode (G0/G1/G2/G3)
   - Provide mode-specific suggestions

2. **Machine-Aware Limits**
   - Suggest values within machine travel limits
   - Warn about out-of-bounds values
   - Store user preferences

3. **Recent Value History**
   - Remember recently used values
   - Prioritize frequently used parameters

4. **Snippet Support**
   - Common G-code patterns (e.g., "safe retract")
   - User-defined snippets
   - Template expansion

5. **Documentation Integration**
   - Show inline documentation for codes
   - Link to GRBL documentation
   - Parameter examples

6. **Auto-formatting**
   - Automatically format inserted code
   - Apply spacing conventions
   - Normalize decimal places

## Configuration

Currently, the auto-completer is initialized with GRBL version "1.1" by default. This can be changed by updating the GcodeEditorState initialization:

```rust
autocomplete: crate::gcodeedit::autocomplete::AutoCompleter::new("1.2"),
```

Future versions will allow runtime configuration through the UI settings.

## API Reference

### AutoCompleter

```rust
pub struct AutoCompleter {
    grbl_version: String,
}

impl AutoCompleter {
    pub fn new(grbl_version: &str) -> Self
    
    pub fn get_suggestions(
        &self,
        line: &str,
        cursor_col: usize,
        context: &CompletionContext,
    ) -> Vec<CompletionItem>
}
```

### CompletionContext

```rust
pub struct CompletionContext {
    pub line_before_cursor: String,
    pub line_after_cursor: String,
    pub recent_commands: Vec<String>,
    pub modal_state: ModalState,
}
```

### CompletionItem

```rust
pub struct CompletionItem {
    pub insert_text: String,  // Text to insert
    pub label: String,         // Display label
    pub detail: String,        // Description
    pub sort_order: usize,     // Priority
    pub category: String,      // Category (G-code, M-code, Parameter, Value)
}
```

## Performance

The auto-completion system is designed for minimal performance impact:

- **Lazy Evaluation**: Suggestions are only computed when triggered
- **Efficient Filtering**: Uses simple string matching, not regex
- **Caching-Ready**: Architecture supports result caching (future enhancement)
- **Small Memory Footprint**: Vocabularies are static references

## Contributing

To add new codes or improve suggestions:

1. Update `src/gcodeedit/vocabulary.rs` with new codes
2. Add tests in `src/gcodeedit/autocomplete.rs`
3. Update this documentation

## License

Same as the main project.
