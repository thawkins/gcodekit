# Code Folding Feature Removal

## Date
2025-10-17

## Rationale
Code folding features were removed from the G-code editor because GRBL does not support code blocks or structured programming constructs that would benefit from folding. G-code files are fundamentally linear sequences of machine commands, making code folding unnecessary complexity.

## Changes Made

### Source Code Changes

1. **src/gcodeedit/virtualized_view.rs**
   - Removed `FoldRegion` struct and implementation
   - Removed `FoldManager` struct and all folding management code
   - Removed `detect_folds()`, `toggle_fold_at()`, `is_line_folded()`, `get_region_at()`, `next_visible_line()`, `visible_lines()`, `clear()` methods
   - Removed all fold-related tests

2. **src/gcodeedit/mod.rs**
   - Removed `fold_manager` field from `GcodeEditorState`
   - Removed `detect_folds()`, `toggle_fold_at_line()`, `expand_all_folds()`, `collapse_all_folds()` methods
   - Removed fold keyboard shortcuts (Ctrl+], Ctrl+Shift+], Ctrl+Shift+[)
   - Removed fold UI buttons ("📁 Detect Folds", "➕ Expand All", "➖ Collapse All")
   - Removed "Code Folding" section from keyboard shortcuts help dialog
   - Removed fold icon rendering from gutter (changed from `format!("{}{} {:05}", fold_icon, icon, i + 1)` to `format!("{} {:05}", icon, i + 1)`)
   - Removed fold-related click detection in gutter
   - Removed call to `detect_folds()` in `load_gcode_file()`

3. **src/gcodeedit/editor.rs**
   - Updated module documentation (removed "and folding" reference)
   - Removed `folds: Vec<(usize, usize)>` field from `TextBufferCore`
   - Removed `toggle_fold()`, `is_line_folded()`, `get_fold_at()`, `is_fold_header()` methods
   - Removed `self.folds.clear()` from `set_content()` method

4. **src/gcodeedit/config.rs**
   - Removed `show_fold_markers` field from `EditorConfig`
   - Removed `auto_detect_folds` field from `EditorConfig`
   - Removed fold auto-detection code from `apply_to_editor()`

5. **src/ui/tabs/gcode_editor.rs**
   - Removed `detect_folds()` call
   - Removed `expand_all_folds()` call

6. **src/ops/gcode_ops.rs**
   - Removed `detect_folds()` call from `sync_gcode_to_editor()`
   - Removed `expand_all_folds()` call from `sync_gcode_to_editor()`

### Documentation Changes

1. **GUTTER_ALIGNMENT_FIX.md**
   - Simplified problem description (removed folding mismatch issue)
   - Simplified solution description (removed fold-related fixes)
   - Updated testing instructions (removed fold verification)
   - Updated technical details (removed fold-related changes)
   - Added note explaining folding feature removal

2. **docs/FOLDING_FEATURE_REMOVAL.md** (this file)
   - Created to document the rationale and changes

## Benefits

1. **Simplified Codebase**: Removed ~300 lines of code related to folding
2. **Eliminated Complexity**: Removed state management for fold regions
3. **Fixed Alignment Issues**: Removed potential source of gutter misalignment
4. **Improved Maintainability**: Fewer features to maintain and test
5. **Better Focus**: Editor now focused on core G-code editing functionality

## Impact

- **No User-Facing Feature Loss**: Folding was not useful for G-code files
- **Cleaner UI**: Removed unnecessary fold icons and buttons
- **Simpler Keyboard Shortcuts**: Removed fold-related shortcuts
- **Better Performance**: Less state to track and update

## Test Results

All tests pass after removal:
- 143 tests passed
- 0 tests failed
- Build completed successfully

## Future Considerations

If code folding becomes necessary in the future (e.g., for structured G-code with subroutines), it could be reimplemented with:
- Support for actual G-code subroutines (if GRBL adds support)
- Better integration with the text editor
- Proper handling of cursor positions in folded regions
- True line hiding (not just visual indicators)
