# Gutter Alignment Fix

## Problem
The gutter (line numbers and diagnostic icons) was not properly aligned with the text editor lines. The gutter rows appeared compressed compared to the TextEdit widget's line height.

## Root Cause
- The original implementation used UI widgets (buttons, labels) which have different spacing than TextEdit's internal line rendering
- TextEdit uses a Galley with specific row heights calculated from font metrics (font height + line gap)
- UI widgets use `item_spacing.y` which doesn't match the TextEdit's internal line spacing

## Solution
Replaced the widget-based gutter with a direct painting approach:

1. **Calculate Correct Row Height**: Use `font_size * 1.2` to match TextEdit's line spacing
   - TextEdit uses `font.row_height()` which includes line_gap
   - For monospace fonts, this is typically 1.2x the font size
   - Note: Can't call `fonts.row_height()` directly as it requires `&mut` access

2. **Direct Painting**: Use `allocate_exact_size()` and `painter.text()` instead of UI widgets
   - Removes all UI spacing interference
   - Gives pixel-perfect control over row positioning
   - Maintains consistent font rendering with the editor

3. **Preserved Features**:
   - ✅ Fold toggle (click on fold icons in first ~20px)
   - ✅ Line selection (click anywhere else on the row)
   - ✅ Diagnostic hover (shows error/warning/info messages)
   - ✅ Visual selection highlight
   - ✅ Fold icons changed to ▶/▼ for better alignment

## Testing
1. Build and run the application:
   ```bash
   cargo build --release
   ./target/release/gcodekit
   ```

2. Open a G-code file with multiple lines

3. Verify alignment:
   - Line numbers should align perfectly with text lines
   - Fold indicators should be at the correct vertical position
   - Diagnostic icons should align with their corresponding lines

## Fine-Tuning
If the alignment is still slightly off, adjust the multiplier in `src/gcodeedit/mod.rs`:

```rust
let row_height = font_id.size * 1.2;  // Try values between 1.15 and 1.25
```

Common values:
- `1.15` - Tighter spacing
- `1.2` - Standard (current)
- `1.25` - Looser spacing

## Technical Details
- **File**: `src/gcodeedit/mod.rs`
- **Lines**: ~1209-1310
- **Key Changes**:
  - Removed `ui.horizontal()` nesting for gutter items
  - Set `item_spacing.y = 0.0` to eliminate UI spacing
  - Use `allocate_exact_size()` with calculated `row_height`
  - Direct text painting with `painter.text()`
  - Fold click detection using pointer position
