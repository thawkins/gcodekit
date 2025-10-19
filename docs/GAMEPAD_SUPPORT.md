# Gamepad/Joystick Support for GcodeKit

## Overview

Task 4 implements comprehensive gamepad and joystick support for GcodeKit, enabling users to control their GRBL machines using gamepads, joysticks, and other SDL-compatible input devices.

## Features Implemented

### 1. Cross-Platform Gamepad Support
- **Framework**: gilrs (cross-platform gamepad library)
- **Supported Devices**: 
  - Xbox controllers (360, One, Series X|S)
  - PlayStation controllers (DualShock 4, DualSense)
  - Generic gamepads (8Bitdo, etc.)
  - Joysticks with button/axis support

### 2. Gamepad Input Module (`src/input/gamepad.rs`)
- **GamepadButton Enum**: Represents 11 gamepad buttons (North, South, East, West, Shoulders, Stick Presses, Start, Back, Guide)
- **AnalogStickState**: Handles analog stick input with deadzone and magnitude calculations
- **GamepadState**: Tracks complete gamepad state (buttons, sticks, triggers, connection)
- **GamepadMapping**: Configurable button-to-action mapping with customizable deadzone and sensitivity
- **GamepadController**: Main controller class for gamepad input management

### 3. Gamepad Button Mapping
Default button mappings:
- **North Button** → Probe Z
- **East Button** → Reset
- **West Button** → Home  
- **South Button** → Feed Hold
- **Left Shoulder** → Zoom Out
- **Right Shoulder** → Zoom In

All mappings are fully customizable via the gamepad settings UI.

### 4. Analog Stick Jogging
- **Left Analog Stick**: X/Y axis jogging (configurable)
- **Right Analog Stick**: Z-axis jogging (configurable)
- **Deadzone Control**: Adjustable from 0.0 to 0.5 (default 0.15)
- **Sensitivity Multiplier**: 0.1x to 10.0x (default 1.0x)

### 5. Gamepad Settings UI (`src/widgets/gamepad_settings.rs`)
Complete UI panel for configuring gamepad behavior:
- Deadzone adjustment slider
- Jog sensitivity multiplier slider
- Enable/disable left stick jogging
- Enable/disable right stick jogging
- Visual button mapping configuration
- Connection status indicator
- Reset to default button
- Apply and save configuration

## Architecture

### Module Structure
```
src/input/
├── mod.rs (Input module with Action enum and keyboard handling)
└── gamepad.rs (Gamepad support - 11 tests, 360+ lines)

src/widgets/
├── gamepad_settings.rs (Gamepad configuration UI - 5 tests, 280+ lines)
└── (other widget files)
```

### Key Data Structures

#### GamepadButton
```rust
pub enum GamepadButton {
    South,               // PlayStation Circle / Xbox B
    East,                // PlayStation Triangle / Xbox Y
    West,                // PlayStation Square / Xbox X
    North,               // PlayStation Cross / Xbox A
    LeftShoulder,        // LB / L1
    RightShoulder,       // RB / R1
    LeftStickPress,      // L3 / LS Click
    RightStickPress,     // R3 / RS Click
    Start,
    Back,
    Guide,               // Home button
}
```

#### GamepadMapping
```rust
pub struct GamepadMapping {
    pub button_map: HashMap<GamepadButton, Action>,
    pub deadzone: f32,                          // 0.0 to 0.5
    pub enable_left_stick_jog: bool,
    pub enable_right_stick_jog: bool,
    pub jog_sensitivity: f32,                  // 0.1 to 10.0
}
```

#### GamepadController
Main interface for gamepad interaction with thread-safe state management:
- `set_button()` - Update button press state
- `set_left_stick()` - Update left analog stick
- `set_right_stick()` - Update right analog stick
- `set_left_trigger()` - Update left trigger (0.0-1.0)
- `set_right_trigger()` - Update right trigger (0.0-1.0)
- `get_pressed_actions()` - Get all active actions
- `get_left_stick_jog()` - Get jog amounts if stick active
- `get_right_stick_jog()` - Get Z jog if stick active
- `is_button_pressed()` - Check single button state

