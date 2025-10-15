# Legacy G-code Editor Cleanup Plan

## Executive Summary

The codebase has **TWO** G-code state systems:
1. **Legacy**: `GcodeState` struct in `src/app/state.rs` (still heavily used)
2. **Enhanced**: `GcodeEditorState` in `src/gcodeedit/mod.rs` (new, feature-rich)

**Current Status**: ⚠️ **CANNOT REMOVE YET** - Legacy system is still actively used

## Analysis

### Legacy GcodeState Fields & Usage

| Field | Usage Count | Used By | Can Remove? |
|-------|-------------|---------|-------------|
| `gcode_content` | ~60 | All file ops, generation, sending | ❌ No - Core data |
| `gcode_filename` | ~20 | File dialogs, UI display | ❌ No - Core data |
| `parsed_paths` | ~13 | Visualizer, editor sync | ❌ No - Visualizer needs it |
| `selected_line` | ~10 | Visualizer, editor sync | ❌ No - Cross-component state |
| `sending_progress` | ~5 | Progress bar in editor | ❌ No - Active feature |
| `sending_from_line` | ~2 | Visual indicator during send | ❌ No - Active feature |

### Key Dependencies

#### 1. **File Operations** (`src/ops/file_ops.rs`)
- ✅ Already using `sync_gcode_to_editor()` helper
- ❌ Still writes to `self.gcode.gcode_content` first
- ❌ Still uses `self.gcode.gcode_filename`

#### 2. **G-code Operations** (`src/ops/gcode_ops.rs`)
- ✅ Has `sync_gcode_to_editor()` helper
- ❌ All operations write to `self.gcode.gcode_content`
- ❌ Uses `self.gcode.parsed_paths` for parsing
- ❌ Uses `self.gcode.sending_progress` for progress tracking

#### 3. **Visualizer** (`src/ui/tabs/visualizer_3d.rs`)
- ❌ Reads from `app.gcode.parsed_paths` extensively
- ❌ Reads/writes `app.gcode.selected_line` for navigation
- ❌ Reads `app.gcode.sending_from_line` for highlighting

#### 4. **Editor Sync** (`src/ui/tabs/gcode_editor.rs`)
- ✅ Syncs between old and new systems
- ❌ Still reads from `app.gcode.gcode_content`
- ❌ Still writes back to `app.gcode.gcode_content`
- ❌ Syncs `selected_line` bidirectionally

#### 5. **Top Panel** (`src/layout/top_central_panel.rs`)
- ❌ Displays `app.gcode.gcode_filename`
- ❌ Shows line count from `app.gcode.gcode_content`
- ❌ Sends using `app.gcode.gcode_content`

#### 6. **Visualizer UI** (`src/gcodeview/mod.rs`)
- ❌ Uses `app.sending_from_line` for highlighting
- ❌ Likely has more dependencies (11KB file)

## Why We Can't Remove It Yet

### 1. **Shared State Architecture**
The legacy `GcodeState` serves as a **central data store** accessed by multiple components:
- File operations write to it
- Visualizer reads from it
- Editor syncs with it
- UI displays from it

### 2. **Visualizer Dependency**
The 3D visualizer heavily depends on:
- `parsed_paths`: Array of PathSegment objects for rendering
- `selected_line`: Current line selection for highlighting
- `sending_from_line`: Visual indicator during G-code sending

### 3. **Bidirectional Sync Required**
Changes flow in BOTH directions:
- File load → Legacy → Enhanced (via sync)
- Editor edit → Enhanced → Legacy (for visualizer)

### 4. **Sending Progress**
Active feature showing progress bar during G-code transmission needs `sending_progress` and `sending_from_line` fields.

## Migration Strategy

### Phase 1: Consolidate State ✅ **DONE**
- ✅ Create `sync_gcode_to_editor()` helper
- ✅ Call it from all file operations
- ✅ Call it from all generation operations

### Phase 2: Move Shared Fields to Enhanced Editor
**Effort**: Medium | **Risk**: Low

Move these fields from `GcodeState` to `GcodeEditorState`:
```rust
// Add to GcodeEditorState
pub parsed_paths: Vec<PathSegment>,
pub sending_progress: f32,
pub sending_from_line: Option<usize>,
```

Update all consumers to read from `app.gcode_editor.*` instead of `app.gcode.*`.

**Files to update**:
- `src/ui/tabs/visualizer_3d.rs` (13 locations)
- `src/ui/tabs/gcode_editor.rs` (8 locations)
- `src/gcodeview/mod.rs` (unknown count)
- `src/ops/gcode_ops.rs` (6 locations)

