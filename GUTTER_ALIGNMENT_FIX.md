# Gutter Alignment Fix

## Problem
The gutter (line numbers and diagnostic icons) was not properly aligned with the text editor lines. The gutter rows drifted progressively out of alignment as you scrolled down.

## Root Cause
The original implementation used UI widgets (buttons, labels) which have different spacing than TextEdit's internal line rendering. TextEdit uses a Galley with specific row heights calculated from font metrics (font height + line gap), while UI widgets use `item_spacing.y` which doesn't match the TextEdit's internal line spacing. An initial fix attempt used a 1.2x font size multiplier, but this caused drift over many lines.

## Solution

Extract the EXACT row height from TextEdit's galley after layout:

1. **Cache Galley Row Height**: 
   - Inside the TextEdit layouter (where we have mutable access to fonts), we create the galley
   - Extract row height: `galley.rect.height() / galley.rows.len()`
   - Store in `cached_row_height` field in `GcodeEditorState`
   - Use cached value for gutter rendering

2. **Direct Painting**: Use `allocate_exact_size()` and `painter.text()` instead of UI widgets
   - Removes all UI spacing interference  
   - Gives pixel-perfect control over row positioning
   - Uses the SAME row height that TextEdit uses

3. **Preserved Features**:
   - ✅ Line selection (click on gutter row)
   - ✅ Diagnostic hover (shows error/warning/info messages)
   - ✅ Visual selection highlight
   - ✅ Perfect line number alignment

## Testing
1. Build and run the application:
   ```bash
   cargo build --release
   ./target/release/gcodekit
   ```

2. Open a G-code file with 100+ lines

3. Verify alignment:
   - Line numbers should align perfectly with text lines from top to bottom
   - No drift should occur when scrolling down
   - Diagnostic icons should align with their corresponding lines

## Technical Details
- **File**: `src/gcodeedit/mod.rs`
- **Key Changes**:
  - Added `cached_row_height: Option<f32>` to `GcodeEditorState`
  - Extract row height in layouter: `galley.rect.height() / galley.rows.len()`
  - Use cached height in gutter: `self.cached_row_height.unwrap_or(font_id.size * 1.45)`
  - Set `item_spacing.y = 0.0` to eliminate UI spacing
  - Use `allocate_exact_size()` with exact `row_height` from galley
  - Direct text painting with `painter.text()`

## Why This Works
By using the galley's actual row height instead of an estimated multiplier, we ensure the gutter uses the EXACT same spacing as the TextEdit widget. This eliminates any drift caused by rounding errors or incorrect estimates.

## Note on Code Folding
Code folding features were removed from the editor (2025-10-17) as GRBL does not support code blocks, making folding unnecessary for G-code files. This simplification removed complexity and potential sources of alignment issues.
