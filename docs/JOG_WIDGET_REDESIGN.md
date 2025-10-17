# Jog Widget Redesign

## Overview
Redesigned the jog control widget to match the layout shown in the reference screenshot, providing a more intuitive and compact interface for machine control.

## Changes Made

### 1. Layout Redesign (`src/widgets/jog.rs`)

The jog widget has been completely redesigned with the following layout:

```
┌─────────────────────────────────────────────────────┐
│                    Control                          │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Step     Z           Y         Multipliers        │
│  [0.5][▼]         [Y+]          [×10][+]          │
│            [↗] [Y+] [↖]                            │
│     X     [X−] [🏠] [X+]        [1.0][▼]          │
│            [↘] [Y−] [↙]                            │
│            [Z−]                                     │
│                                  [÷10][-]          │
│                                                     │
│  Command: [_____________________________]          │
│                                                     │
│  [Rotary Axes] (collapsible)                       │
│  [🚨 EMERGENCY STOP]                               │
└─────────────────────────────────────────────────────┘
```

### Button Icons:
- **Axis buttons**: Display axis labels (X+, X−, Y+, Y−, Z+, Z−)
- **Diagonal buttons**: Use directional arrows (↗, ↖, ↘, ↙)
- **Home button**: Uses house emoji (🏠)
- **Emergency stop**: Uses siren emoji (🚨)

### 2. Main Features

#### Central 3×3 Control Grid:
- **Z Controls**: Labeled buttons (Z+/Z−) for Z-axis movement
- **Y Controls**: Labeled buttons (Y+/Y−) for Y-axis movement  
- **X Controls**: Labeled buttons (X+/X−) for X-axis movement
- **Diagonal Buttons**: Directional arrows (↗/↖/↘/↙) for combined Y+Z movements
- **Home Button**: House icon (🏠) for homing all axes

#### Left Side Controls:
- Step size numeric input with dropdown
- X-axis label for visual reference

#### Right Side Controls:
- **×10 button**: Multiply step size by 10
- **+ button**: Increment step size by 0.1
- Step size display with dropdown
- **÷10 button**: Divide step size by 10
- **- button**: Decrement step size by 0.1

#### Bottom Section:
- **Command input**: Manual G-code command entry (press Enter to send)
- **Rotary Axes**: Collapsible section for A, B, C, D axes (when available)
- **Emergency Stop**: Full-width red button for immediate stop

### 3. New State Field (`src/app/state.rs`)

Added `manual_command: String` field to `MachineState`:
- Stores the current manual command being typed
- Cleared after sending the command
- Accessible via the Command input field

### 4. Button Functionality

All buttons use clear, relevant icons and descriptive hover tooltips:
- **Z+/Z−**: Axis-labeled buttons for vertical movement
- **Y+/Y−**: Axis-labeled buttons for forward/back movement
- **X+/X−**: Axis-labeled buttons for left/right movement
- **Diagonal arrows (↗/↖/↘/↙)**: Combined Y+Z movements
- **Home (🏠)**: Home all axes
- **Step multipliers**: Mathematical operators (×10, ÷10, +, −)

### 5. Improved User Experience

- **Compact Layout**: All controls visible and accessible without scrolling
- **Visual Hierarchy**: Clear separation between main controls and auxiliary functions
- **Intuitive Symbols**: Directional arrows and mathematical operators
- **Quick Step Adjustment**: Multiple ways to change step size quickly
- **Manual Commands**: Direct G-code input for advanced users
- **Safety**: Emergency stop prominently displayed

## Technical Details

### Diagonal Movement Implementation
Diagonal buttons execute two jog commands in sequence:
```rust
// Example: Y+ Z+ diagonal
app.machine.communication.jog_axis('Y', app.machine.jog_step_size);
app.machine.communication.jog_axis('Z', app.machine.jog_step_size);
```

### Step Size Constraints
- Minimum: 0.1 mm
- Maximum: 100.0 mm
- Increment/Decrement: 0.1 mm
- Multiplier: ×10 / ÷10

### Manual Command Input
- Press Enter to send command
- Command is sent via `communication.send_gcode_line()`
- Input is cleared after successful send
- Errors are displayed in status message

## Testing
- ✅ All tests pass (194 tests)
- ✅ Build successful with no warnings (except deprecated external package)
- ✅ Widget compiles and renders correctly

## Future Enhancements
Potential improvements:
1. Keyboard shortcuts for jog buttons (arrow keys, Page Up/Down)
2. Visual feedback for active jog direction
3. Continuous jogging while button is held
4. Gamepad/joystick support
5. Customizable button layout
6. Step size presets save/load