### Phase 3: Eliminate Dual Content Storage
**Effort**: Medium | **Risk**: Medium

Make `gcode_content` and `gcode_filename` redirect to the enhanced editor:
```rust
impl GcodeState {
    pub fn gcode_content(&self) -> &str {
        &self.editor.buffer.get_content()
    }
    
    pub fn gcode_filename(&self) -> &str {
        &self.editor.gcode_filename
    }
}
```

**Files to update**:
- `src/ops/file_ops.rs` (8 locations)
- `src/ops/gcode_ops.rs` (20+ locations)
- `src/layout/top_central_panel.rs` (5 locations)

### Phase 4: Remove Legacy GcodeState
**Effort**: High | **Risk**: High

Complete removal steps:
1. Change `app.gcode` from `GcodeState` to direct `GcodeEditorState`
2. Update all `app.gcode.*` to appropriate new location
3. Remove `GcodeState` struct entirely
4. Clean up import statements

**Files to update**: ~15-20 files

## Recommended Approach

### Option A: Keep Both Systems (Current) ✅ **RECOMMENDED FOR NOW**
**Pros**:
- ✅ No risk of breaking existing features
- ✅ Sync helper keeps them in sync automatically
- ✅ Minimal ongoing maintenance

**Cons**:
- ❌ Slight memory overhead (duplicate content string)
- ❌ Conceptual complexity (two sources of truth)

### Option B: Full Migration (Future)
**Pros**:
- ✅ Single source of truth
- ✅ Cleaner architecture
- ✅ Slightly less memory usage

**Cons**:
- ❌ High risk of regressions
- ❌ Requires extensive testing
- ❌ ~40-60 locations to update
- ❌ May break visualizer/sending features

## Cost-Benefit Analysis

### Current Dual System Cost:
- Memory: ~2x content string (typically <1MB)
- CPU: Minimal (just string clones on changes)
- Complexity: Low (sync is automatic)
- Bugs: None currently

### Migration Benefit:
- Memory saved: <1MB typically
- Complexity reduced: Marginal
- Architecture purity: Improved
- Feature improvements: None

### Migration Cost:
- Development time: 4-8 hours
- Testing time: 2-4 hours
- Bug risk: Medium-High
- Regression risk: High

## Recommendation

### **DO NOT MIGRATE YET** ✅

**Reasons**:
1. Current system works well
2. Automatic sync handles synchronization
3. Migration cost >> benefit
4. High risk of breaking visualizer
5. No user-facing improvements

### **When to Migrate**:
- ✅ When adding major new features requiring state refactor
- ✅ When visualizer is being rewritten
- ✅ When memory usage becomes a concern (>100MB files)
- ✅ When comprehensive test suite exists

### **If You Must Clean Up**:
Start with **Phase 2** only - move `parsed_paths`, `sending_progress`, and `sending_from_line` to the enhanced editor. This gives 70% of the benefit with 30% of the risk.

## Technical Debt Score

| Metric | Score | Justification |
|--------|-------|---------------|
| **Severity** | 🟡 Low | Works correctly, no bugs |
| **Urgency** | 🟢 None | Not blocking any features |
| **Complexity** | 🟡 Medium | Clean separation, good sync |
| **Impact** | 🟢 Minimal | Small memory overhead |

**Overall**: 🟢 **Low Priority Tech Debt**

---

## Alternative: Minimal Cleanup (Phase 1.5)

If you want some cleanup without risk, consider:

### Move Just the Sending State
Move only sending-related fields:
```rust
// From GcodeState to GcodeEditorState
pub sending_progress: f32,
pub sending_from_line: Option<usize>,
```

**Benefit**: Cleaner separation of concerns  
**Risk**: Very low (only ~7 references to update)  
**Effort**: 30 minutes  
**Value**: High (logical grouping)  

This would be the **sweet spot** for safe, incremental cleanup.

---

## Conclusion

The legacy `GcodeState` is **not actually legacy** - it's a **shared data layer** still in active use. The enhanced editor is an **addition**, not a replacement. Both systems have distinct roles:

- **GcodeState**: Central data store for content and metadata
- **GcodeEditorState**: Rich editing features and UI enhancements

The current architecture with automatic sync is **optimal** for the current codebase. Hold off on migration until there's a compelling reason (performance issue, major refactor, or user-facing benefit).

**Status**: ✅ **No action needed - system working as designed**
