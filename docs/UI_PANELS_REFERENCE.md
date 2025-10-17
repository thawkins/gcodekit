# GcodeKit UI Panels and Widgets Reference

## Overview
This document provides a comprehensive list of all UI panels, tabs, and widgets in the GcodeKit application.

---

## Main Application Layout

### Top-Level Panels

1. **Top Menu Bar** (`src/layout/top_menu.rs`)
   - File operations (New, Open, Save, Save As, Exit)
   - Edit operations (Undo, Redo, Cut, Copy, Paste)
   - View options (panels visibility toggles)
   - Help menu

2. **Top Central Panel** (`src/layout/top_central_panel.rs`)
   - Quick action buttons (Refresh, Load File, Save, Save As)
   - G-code file controls
   - Send to Device controls
   - Connection status indicator
   - Current position display

3. **Left Panel** (`src/layout/left_panel.rs`)
   - Machine Control section
   - Contains: Connection, Jog, and Overrides widgets

4. **Right Panel** (`src/layout/right_panel.rs`)
   - CAM Functions section
   - Contains all CAM operation widgets

5. **Center Panel** (`src/layout/center_panel.rs`)
   - Tabbed interface for main content
   - Material database dialog

6. **Bottom Status Bar** (`src/layout/bottom_status.rs`)
   - Controller type and port information
   - Status messages

---

## Center Panel Tabs

The center panel contains six main tabs:

### 1. **G-code Editor Tab** (Default) 📝
   - **File**: `src/ui/tabs/gcode_editor.rs`
   - **Features**:
     - Syntax-highlighted G-code editor
     - Line numbers with diagnostic icons
     - Search and replace functionality
     - Code folding
     - Auto-completion
     - Find & Replace dialog
     - Keyboard shortcuts help
     - Real-time validation

### 2. **3D Visualizer Tab** 🔲
   - **File**: `src/ui/tabs/visualizer_3d.rs`
   - **Features**:
     - 3D visualization of toolpaths
     - Camera controls
     - Zoom, pan, rotate
     - Path highlighting
     - Work coordinate display

### 3. **Device Console Tab** 💻
   - **File**: `src/ui/tabs/device_console.rs`
   - **Features**:
     - Real-time communication log
     - Command history
     - Manual command input
     - Response monitoring
     - Error tracking

### 4. **Job Manager Tab** 📋
   - **File**: `src/ui/tabs/job_manager.rs`
   - **Features**:
     - Job queue management
     - Job creation and editing
     - Material selection
     - Job scheduling
     - Progress tracking
     - Resume/pause controls

### 5. **Feeds & Speeds Tab** ⚙️
   - **File**: `src/ui/tabs/feeds_speeds.rs`
   - **Features**:
     - Feed rate calculator
     - Spindle speed calculator
     - Material database integration
     - Tool selection
     - Cutting parameters

### 6. **Designer Tab** ✏️
   - **File**: `src/ui/tabs/designer.rs`
   - **Features**:
     - 2D shape drawing
     - Vector graphics import
     - Design workspace
     - Shape manipulation tools

---

## Left Panel Widgets (Machine Control)

### 1. **Connection Widget** 🔌
   - **File**: `src/widgets/connection.rs`
   - **Features**:
     - Controller type selection (Grbl, TinyG, etc.)
     - Port selection dropdown
     - Baud rate configuration
     - Connect/Disconnect buttons
     - Connection status

### 2. **Jog Widget** 🕹️
   - **File**: `src/widgets/jog.rs`
   - **Features**:
     - 3×3 directional control grid
     - X, Y, Z axis controls
     - Diagonal movement buttons
     - Step size adjustment
     - Quick multipliers (×10, ÷10, +, -)
     - Home button
     - Manual G-code command input
     - Rotary axes support (A, B, C, D)
     - Emergency stop button

### 3. **Overrides Widget** 🎚️
   - **File**: `src/widgets/overrides.rs`
   - **Features**:
     - Feed rate override slider
     - Spindle speed override slider
     - Rapid override controls
     - Real-time adjustment

---

## Right Panel Widgets (CAM Functions)

### 1. **Shape Generation Widget** ⬜
   - **File**: `src/designer/shape_generation.rs`
   - **Features**:
     - Rectangle generation
     - Circle/ellipse generation
     - Polygon generation
     - Dimension inputs
     - Position controls

### 2. **Toolpath Generation Widget** 🛤️
   - **File**: `src/designer/toolpath_generation.rs`
   - **Features**:
     - Toolpath pattern selection
     - Cut depth settings
     - Tool diameter input
     - Step over configuration
     - G-code generation

### 3. **Vector Import Widget** 📄
   - **File**: `src/designer/vector_import.rs`
   - **Features**:
     - SVG file import
     - DXF file import
     - Scale and position controls
     - Path conversion options

### 4. **Image Engraving Widget** 🖼️
   - **File**: `src/designer/image_engraving.rs`
   - **Features**:
     - Bitmap image import
     - Engraving parameter setup
     - Depth mapping
     - Resolution settings
     - Preview generation

### 5. **Tabbed Box Widget** 📦
   - **File**: `src/designer/tabbed_box.rs`
   - **Features**:
     - Box dimension inputs
     - Tab size configuration
     - Material thickness
     - Kerf compensation
     - Generate box panels

### 6. **Jigsaw Widget** 🧩
   - **File**: `src/designer/jigsaw.rs`
   - **Features**:
     - Jigsaw puzzle piece generation
     - Piece count configuration
     - Interlocking tab design
     - Custom patterns

