# Machine Calibration System

## Overview

The machine calibration system provides comprehensive machine calibration capabilities for GRBL-based CNC machines and laser engravers. It manages three key aspects of machine calibration: step calibration, backlash compensation, and homing configuration.

## Features

### 1. Step Calibration (steps/mm)

Calibrates the number of stepper motor steps required to move each axis by 1mm.

- **Default Value**: 100 steps/mm (typical for GRBL)
- **Range**: 0.1 to 1000 steps/mm
- **GRBL Settings**: $100 (X), $101 (Y), $102 (Z)

#### Calibration Formula

```
new_steps = current_steps × (commanded_distance / actual_distance)
```

**Example**:
- Current: 100 steps/mm
- Commanded: 10mm
- Actual: 9.5mm
- New value: 100 × (10 / 9.5) = 105.26 steps/mm

### 2. Backlash Compensation

Compensates for mechanical play in lead screws or belt drive systems.

- **Default Value**: 0mm (no compensation)
- **Range**: 0 to 10mm per axis
- **GRBL Settings**: $130 (X), $131 (Y), $132 (Z)

#### Backlash Detection

Backlash is measured by moving in both directions:

```
backlash = |forward_position - backward_position|
```

When a machine has backlash, it returns to a different position after moving backward from a forward position, causing a positioning error.

### 3. Homing Configuration

Configures automatic machine zeroing using limit switches.

- **Enable/Disable**: $22 (0 = disabled, 1 = enabled)
- **Direction Invert**: $23 (bitmask for axis directions)
- **Feed Rate**: $24 (slow approach after switch contact, mm/min)
- **Seek Rate**: $25 (fast approach to switch, mm/min)

#### Direction Invert Bitmask

```
Bit 0 (0x01): X-axis direction
Bit 1 (0x02): Y-axis direction  
Bit 2 (0x04): Z-axis direction
```

Example: To invert X and Y: 0x03 (0x01 | 0x02)

## Architecture

### Core Module Structure

```
calibration/
├── mod.rs                      # Main module with profiles and results
├── step_calibration.rs         # Step calibration (steps/mm)
├── backlash_compensation.rs    # Backlash compensation (mm)
├── homing_config.rs            # Homing configuration
└── calibration_procedures.rs   # Automated calibration procedures
```

### Key Data Structures

#### MachineCalibration

```rust
pub struct MachineCalibration {
    pub step_calibration: StepCalibration,
    pub backlash_compensation: BacklashCompensation,
    pub homing_config: HomingConfiguration,
    pub calibration_history: Vec<CalibrationResult>,
    pub last_calibrated: Option<SystemTime>,
    pub machine_name: String,
    pub notes: String,
}
```

#### CalibrationProfiles

Multi-profile support for different machines:

```rust
pub struct CalibrationProfiles {
    pub profiles: HashMap<String, MachineCalibration>,
    pub active_profile: String,
}
```

#### CalibrationResult

Tracks each calibration operation:

```rust
pub struct CalibrationResult {
    pub axis: Axis,
    pub parameter: String,
    pub old_value: f32,
    pub new_value: f32,
    pub timestamp: SystemTime,
    pub success: bool,
    pub notes: String,
}
```

## Calibration Procedures

The system includes automated step-by-step procedures for guided calibration:

### 1. Step Calibration Procedure

7 steps for calibrating steps/mm:
1. Home all axes
2. Measure accurate reference distance (100mm typical)
3. Command X axis movement
4. Measure actual position
5. Calculate new step value
6. Apply calibration to GRBL
7. Repeat for Y and Z axes

**Estimated Duration**: 30 minutes

### 2. Backlash Measurement Procedure

8 steps for measuring and recording backlash:
1. Home all axes
2. Move to test position
3. Record forward position
4. Move backward 10mm
5. Move forward 10mm again
6. Record backward position
7. Calculate backlash
8. Repeat for Y and Z axes

**Estimated Duration**: 20 minutes

### 3. Homing Setup Procedure

7 steps for configuring homing:
1. Inspect limit switches
2. Enable homing cycle ($22=1)
3. Set homing seek rate (500 mm/min typical)
4. Set homing feed rate (25 mm/min typical)
5. Set homing direction invert if needed ($23)
6. Test homing cycle
7. Verify consistent homing position

**Estimated Duration**: 15 minutes

### 4. Full Calibration Procedure

Combines all three procedures:
- All 22 steps (7 + 8 + 7)
- **Estimated Duration**: 65 minutes

## Usage Examples

### Basic Usage

```rust
use gcodekit::calibration::{MachineCalibration, StepCalibration};

// Create calibration
let mut cal = MachineCalibration::new("My CNC".to_string());

// Set step values
cal.step_calibration.set_x_steps(105.26)?;
cal.step_calibration.set_y_steps(102.15)?;
cal.step_calibration.set_z_steps(110.00)?;

// Get GRBL commands to apply
let commands = cal.get_grbl_commands();
// Returns: ["$100=105.260", "$101=102.150", "$102=110.000", ...]
```

### Working with Profiles

```rust
use gcodekit::calibration::CalibrationProfiles;

let mut profiles = CalibrationProfiles::default();

// Create new profile
profiles.create_profile("Laser Cutter".to_string())?;

// Switch to profile
profiles.set_active_profile("Laser Cutter".to_string())?;

// Modify active profile
if let Some(cal) = profiles.get_active_mut() {
    cal.step_calibration.set_x_steps(95.5)?;
}

// Save/load profiles
profiles.save_to_file(Path::new("calibration.json"))?;
let loaded = CalibrationProfiles::load_from_file(Path::new("calibration.json"))?;
```

