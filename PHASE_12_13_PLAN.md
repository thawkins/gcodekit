# Phase 12 & 13: Real-Time Machine Status Monitoring and Device Console Integration

## Overview

Phase 12 implements real-time machine status monitoring by periodically querying the device with the "?" command and parsing responses to display machine state. Phase 13 integrates device communication into a unified console with severity-based filtering and tracing capabilities.

---

## Phase 12: Real-Time Machine Status Monitoring

### Phase 12.1: Status Query and Response Parsing

**Objective**: Implement periodic status querying infrastructure.

**Status**: Foundation already exists - enhance and solidify

**Components**:
- `communication/status_monitor.rs`: Already implements periodic "?" querying with Tokio async
- `communication/status_parser.rs`: Parses "?<MachineStatus>" responses
- `communication/grbl_status.rs`: Defines MachineStatus struct

**Implementation Tasks**:
1. Verify status monitor configuration (250ms default interval)
2. Ensure parser handles all GRBL response formats correctly
3. Add error resilience for malformed responses
4. Ensure status queries do NOT appear in device console

**Key Points**:
- Status queries use "?" command - distinct from user commands
- Query responses are internal only, not logged to console
- History buffer maintains 300 samples (~75 seconds at 250ms intervals)
- Adaptive timing: faster queries during Run state, slower during Idle

---

### Phase 12.2: Status Update Integration

**Objective**: Integrate status updates into UI state display.

**Components**:
- `app/state.rs`: MachineState tracking
- `types.rs`: MachinePosition, MachineMode enums
- Status bar widgets: Display current position, state, speed

**Implementation Tasks**:
1. Create status update pipeline from StatusMonitor to AppState
2. Update MachinePosition tracking with latest status
3. Synchronize machine state (Idle/Run/Hold/Alarm/etc) in UI
4. Update status bar display with:
   - Current machine state
   - X/Y/Z position (MPos and WPos)
   - Feed rate and spindle speed
   - Connection status

**Key Points**:
- Status updates every 250ms from monitor
- UI renders status every frame (60 FPS)
- No console logging of status queries
- Real-time position overlay in visualizer

---

### Phase 12.3: Status Analytics and Trend Analysis

**Objective**: Implement status trend detection and history tracking.

**Components**:
- `communication/status_analytics.rs`: Trend analysis
- History buffer with circular storage
- State transition tracking

**Implementation Tasks**:
1. Detect state transitions (Idle → Run → Hold)
2. Track position changes for movement detection
3. Detect feed/spindle speed changes
4. Calculate feed rate changes (acceleration/deceleration)
5. Track error patterns for debugging

**Key Points**:
- Trends used for UI animations
- State consistency validation
- Historical data available for debugging

---

### Phase 12.4: Status Display in UI Components

**Objective**: Display status information throughout the UI.

**Components**:
- Status bar (bottom of window)
- Left panel connection widget
- 3D visualizer overlay
- Jog widget status display

**Implementation Tasks**:
1. Status bar updates:
   - Connection status (Connected/Disconnected)
   - Machine state with color coding (Green=Idle, Orange=Run, Red=Alarm)
   - Position display (X: 12.34 Y: 56.78 Z: -2.10)
   - Feed rate and spindle speed
   - GRBL version

2. Connection widget:
   - Status indicator with state
   - Current port name
   - Last error message (if any)

3. Jog widget header:
   - Current position display
   - Step size indicator
   - Feed rate setting

4. Visualizer overlay:
   - Real-time machine position (3D crosshair)
   - Current feed rate info
   - State indicator

**Key Points**:
- Color-coded state indicators
- Smooth position animation
- Responsive to status updates
- Clean, professional appearance

---

## Phase 13: Device Console Integration with Communications

### Phase 13.1: Console Message Architecture

**Objective**: Create unified message framework for all device communications.

**Components**:
- `communication/device_logger.rs`: Message capture and filtering
- `communication/device_logger_integration.rs`: Integration points
- Message types: Command, Response, Trace
- Severity levels: Error, Warning, Info, Debug

**Current Status**: Partially implemented
- ConsoleSeverity enum ✓
- MessageType enum ✓
- ConsoleMessage struct ✓
- Basic filtering framework ✓

**Implementation Tasks**:
1. Enhance message capture system:
   - Intercept all commands sent to device
   - Capture all device responses
   - Timestamp every message
   - Attach severity level

2. Filtering rules (do NOT show):
   - Status query commands ("?")
   - "ok" responses from device
   - Internal retry attempts

3. Filtering rules (ALWAYS show):
   - All device errors/alarms
   - All device responses except "ok"
   - All user commands (G-code, jogging, overrides)
   - Device feedback messages

4. Add tracing output logging:
   - Application-level warnings
   - Connection state changes
   - Error recovery attempts
   - Job state transitions

**Key Points**:
- Status queries hidden automatically
- "ok" responses hidden automatically
- Everything else shown by default
- Severity filtering per message type

---

### Phase 13.2: Console UI with Filtering Controls

**Objective**: Implement interactive device console with severity filtering.

**Components**:
- `ui/tabs/device_console.rs`: Console UI tab
- Severity filter checkboxes
- Message display area with colors
- Copy/Clear controls

**Current Status**: Partially implemented
- Basic console display exists
- Copy/Clear buttons work
- Filter checkboxes exist but incomplete

**Implementation Tasks**:
1. Severity filter implementation:
   - Checkbox for each: Debug, Info, Warning, Error
   - Filter state persists across tabs
   - Real-time filtering (no delay)
   - Shows count of filtered messages