---

## Additional Widgets (Not Currently Visible in Main UI)

### 7. **Machine Control Widget** 🎮
   - **File**: `src/widgets/machine_control.rs`
   - **Features**:
     - Comprehensive machine control
     - Multi-axis management

### 8. **Safety Widget** ⚠️
   - **File**: `src/widgets/safety.rs`
   - **Features**:
     - Safety limit configuration
     - Soft limits enable/disable
     - Hard limit monitoring
     - Emergency procedures

### 9. **Tool Management Widget** 🔧
   - **File**: `src/widgets/tool_management.rs`
   - **Features**:
     - Tool library management
     - Tool selection
     - Tool offset configuration
     - Tool change procedures

### 10. **Calibration Widget** 📏
   - **File**: `src/widgets/calibration.rs`
   - **Features**:
     - Axis calibration
     - Steps per mm configuration
     - Homing settings
     - Machine parameter tuning

### 11. **G-code Loading Widget** 📂
   - **File**: `src/widgets/gcode_loading.rs`
   - **Features**:
     - Quick file loading
     - Recent files list

### 12. **Job Scheduling Widget** 📅
   - **File**: `src/widgets/job_scheduling.rs`
   - **Features**:
     - Job queue interface
     - Scheduling controls

### 13. **CAM Operations Widget** 🏭
   - **File**: `src/widgets/cam_operations.rs`
   - **Features**:
     - CAM operation management
     - Operation sequencing

---

## Dialogs and Popups

### Material Database Dialog
   - **Location**: `src/layout/center_panel.rs` (show_central_panel)
   - **Features**:
     - Add new materials
     - Material properties editor
     - Physical properties (density, hardness)
     - Machining parameters
     - Tool recommendations
     - Notes field

### Find and Replace Dialog
   - **Location**: `src/gcodeedit/mod.rs` (G-code Editor)
   - **Features**:
     - Text search
     - Regular expression support
     - Replace functionality
     - Case sensitivity toggle
     - Whole word matching

### Keyboard Shortcuts Help
   - **Location**: `src/gcodeedit/mod.rs` (G-code Editor)
   - **Features**:
     - Complete shortcut reference
     - Navigation shortcuts
     - Editing shortcuts
     - Search shortcuts
     - Folding shortcuts

### Autocomplete Popup
   - **Location**: `src/gcodeedit/mod.rs` (G-code Editor)
   - **Features**:
     - G-code command suggestions
     - Context-aware completions
     - Modal state tracking

---

## Panel Visibility Toggles

Users can toggle visibility of:
- ✅ Left Panel (Machine Control)
- ✅ Right Panel (CAM Functions)
- ✅ Top Central Panel (Quick Actions)
- ✅ Bottom Status Bar

---

## Navigation Summary

```
Application Layout:
├── Top Menu Bar (always visible)
├── Top Central Panel (toggleable)
│   ├── File Operations
│   ├── G-code Controls
│   └── Status Display
├── Main Area
│   ├── Left Panel (toggleable)
│   │   ├── Connection Widget
│   │   ├── Jog Widget
│   │   └── Overrides Widget
│   ├── Center Panel (tabs)
│   │   ├── G-code Editor Tab ⭐ (default)
│   │   ├── 3D Visualizer Tab
│   │   ├── Device Console Tab
│   │   ├── Job Manager Tab
│   │   ├── Feeds & Speeds Tab
│   │   └── Designer Tab
│   └── Right Panel (toggleable)
│       ├── Shape Generation
│       ├── Toolpath Generation
│       ├── Vector Import
│       ├── Image Engraving
│       ├── Tabbed Box
│       └── Jigsaw
└── Bottom Status Bar (toggleable)
```

---

## Quick Reference: Panel Files

### Layout Panels
- `src/layout/top_menu.rs` - Top menu bar
- `src/layout/top_central_panel.rs` - Quick actions panel
- `src/layout/left_panel.rs` - Machine control panel
- `src/layout/right_panel.rs` - CAM functions panel
- `src/layout/center_panel.rs` - Main tabbed content
- `src/layout/bottom_status.rs` - Status bar

### Main Tabs
- `src/ui/tabs/gcode_editor.rs` - G-code editor
- `src/ui/tabs/visualizer_3d.rs` - 3D visualization
- `src/ui/tabs/device_console.rs` - Console/communication
- `src/ui/tabs/job_manager.rs` - Job management
- `src/ui/tabs/feeds_speeds.rs` - Feed/speed calculator
- `src/ui/tabs/designer.rs` - 2D designer

### Widgets
- `src/widgets/connection.rs` - Connection controls
- `src/widgets/jog.rs` - Jog controls
- `src/widgets/overrides.rs` - Override sliders
- `src/widgets/machine_control.rs` - Machine control
- `src/widgets/safety.rs` - Safety settings
- `src/widgets/tool_management.rs` - Tool management
- `src/widgets/calibration.rs` - Calibration

### CAM Widgets
- `src/designer/shape_generation.rs` - Shape tools
- `src/designer/toolpath_generation.rs` - Toolpath tools
- `src/designer/vector_import.rs` - Vector import
- `src/designer/image_engraving.rs` - Image engraving
- `src/designer/tabbed_box.rs` - Box generator
- `src/designer/jigsaw.rs` - Jigsaw generator

---

## Notes

- The **G-code Editor** is set as the default tab
- Some widgets exist but are not currently exposed in the main UI
- All panels can be toggled on/off via the View menu
- The application uses egui for the UI framework
