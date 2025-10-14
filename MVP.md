# Minimum Viable Product Test Procedure

## Prerequisites
- **Hardware**: CNC machine or laser engraver with GRBL v1.1+ firmware (or Smoothieware/TinyG/G2core/FluidNC compatible device)
- **Software**: gcodekit application built with `cargo build --release`
- **Test Files**: Use `assets/gcode/test_gcode.gcode` or create simple G-code (e.g., `G0 X0 Y0 Z0`, `G1 X10 Y10 F100`)
- **Serial Connection**: USB cable connecting computer to CNC controller

## Test Steps

1. **Build and Launch Application**
   - Run `cargo build --release` to compile optimized binary
   - Execute `./target/release/gcodekit` (or equivalent for your platform)
   - Verify application window opens with left/right panels and central tabs

2. **Connect to Hardware Device**
   - In left panel, click "Connection" widget
   - Select appropriate serial port (e.g., `/dev/ttyUSB0` on Linux, `COM3` on Windows)
   - Set baud rate (typically 115200 for GRBL)
   - Click "Connect" button
   - Verify status bar shows "Connected", firmware version, and machine position (X/Y/Z)

3. **Verify Machine Status**
   - Check status bar for "Idle" state and current position
   - Use jog widget to perform small movements (0.1mm steps) on X/Y axes
   - Confirm machine responds to jog commands and position updates in real-time

4. **Load and Send G-code**
   - In left panel, click "G-code Loading" widget
   - Browse and select `assets/gcode/test_gcode.gcode`
   - Click "Send" to queue the file
   - Monitor "Job Manager" tab for job progress and status

5. **Monitor Execution**
   - Switch to "3D Visualizer" tab to watch toolpath simulation
   - Observe real-time machine position overlay on visualization
   - Check status bar for execution progress and state changes
   - Use "Overrides" widget to adjust feed rate/spindle power during execution

6. **Test Interactive Controls**
   - Pause/resume job using Job Manager controls
   - Perform manual jog movements while job is paused
   - Test emergency stop functionality if available
   - Verify error recovery by simulating connection interruption (disconnect USB briefly)

7. **Complete Test and Disconnect**
   - Allow job to complete or manually stop it
   - Verify final machine position matches expected coordinates
   - Disconnect using Connection widget
   - Confirm status bar shows "Disconnected"

## Expected Results
- Successful connection establishment within 5 seconds
- Real-time position updates during movement
- G-code execution completes without errors
- Visualization accurately represents machine movement
- All interactive controls respond immediately
- Application remains stable throughout testing

## Troubleshooting
- If connection fails: Verify serial port permissions, correct baud rate, and device power
- If G-code doesn't execute: Check file format, ensure machine is homed, verify controller compatibility
- If visualization lags: Reduce G-code complexity or check system resources

## Success Criteria
- All connection, loading, and execution steps complete successfully
- No application crashes or hangs during testing
- Machine responds accurately to all commands
- Real-time feedback works for position, status, and progress