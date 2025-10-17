# Save and Save As Button Implementation

## Overview
Added "Save" and "Save As" buttons to the G-code editor interface, providing users with quick access to file-saving functionality. Also fixed the editor to properly handle text editing and save changes. Made G-code Editor the default tab and enabled creation of new files from scratch.

## Changes Made

### 1. UI Changes (`src/layout/top_central_panel.rs`)
- Added "💾 Save" button next to the "📁 Load File" button
- Added "💾 Save As..." button
- Both buttons are conditionally enabled based on content availability and file path

#### Button Behavior:
- **Save Button**: 
  - Enabled only when:
    - Editor has content (`!app.gcode_editor.buffer.get_content().is_empty()`)
    - A file path is associated with the content (`app.gcode_editor.current_file_path.is_some()`)
  - Saves to the current file path, overwriting if necessary
  - Displays success or error message in status bar

- **Save As Button**:
  - Enabled only when:
    - Editor has content (`!app.gcode_editor.buffer.get_content().is_empty()`)
  - Opens a file dialog to select a new save location
  - Warns about file overwrite (handled by native file dialog)
  - After successful save, updates the current file path and filename
  - Displays success or error message in status bar

### 2. Backend Improvements (`src/gcodeedit/mod.rs`)

#### Editor Text Editing Fix:
- **Fixed critical bug**: Editor was using a temporary string from `buffer.get_content()` which prevented text changes from being saved
- **Solution**: Now uses the persistent `gcode_content` field for the TextEdit widget
- When text changes are detected (`response.changed()`), the content is synced back to the buffer
- This ensures all edits are properly captured and saved

#### Empty Editor Initialization:
- **New Feature**: When editor content is empty, it now initializes with a single newline (`"\n"`)
- This allows users to start typing immediately to create new G-code files from scratch
- Previously showed a "No G-code file loaded" message that prevented editing
- Now provides a seamless experience for creating new files

#### Default Tab Changed (`src/types/enums.rs`):
- Changed default tab from `Designer` to `GcodeEditor`
- Users now start directly in the G-code editor when launching the app
- Provides immediate access to the most commonly used feature

#### `save_gcode_file()` Enhancement:
- Now syncs `gcode_content` field with buffer content after successful save
- Ensures consistency between buffer and stored content

#### `save_gcode_file_as()` Enhancement:
- Now syncs `gcode_content` field with buffer content after successful save
- Updates both `current_file_path` and `gcode_filename` after save
- File overwrite warning is automatically handled by `rfd::FileDialog::save_file()` on supported platforms

## User Experience

### Application Startup:
- G-code Editor tab is now selected by default
- Editor displays with an empty editable line
- User can immediately start typing G-code commands
- Save As button is enabled (content exists)
- Save button is disabled (no file path associated yet)

### Creating New Files:
1. Start typing G-code directly in the editor
2. Click "Save As" to save to a new file
3. File path becomes associated with the content
4. Save button becomes enabled for quick saves
5. Future edits can be saved with the Save button

### When No Content:
- Both Save and Save As buttons appear disabled (grayed out)

### When Content Exists (No File Path):
- Save button is disabled
- Save As button is enabled and functional

### When Content Exists (With File Path):
- Both Save and Save As buttons are enabled
- Save button saves to current path
- Save As button prompts for new path

### File Overwrite Protection:
The native file dialog (via `rfd` crate) automatically handles overwrite warnings on most platforms, asking the user to confirm before overwriting an existing file.

## Testing
- All existing tests pass successfully (194 tests)
- Build completes without errors
- Clippy shows no new warnings related to these changes
- Updated tests to reflect new default tab (GcodeEditor instead of Designer)

## Keyboard Shortcuts
The existing Ctrl+S shortcut for saving (defined in `src/gcodeedit/mod.rs` line 845) continues to work as expected.

## Future Enhancements
Potential improvements for future iterations:
1. Add visual indicator (e.g., asterisk) when file has unsaved changes
2. Add "unsaved changes" warning when loading a new file or closing
3. Add autosave functionality
4. Add file change detection (warn if file modified externally)