### Using Calibration Procedures

```rust
use gcodekit::calibration::CalibrationProcedure;

// Get step calibration procedure
let proc = CalibrationProcedure::step_calibration();

// Get current step
if let Some(step) = proc.get_current_step(1) {
    println!("Step {}: {}", step.step_number, step.description);
    if let Some(cmd) = &step.command {
        println!("Send: {}", cmd);
    }
    if step.user_action_required {
        println!("User action required!");
    }
}

// Navigate procedure
let next_step = proc.get_next_step(1);
```

### Calculate Step Correction

```rust
use gcodekit::calibration::StepCalibration;

// Machine moved 9.5mm when told to move 10mm with 100 steps/mm
let corrected = StepCalibration::calculate_correction(100.0, 10.0, 9.5);
// Result: ~105.26 steps/mm
```

### Measure Backlash

```rust
use gcodekit::calibration::BacklashCompensation;

// Forward to 10mm, backward from 10mm settles at 9.9mm
let backlash = BacklashCompensation::detect_backlash(10.0, 9.9, 10.0);
// Result: 0.1mm
```

## GRBL Commands Reference

### Step Calibration ($100-$102)

```grbl
$100=105.26    ; X steps/mm
$101=102.15    ; Y steps/mm
$102=110.00    ; Z steps/mm
```

### Backlash Compensation ($130-$132)

```grbl
$130=0.10      ; X backlash (mm)
$131=0.15      ; Y backlash (mm)
$132=0.05      ; Z backlash (mm)
```

### Homing Configuration ($22-$25)

```grbl
$22=1          ; Enable homing
$23=0          ; Direction invert mask (0=none, 1=X, 2=Y, 4=Z)
$24=25.0       ; Homing feed rate (mm/min)
$25=500.0      ; Homing seek rate (mm/min)
```

## Calibration History & Persistence

### Recording Calibrations

```rust
let result = CalibrationResult {
    axis: Axis::X,
    parameter: "$100".to_string(),
    old_value: 100.0,
    new_value: 105.26,
    timestamp: SystemTime::now(),
    success: true,
    notes: "Step calibration measurement".to_string(),
};

cal.record_calibration(result);
```

### History Queries

```rust
// Get latest calibration for axis
let latest = cal.get_latest_calibration(Axis::X, "$100");

// Get all calibrations for axis
let history = cal.get_axis_calibrations(Axis::X);

// Check if calibration is stale
use std::time::Duration;
let is_stale = cal.is_stale(Duration::from_days(30));

// Clear history (keeps current values)
cal.clear_history();
```

### File I/O

```rust
use std::path::Path;

// Save single profile
cal.save_to_file(Path::new("calibration.json"))?;

// Load single profile
let loaded = MachineCalibration::load_from_file(Path::new("calibration.json"))?;

// Save all profiles
profiles.save_to_file(Path::new("profiles.json"))?;

// Load all profiles
let profiles = CalibrationProfiles::load_from_file(Path::new("profiles.json"))?;
```

## Testing

The calibration system includes 58 comprehensive tests covering:

- Default value initialization
- Value validation and constraints
- GRBL command generation
- Calibration correction calculations
- Backlash detection formulas
- Homing configuration bit manipulation
- Profile management (create, delete, switch)
- File I/O and persistence
- Calibration history tracking
- Calibration procedures

Run tests:
```bash
cargo test --lib calibration
```

## Validation Rules

### Step Calibration

- **Valid Range**: 0.1 to 1000 steps/mm
- **Typical Range**: 50 to 200 steps/mm

### Backlash Compensation

- **Valid Range**: 0 to 10mm per axis
- **Typical Range**: 0.05 to 0.5mm

### Homing Feed/Seek Rates

- **Valid Range**: 0.1 to 5000 mm/min
- **Seek Rate** (fast): 300 to 1000 mm/min typical
- **Feed Rate** (slow): 10 to 50 mm/min typical

## Best Practices

1. **Frequent Calibration**: Check calibration every 50 hours of operation or when accuracy degrades

2. **Environmental Factors**: Recalibrate after temperature changes, especially for machines with significant thermal expansion

3. **Backlash Trending**: Track backlash over time; increasing backlash indicates wear in lead screws or belts

4. **Multiple Measurements**: Take 3-5 measurements for each axis and average the results

5. **Profile Management**: Keep separate profiles for different materials or machine configurations

6. **Documentation**: Use the notes field to record why calibration was performed and any special conditions

## Integration with UI

The calibration system integrates with the existing calibration widget in `src/widgets/calibration.rs`, providing:

- Real-time value adjustment with DragValue widgets
- GRBL command sending on value changes
- Three collapsing sections: Step Calibration, Backlash Compensation, Homing Configuration

## Future Enhancements

- Automated step calibration using computer vision
- Backlash trend analysis and wear prediction
- Temperature compensation for thermal expansion
- Automatic machine profile detection
- Web UI for remote calibration management
- Machine-specific calibration presets library

## References

- [GRBL Documentation](https://github.com/grbl/grbl/wiki)
- [CNC Step Calibration Guide](https://github.com/grbl/grbl/wiki/Calibrating-your-machine)
- [Backlash Compensation Theory](https://www.cnccookbook.com/backlash-cnc-machines/)