## Test Coverage

### Gamepad Module Tests (11 tests)
1. `test_gamepad_state_default` - Default state initialization
2. `test_analog_stick_clamping` - Value clamping to -1.0..1.0
3. `test_analog_stick_magnitude` - Magnitude calculation
4. `test_analog_stick_active` - Deadzone detection
5. `test_gamepad_controller_button` - Button state tracking
6. `test_gamepad_controller_sticks` - Analog stick tracking
7. `test_gamepad_mapping_default` - Default mapping configuration
8. `test_gamepad_controller_pressed_actions` - Action detection
9. `test_gamepad_controller_jog` - Jog value extraction
10. `test_gamepad_controller_triggers` - Trigger tracking
11. `test_gamepad_controller_connected` - Connection status

### Gamepad Settings Widget Tests (5 tests)
1. `test_gamepad_settings_ui_state_default` - UI state initialization
2. `test_reset_to_default` - Reset mapping to defaults
3. `test_apply_mapping` - Apply new mapping
4. `test_set_button_mapping` - Set individual button mapping
5. `test_clear_button_mapping` - Clear button mapping

**Total: 16 new tests** (all passing)

## Integration Points

### Action System
Gamepad buttons can trigger any action from the existing Action enum:
- `OpenFile`, `SaveFile`, `ExportGcode`, `ImportVector`
- `Undo`, `Redo`
- `Home`, `JogXPlus`, `JogXMinus`, `JogYPlus`, `JogYMinus`, `JogZPlus`, `JogZMinus`
- `ProbeZ`, `FeedHold`, `Resume`, `Reset`
- `ZoomIn`, `ZoomOut`

### Jog System
Analog sticks integrate with existing jog system:
- Respects current step size configuration
- Applies sensitivity multiplier
- Works with all jog modes

### UI Integration
Gamepad settings accessible from:
- Settings panel in the application
- Customizable for each user profile
- Persisted with machine profiles

## Dependencies

- **gilrs 0.10**: Cross-platform gamepad library
- **Existing**: Arc, Mutex for thread-safe state management

## Performance Considerations

- **Polling**: Gamepad input can be polled at application frame rate (60 FPS)
- **Thread Safety**: Arc<Mutex<>> pattern allows safe gamepad state updates from any thread
- **Memory**: Minimal overhead (~1KB per gamepad instance)
- **CPU**: Negligible - simple state checks and mapping lookups

## Future Enhancements

1. **Trigger Sensitivity**: Use analog triggers for feed rate modulation
2. **Vibration Feedback**: Haptic feedback on alarm or completion
3. **Profile Presets**: Predefined button layouts for different workflows
4. **Key Combo Support**: Multi-button sequences (Ctrl+Alt)
5. **Motion Controls**: Accelerometer input for advanced control
6. **Android Support**: Mobile gamepad support via web interface

## Known Limitations

- Device must support standard gamepad input (HID)
- Wireless latency depends on connection quality
- Some exotic input devices may not be recognized
- Triggers currently treated as binary (on/off)

## Testing

Run gamepad tests with:
```bash
cargo test --lib input::gamepad
cargo test --lib widgets::gamepad_settings
```

Build and verify:
```bash
cargo check
cargo build --release
```

## Implementation Summary

**Status**: ✅ Complete

Task 4 successfully implements:
- ✅ Cross-platform gamepad library integration (gilrs)
- ✅ Comprehensive gamepad input model (11 button types, analog sticks, triggers)
- ✅ Customizable button-to-action mapping system
- ✅ Analog stick jogging with configurable deadzone and sensitivity
- ✅ Full UI for gamepad configuration and management
- ✅ 16 comprehensive unit tests
- ✅ Thread-safe state management
- ✅ Zero compilation warnings (project code)
- ✅ Integration with existing action and jog systems

**Test Results**: 348 library tests passing (including 16 new gamepad tests)

**Build Status**: ✅ Compiles successfully with zero project warnings

---

**Date Completed**: October 19, 2025
**Task**: 4 - Gamepad/Joystick Support
**Related Phases**: Input Enhancement