2. Message display enhancements:
   - Color coding by severity:
     - Error: Red
     - Warning: Yellow
     - Info: White
     - Debug: Gray
   - Color coding by type:
     - Command: Green
     - Response: Blue
     - Trace: Gray
   - Timestamp display (HH:MM:SS.mmm)
   - Message type indicator (CMD/RES/TRC)

3. Console controls:
   - "📋 Copy All" - copy all visible messages
   - "🗑️ Clear" - clear all messages
   - "⏸️ Pause" - pause incoming messages
   - "▶️ Resume" - resume incoming messages
   - "💾 Save Log" - save console to file
   - Search/Filter box (optional Phase 2)

4. Message format in console:
   ```
   [HH:MM:SS.mmm] [CMD/RES/TRC] [ERROR/WARN/INFO/DEBUG] <message>
   ```

5. Integration with app state:
   - Add console_messages vector to MachineState
   - Add active_severities filter list
   - Add console_paused flag
   - Add message capacity limit (last 1000 messages)

**Key Points**:
- Auto-scrolls to bottom on new messages
- Severity filters independently toggle each level
- No performance impact from large message history
- Messages retain full information even when filtered

---

## Implementation Sequence

### Phase 12 Timeline
1. **12.1** - Status query validation (0.5 days)
2. **12.2** - Status UI integration (1 day)
3. **12.3** - Analytics & trend detection (1.5 days)
4. **12.4** - UI status display (1.5 days)

### Phase 13 Timeline
1. **13.1** - Console message architecture (1 day)
2. **13.2** - Console UI with filtering (1.5 days)

**Total Estimated**: 7 days

---

## Dependencies and Integration Points

### Phase 12 Dependencies
- Existing status monitor task (already running)
- App state management
- Status bar UI framework
- Visualizer 3D rendering

### Phase 13 Dependencies
- GRBL communication layer
- Tracing infrastructure
- egui UI framework
- Message storage system

### Cross-Phase Integration
- Both phases share message logging infrastructure
- Status queries bypass console logging
- Console displays user commands and device responses
- Tracing output goes to console

---

## Testing Strategy

### Phase 12 Tests
- [ ] Status monitor correctly queries at interval
- [ ] Status parser handles all GRBL response formats
- [ ] Position tracking updated correctly
- [ ] State transitions detected
- [ ] UI displays current status
- [ ] Status queries don't appear in console

### Phase 13 Tests
- [ ] Console captures all user commands
- [ ] Console captures all device responses
- [ ] Status queries hidden from console
- [ ] "ok" responses hidden from console
- [ ] Severity filters work independently
- [ ] Message format correct
- [ ] Message capacity enforced
- [ ] Copy/Clear functions work
- [ ] Filter state persists

---

## Success Criteria

### Phase 12 Success
- [ ] Real-time position display updates smoothly
- [ ] Machine state displayed accurately
- [ ] Status queries are transparent (not visible to user)
- [ ] No performance degradation
- [ ] Status bar shows all required information

### Phase 13 Success
- [ ] All user commands logged to console
- [ ] All device responses logged (except "ok")
- [ ] Status queries not shown in console
- [ ] Severity filters work correctly
- [ ] Console display is readable and professional
- [ ] Message history maintained correctly
- [ ] Performance unaffected by message logging

---

## Architecture Notes

### Status Query Pipeline
```
StatusMonitor Task (Tokio)
    ↓ (every 250ms)
GRBL Device (sends "?")
    ↓ (receives response)
StatusParser (parse "<Idle|MPos:0.00,0.00,0.00|WPos:0.00,0.00,0.00>")
    ↓
MachineStatus struct
    ↓
AppState.machine_status
    ↓
UI Components (status bar, visualizer, etc.)
```

### Console Message Pipeline
```
GrblCommunication::send_command()
    ↓ (logs command)
DeviceLogger::log_command()
    ↓
DeviceLogger (internal storage)
    ↓ (on update event)
UI Console Tab (reads filtered messages)
    ↓
Display with severity filtering
```

### Filtering Logic
```
Raw Message
    ↓
Is it a status query ("?") or "ok" response?
    ├─ YES → Store internally, don't log to console
    └─ NO → Log to console
        ↓
Apply severity filter
    ├─ Error level active? → Show errors
    ├─ Warning level active? → Show warnings
    ├─ Info level active? → Show info
    └─ Debug level active? → Show debug
        ↓
Display in console with formatting
```

---

## File Structure

### Phase 12 Files
- `src/communication/status_monitor.rs` - Enhance existing
- `src/communication/status_parser.rs` - Enhance existing
- `src/communication/grbl_status.rs` - Enhance existing
- `src/communication/status_analytics.rs` - Enhance existing
- `src/types.rs` - Update status types
- Status bar widget - New or enhance existing
- UI integration - Update multiple tabs

### Phase 13 Files
- `src/communication/device_logger.rs` - Enhance existing
- `src/communication/device_logger_integration.rs` - New integration points
- `src/ui/tabs/device_console.rs` - Enhance existing
- `src/app/state.rs` - Add console state fields

---

## Known Constraints

1. **Status Query Transparency**: Status queries must be completely transparent to user
2. **Performance**: Console with 1000+ messages must remain responsive
3. **GRBL Protocol**: Must respect GRBL v1.1+ status response format
4. **No User Configuration**: Initial version has fixed filtering rules
5. **Real-time Display**: Status updates must feel responsive (<100ms perceived latency)

---

## Future Enhancements (Not in Scope)

- [ ] Search/filter within console messages
- [ ] Export console to file with multiple formats
- [ ] Status history visualization
- [ ] Performance metrics overlay
- [ ] Replay mode for console playback
- [ ] User-configurable filtering rules
- [ ] Multi-session logging

