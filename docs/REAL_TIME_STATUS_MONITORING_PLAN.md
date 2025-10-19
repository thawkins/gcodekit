# Real-Time Machine Status Monitoring Implementation Plan

## Overview

This document outlines the comprehensive plan for implementing real-time machine status monitoring in gcodekit. The system will use GRBL's "?" status query command to periodically interrogate the device and decode responses to display live machine status information.

**Status**: Plan Document  
**Priority**: High  
**Estimated Effort**: 3-4 development phases  
**Target Release**: Phase 12  

---

## 1. Architecture Overview

### 1.1 High-Level Design

The real-time status monitoring system consists of five primary components:

```
┌─────────────────────────────────────────────────────────────┐
│                    Main Application                         │
├─────────────────────────────────────────────────────────────┤
│  Status Monitor Thread (Tokio Task)                         │
│  ├─ Periodic "?" Query (configurable interval, 200-500ms)   │
│  ├─ Response Parsing & Decoding                             │
│  └─ Status State Management                                 │
├─────────────────────────────────────────────────────────────┤
│  Status Data Structures                                     │
│  ├─ MachineStatus (comprehensive status snapshot)           │
│  ├─ Position (XYZ coordinates)                              │
│  ├─ Feedback Counters (lines processed, lines remaining)    │
│  └─ Override States (feed rate, spindle speed, laser power)  │
├─────────────────────────────────────────────────────────────┤
│  Status History & Trends                                    │
│  ├─ Circular buffer for status history (100+ samples)       │
│  ├─ Performance metrics (feedrate average, state transitions)│
│  └─ Error/alarm tracking                                    │
├─────────────────────────────────────────────────────────────┤
│  UI Components                                              │
│  ├─ Live Status Display Widget (updates 5+ times/sec)       │
│  ├─ Status History Visualizer (graph/chart)                 │
│  ├─ Machine State Indicator (animated)                      │
│  └─ Performance Metrics Panel                               │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 GRBL Status Query Format

GRBL responds to the "?" query with a status message in this format:

```
<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100|A:SFM>
```

Breaking down the response:
- **State**: Current machine state (Idle, Run, Hold, Jog, Alarm, Door, Check, Home, Sleep)
- **MPos**: Machine position (X,Y,Z in absolute coordinates)
- **WPos**: Work position (X,Y,Z in work coordinates) - only in some responses
- **FS**: Feed rate (current), Spindle speed (RPM)
- **Ov**: Override percentages (feed rate %, spindle speed %, laser power %)
- **Pn**: Pin states (e.g., X:probe, Y:limit, Z:door, A:cycle_start)
- **Buf**: Planner buffer fill count
- **Rx**: Serial RX buffer fill count
- **Line**: Line number being executed (GRBL 1.1+)

Example responses:
```
<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100|A:SFM>
<Run|MPos:10.50,5.25,2.10|FS:1500,12000|Ov:100,100,100|Buf:18|Rx:256|Line:42>
<Hold|MPos:12.30,6.15,1.85|FS:0,12000|Ov:120,100,50>
<Alarm|Pn:XYZ>
```

---

## 2. Data Structure Definitions

### 2.1 New Status Types Module

**File**: `src/communication/grbl_status.rs`

Comprehensive status data structures supporting GRBL v1.0, v1.1, and v1.2 specifications.

```rust
/// Real-time machine status snapshot
#[derive(Debug, Clone, Default)]
pub struct MachineStatus {
    /// Current machine state
    pub state: MachineState,
    
    /// Machine position (MPos) - absolute coordinates
    pub machine_position: Position,
    
    /// Work position (WPos) - relative to work coordinate system
    pub work_position: Option<Position>,
    
    /// Feed rate and spindle speed
    pub feed_speed: FeedSpeed,
    
    /// Override values (feed, spindle, laser/coolant)
    pub overrides: OverrideState,
    
    /// Current line number being executed (GRBL 1.1+)
    pub line_number: Option<u32>,
    
    /// Planner buffer status
    pub buffer_state: BufferState,
    
    /// Input pin states (probe, limit switches, etc.)
    pub pin_states: PinStates,
    
    /// Feedback counters and rates
    pub feedback: FeedbackMetrics,
    
    /// Timestamp of when this status was captured
    pub timestamp: std::time::Instant,
}

/// Position with optional rotary axes
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: Option<f32>,
    pub b: Option<f32>,
    pub c: Option<f32>,
}

/// Feed rate and spindle/laser speed
#[derive(Debug, Clone, Default)]
pub struct FeedSpeed {
    /// Current feed rate (mm/min)
    pub feed_rate: f32,
    /// Current spindle speed (RPM) or laser power (0-100%)
    pub spindle_speed: f32,
}

/// Override percentages (100% = normal speed)
#[derive(Debug, Clone, Default)]
pub struct OverrideState {
    /// Feed rate override (100% = 100 mm/min)
    pub feed_override: u8,
    /// Spindle speed override
    pub spindle_override: u8,
    /// Laser power or coolant override
    pub coolant_override: u8,
}

/// Buffer status
#[derive(Debug, Clone, Default)]
pub struct BufferState {
    /// Planner buffer fill count (0-128 typical)
    pub planner_buffer: u8,
    /// Serial RX buffer fill count
    pub rx_buffer: u8,
}

/// Input pin states
#[derive(Debug, Clone, Default)]
pub struct PinStates {
    pub probe: bool,
    pub x_limit: bool,
    pub y_limit: bool,
    pub z_limit: bool,
    pub door_open: bool,
    pub cycle_start: bool,
    pub feed_hold: bool,
}

/// Feedback metrics
#[derive(Debug, Clone, Default)]
pub struct FeedbackMetrics {
    /// Lines queued (planned)
    pub lines_queued: u32,
    /// Lines remaining to execute
    pub lines_remaining: u32,
    /// Lines completed
    pub lines_completed: u32,
}

/// Status parser result
#[derive(Debug, Clone)]
pub struct ParsedStatus {
    pub status: MachineStatus,
    pub raw_response: String,
    pub parse_success: bool,
    pub parse_errors: Vec<String>,
}
```

---

## 3. Core Components

### 3.1 Status Parser Module

**File**: `src/communication/status_parser.rs`

Robust parser for decoding GRBL status responses with support for multiple GRBL versions.

**Key Functions**:
- `parse_status_response(raw: &str) -> Result<MachineStatus, StatusParseError>` - Parse status string
- `parse_machine_state(state_str: &str) -> MachineState` - Extract machine state
- `parse_position(pos_str: &str) -> Result<Position, StatusParseError>` - Parse XYZ position
- `parse_feed_speed(fs_str: &str) -> Result<FeedSpeed, StatusParseError>` - Parse F/S values
- `parse_overrides(ov_str: &str) -> Result<OverrideState, StatusParseError>` - Parse override %
- `parse_pin_states(pn_str: &str) -> Result<PinStates, StatusParseError>` - Parse pin state
- `parse_buffer_state(buf_rx: &str) -> Result<BufferState, StatusParseError>` - Parse buffer info
- `validate_status(status: &MachineStatus) -> Result<(), Vec<StatusValidationError>>` - Validate

**Features**:
- Fault-tolerant parsing (partial data accepted)
- GRBL version detection
- Detailed error reporting with context
- Cache of last valid status for fallback

### 3.2 Status Monitor Task

**File**: `src/communication/status_monitor.rs`

Async task that periodically queries device status and manages state updates.

**Key Components**:
```rust
/// Status monitoring configuration
#[derive(Debug, Clone)]
pub struct StatusMonitorConfig {
    /// Query interval in milliseconds (default: 250ms)
    pub query_interval_ms: u64,
    
    /// Maximum retries on parse failure
    pub max_parse_retries: u32,
    
    /// Enable adaptive query timing (faster during Run, slower during Idle)
    pub adaptive_timing: bool,
    
    /// History buffer size (default: 300 samples = 1.5min @ 200ms)
    pub history_buffer_size: usize,
    
    /// Enable circular buffer (discard oldest on full)
    pub circular_buffer: bool,
    
    /// Alarm/error tracking enabled
    pub track_errors: bool,
    
    /// Max error patterns to track
    pub max_error_patterns: usize,
}

/// Running status monitor instance
pub struct StatusMonitor {
    config: StatusMonitorConfig,
    
    /// Current status snapshot
    current_status: Arc<Mutex<MachineStatus>>,
    
    /// Status history
    status_history: Arc<Mutex<VecDeque<MachineStatus>>>,
    
    /// Cancellation token
    cancel_token: tokio_util::sync::CancellationToken,
    
    /// Monitor task handle
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl StatusMonitor {
    pub fn new(config: StatusMonitorConfig) -> Self { /* ... */ }
    pub async fn start(&mut self, comm: Arc<Mutex<GrblCommunication>>) { /* ... */ }
    pub async fn stop(&mut self) { /* ... */ }
    pub fn get_current_status(&self) -> MachineStatus { /* ... */ }
    pub fn get_status_history(&self, count: usize) -> Vec<MachineStatus> { /* ... */ }
    pub fn get_average_feedrate(&self, samples: usize) -> f32 { /* ... */ }
}
```

**Algorithm**:
1. Wait for configured interval (e.g., 250ms)
2. Send "?" command to device
3. Receive and parse response
4. Update current status and history
5. Detect state transitions and alarms
6. Calculate trends and metrics
7. Repeat until cancelled

### 3.3 Status History & Analytics

**File**: `src/communication/status_analytics.rs`

Analyze status trends and provide performance metrics.

**Key Metrics**:
- Average feedrate over window
- Buffer fullness trends
- State transition frequency
- Alarm/error frequency
- Position accuracy
- Velocity profiles
- Estimated time remaining

```rust
#[derive(Debug, Clone)]
pub struct StatusAnalytics {
    /// Average feedrate over last N samples
    pub avg_feedrate: f32,
    
    /// Peak feedrate reached
    pub peak_feedrate: f32,
    
    /// Average buffer fill %
    pub avg_buffer_fill: f32,
    
    /// State transitions in history
    pub state_transitions: Vec<StateTransition>,
    
    /// Error frequency (errors per minute)
    pub error_frequency: f32,
    
    /// Estimated job completion time
    pub estimated_completion: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: MachineState,
    pub to: MachineState,
    pub timestamp: Instant,
    pub duration_in_state: Duration,
}
```

---

## 4. Integration with GrblCommunication

### 4.1 Extending GrblCommunication

**Location**: `src/communication/grbl.rs`

Add to existing `GrblCommunication` struct:

```rust
pub struct GrblCommunication {
    // ... existing fields ...
    
    /// Status monitor instance
    pub status_monitor: Option<StatusMonitor>,
    
    /// Latest machine status
    pub latest_status: MachineStatus,
    
    /// Status change callbacks
    pub status_callbacks: Vec<StatusChangeCallback>,
    
    /// Error tracking
    pub error_history: VecDeque<(Instant, String)>,
}

impl GrblCommunication {
    /// Start real-time status monitoring
    pub async fn start_status_monitoring(&mut self, config: StatusMonitorConfig) -> Result<()>;
    
    /// Stop status monitoring
    pub async fn stop_status_monitoring(&mut self) -> Result<()>;
    
    /// Get current status
    pub fn get_current_status(&self) -> MachineStatus;
    
    /// Get status history
    pub fn get_status_history(&self, samples: usize) -> Vec<MachineStatus>;
    
    /// Get status analytics
    pub fn get_analytics(&self) -> StatusAnalytics;
    
    /// Register callback for status changes
    pub fn on_status_change(&mut self, callback: StatusChangeCallback);
}
```

### 4.2 Connection Flow

Updated connection sequence:

```
User initiates connection
        ↓
Connect to serial port
        ↓
Query version (!)
        ↓
Initialize GrblCommunication
        ↓
START status monitoring (async task)
        ↓
Set connection_state = Connected
        ↓
Begin querying status every 250ms
        ↓
Update UI with live status
```

---

## 5. UI Components

### 5.1 Status Display Widget

**File**: `src/widgets/status_display.rs`

Real-time status display with color-coded states.

**Features**:
- Machine state indicator (animated icons for Run, Hold, Alarm)
- Live XYZ position with smoothing
- Feed rate and spindle speed
- Override percentages with adjustment controls
- Input pin state indicators (limits, probe, door)
- Buffer fill visualization
- Update frequency: 5+ times per second

**Visual Design**:
```
┌─ MACHINE STATUS ─────────────────────────────┐
│ State: ● RUNNING (animated)                  │
│ Position: X: 123.45mm  Y: 67.89mm  Z: 10.00│
│ Feed: 1500 mm/min  Spindle: 12000 RPM       │
│ Overrides: Feed 100% | Spindle 100%         │
│ Buffer: ████████░░ (80%)                     │
│ Pins: Probe □  X-Limit □  Y-Limit □  Z-Limit │
└──────────────────────────────────────────────┘
```

### 5.2 Status History Visualizer

**File**: `src/widgets/status_history.rs`

Time-series visualization of status data.

**Graphs**:
- Position vs time (XYZ traces)
- Feed rate vs time
- Buffer fill vs time
- State timeline (gantt-style)
- Machine state histogram

### 5.3 Machine State Indicator

**File**: `src/widgets/state_indicator.rs`

Large animated indicator showing current machine state with status transitions.

**States with visual indicators**:
- 🟢 Idle (green, static)
- 🔵 Run (blue, pulsing animation)
- 🟡 Hold (yellow, slow pulse)
- 🔴 Alarm (red, blinking)
- 🟠 Door (orange, alert)
- ⚪ Unknown (gray)

### 5.4 Integration into Layout

Update `src/layout/bottom_status.rs`:
- Expand to show real-time status data
- Add state indicator icon with animation
- Show buffer fill status
- Display override percentages
- Show pin states (limits, probe, door)

Update left panel widgets:
- Add status monitor control widget
- Query interval adjustment
- Status history access

---

## 6. Configuration Management

### 6.1 Status Monitor Configuration

**File**: `src/config/status_monitor_config.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMonitorConfig {
    pub enabled: bool,
    pub query_interval_ms: u64,
    pub adaptive_timing: bool,
    pub history_buffer_size: usize,
    pub auto_start_on_connect: bool,
}

impl Default for StatusMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            query_interval_ms: 250,      // 4 updates/second
            adaptive_timing: true,        // Faster during Run, slower during Idle
            history_buffer_size: 300,     // ~75 seconds at 250ms interval
            auto_start_on_connect: true,  // Begin monitoring immediately
        }
    }
}
```

**User Preferences**:
- Load/save configuration to `~/.config/gcodekit/status_monitor.json`
- Add UI controls in preferences dialog
- Presets: Responsive (100ms), Balanced (250ms), Power-saving (500ms)

---

## 7. Implementation Phases

### Phase 12.1: Core Status Infrastructure (Week 1)

**Deliverables**:
1. Status data structures (`src/communication/grbl_status.rs`)
2. Status parser (`src/communication/status_parser.rs`)
3. Unit tests for parser with sample GRBL responses
4. Status validation logic
5. Error types for status operations

**Success Criteria**:
- ✅ Parser handles all GRBL v1.0/1.1/1.2 response formats
- ✅ 100+ test cases passing (edge cases, malformed data)
- ✅ Zero clippy warnings
- ✅ Documentation complete

### Phase 12.2: Status Monitor Task & Integration (Week 2)

**Deliverables**:
1. Async status monitor task (`src/communication/status_monitor.rs`)
2. Status history management with circular buffer
3. Analytics calculations (`src/communication/status_analytics.rs`)
4. Integration with `GrblCommunication`
5. Tokio task lifecycle management (start/stop/cleanup)

**Success Criteria**:
- ✅ Monitor queries device every 250ms
- ✅ History buffer maintains last 300 samples
- ✅ No memory leaks or unbounded growth
- ✅ Graceful shutdown on disconnect
- ✅ 50+ test cases passing

### Phase 12.3: UI Components & Display (Week 3)

**Deliverables**:
1. Status display widget (`src/widgets/status_display.rs`)
2. Real-time position display with smoothing
3. Machine state indicator with animations
4. Status history visualizer with graphs
5. Pin state indicators
6. Integration into bottom status bar and left panel

**Success Criteria**:
- ✅ Status updates visible 5+ times per second
- ✅ Position display smooth (no jitter)
- ✅ Animations perform at 60 FPS
- ✅ Charts render responsively
- ✅ No UI lag during monitor queries

### Phase 12.4: Advanced Features & Optimization (Week 4)

**Deliverables**:
1. Adaptive query timing (faster during Run, slower during Idle)
2. Status-based UI callbacks (e.g., highlight on Alarm)
3. Performance metrics dashboard
4. State transition tracking and alerts
5. Error frequency monitoring
6. Configuration persistence

**Success Criteria**:
- ✅ Adaptive timing reduces power consumption 20%+
- ✅ Alarm states trigger UI alerts immediately
- ✅ Performance dashboard accurate
- ✅ Configuration saves/loads correctly
- ✅ All tests passing (300+)

---

## 8. Testing Strategy

### 8.1 Unit Tests

**Parser Tests** (`tests/communication/status_parser.rs`):
- Parse valid GRBL responses for v1.0, v1.1, v1.2
- Handle malformed input gracefully
- Test edge cases (missing fields, extra whitespace, etc.)
- Test coordinate limits and overflow scenarios
- Test override percentage boundaries (0-200%)

**Monitor Tests** (`tests/communication/status_monitor.rs`):
- Verify periodic query interval
- Test history buffer circular behavior
- Test pause/resume of monitoring
- Test rapid connect/disconnect
- Verify memory stability over long runs (1000+ queries)

**Analytics Tests** (`tests/communication/status_analytics.rs`):
- Verify feedrate calculations
- Test state transition detection
- Verify buffer fill trends
- Test moving averages

### 8.2 Integration Tests

**Device Communication Tests**:
- Connect to real device, verify status queries work
- Test with simulated device responses
- Test recovery from malformed responses
- Test high-frequency query rates (stress test)

**UI Tests**:
- Verify real-time updates in UI
- Test status widget rendering
- Verify animations don't block UI thread
- Test responsiveness during heavy status queries

### 8.3 Performance Tests

- Query latency measurements
- UI frame rate during status updates
- Memory usage profiling (steady state)
- CPU usage profiling

---

## 9. Error Handling & Recovery

### 9.1 Parser Errors

Strategy: Graceful degradation
- Missing optional fields → use defaults
- Malformed coordinates → use last valid position
- Invalid machine state → log warning, assume "Unknown"
- Parse failures → use cached last status, retry next cycle

### 9.2 Communication Errors

Strategy: Automatic retry with backoff
- Timeout on status query → retry after 500ms
- Device disconnection → stop monitor gracefully
- Buffer overflow → reduce query frequency adaptively
- Repeated errors → log analytics, possible alert to user

### 9.3 UI Thread Safety

- Status data behind `Arc<Mutex<>>` for thread-safe access
- Update callbacks run on main UI thread
- Avoid blocking queries in UI thread
- Use async channels for status updates

---

## 10. Performance Targets

### 10.1 Query Performance

| Metric | Target | Notes |
|--------|--------|-------|
| Query latency | <50ms | Time from "?" send to response parse |
| History buffer size | 300 samples | ~75 seconds @ 250ms interval |
| Memory footprint | <5MB | Status monitor + history |
| CPU usage | <2% | Per-core usage during monitoring |

### 10.2 UI Performance

| Metric | Target | Notes |
|--------|--------|-------|
| Status update rate | 5+ FPS | At least 5 times per second |
| Position smoothing | <100ms lag | Display lag behind actual position |
| Animation frame rate | 60 FPS | State indicator animations |
| Chart responsiveness | <200ms | History chart interaction |

### 10.3 Reliability

| Metric | Target | Notes |
|--------|--------|-------|
| Parser success rate | >99.5% | Should handle nearly all responses |
| Monitor uptime | 99.9%+ | Graceful handling of errors |
| Memory stability | No growth | Circular buffer prevents memory leaks |

---

## 11. Documentation

### 11.1 Code Documentation

**DOCBLOCK style** per project standards:
- Module docblocks at top of each file
- Function docblocks with purpose, arguments, return values
- Type docblocks with field descriptions
- Examples in docblocks for complex functions

### 11.2 User Documentation

**Files to create/update**:
- `docs/STATUS_MONITORING.md` - User guide for status display
- `docs/TROUBLESHOOTING_STATUS.md` - Common status issues
- `README.md` - Update feature list
- In-app help text for status widgets

### 11.3 Developer Documentation

**Files to create**:
- `docs/STATUS_MONITORING_ARCHITECTURE.md` - Technical overview
- `docs/GRBL_STATUS_PROTOCOL.md` - Protocol specification
- Code comments for complex algorithms
- Example usage in docblocks

---

## 12. Dependency Analysis

### 12.1 New Dependencies

**Required**:
- None! Uses existing dependencies

**Recommended for Phase 13+ features**:
- `plotly` - Advanced charting (optional, for detailed graphs)
- `serde_json` - Already available, use for config
- `tokio-util` - CancellationToken support

### 12.2 Existing Dependencies Used

- `tokio` - Async task for monitoring
- `tracing` - Structured logging
- `egui` - UI rendering
- `serialport` - Device communication
- `chrono` - Timestamps
- `serde/serde_json` - Config serialization

---

## 13. Migration from Current Implementation

### 13.1 Changes to Existing Code

**`src/communication/grbl.rs`**:
- Add `status_monitor` field
- Add `latest_status` field
- Modify `connect()` to start monitor
- Modify `disconnect()` to stop monitor
- Add status query methods

**`src/layout/bottom_status.rs`**:
- Expand display fields
- Add real-time position updates
- Add buffer indicator
- Add state animation

**`src/app/mod.rs`**:
- Add status monitoring UI toggle
- Add status history access

### 13.2 Backward Compatibility

- No breaking changes to public API
- Existing code continues to work
- New features are additive
- Configuration is optional (sensible defaults)

---

## 14. Success Criteria & Acceptance Tests

### 14.1 Functional Requirements

- ✅ Device status queried every 250ms ± 50ms
- ✅ Status response parsed with >99% success rate
- ✅ Position displayed with <100ms lag
- ✅ Machine state shown in real-time
- ✅ Status history accessible for last 300+ samples
- ✅ State transitions detected and logged
- ✅ Alarms trigger immediate UI alert
- ✅ Buffer fill status visible
- ✅ Pin states (limits, probe) displayed

### 14.2 Non-Functional Requirements

- ✅ Memory usage stable (circular buffer)
- ✅ CPU usage <2% per core
- ✅ UI responsive during status queries
- ✅ 60 FPS animations maintained
- ✅ No memory leaks (valgrind clean)
- ✅ Graceful error recovery
- ✅ No blocking operations on UI thread
- ✅ Configuration persistent

### 14.3 Test Coverage

- ✅ >90% code coverage
- ✅ >200 unit tests
- ✅ >50 integration tests
- ✅ All edge cases handled
- ✅ Malformed input doesn't crash
- ✅ Zero clippy warnings
- ✅ All tests passing

---

## 15. Future Enhancements (Phase 13+)

### 15.1 Advanced Status Features

1. **Predictive Status**: Estimate remaining job time based on feedrate and lines remaining
2. **Status Alerts**: Configurable alerts for state changes, alarms, low buffer
3. **Status Recording**: Record complete status history to file for post-analysis
4. **Thermal Monitoring**: Track spindle/motor temperatures if available
5. **Advanced Charting**: Plotly/WebGL-based interactive charts

### 15.2 Integration with Job System

1. Correlate status changes with job progress
2. Automatic pause on errors
3. Performance optimization based on trends
4. Workpiece tracking (time estimation accuracy)

### 15.3 Machine Learning Features

1. Anomaly detection (unusual vibration patterns)
2. Predictive maintenance (wear prediction)
3. Tool breakage detection
4. Collision detection from position anomalies

---

## 16. Development Workflow

### 16.1 Git Workflow

```bash
# Create feature branch
git checkout -b feature/status-monitoring

# Create sub-branches per phase
git checkout -b feature/status-monitoring-phase-12-1-core

# Commit with conventional commit messages
git commit -m "feat(status): add GRBL status parser with v1.0/1.1/1.2 support"
git commit -m "test(status): add parser edge case tests"

# Create PR for each phase
gh pr create --title "Phase 12.1: Core Status Infrastructure"
```

### 16.2 Code Review Checklist

- ✅ All tests passing
- ✅ Clippy warnings resolved
- ✅ Code formatted with rustfmt
- ✅ Documentation complete
- ✅ No memory issues (Valgrind)
- ✅ Performance targets met
- ✅ Backward compatibility maintained

### 16.3 Release Process

1. Merge feature branch to main
2. Tag with version (e.g., v0.1.1-status-monitoring)
3. Update CHANGELOG.md
4. Build release binary
5. Create GitHub release with notes

---

## 17. Risk Analysis

### 17.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Parser fails on unexpected GRBL version | Medium | Low | Robust parser with fallback, version detection |
| UI lag during frequent updates | Low | High | Async processing, throttled UI updates |
| Memory leak in monitor task | Low | High | Circular buffer, comprehensive testing |
| Serialport communication blocking | Medium | Medium | Async I/O, timeout handling |
| State machine race conditions | Low | High | Careful mutex usage, comprehensive tests |

### 17.2 Mitigation Strategies

1. **Robust Error Handling**: All operations wrapped in Result types
2. **Comprehensive Testing**: >90% code coverage, edge case testing
3. **Performance Profiling**: Profile on each phase completion
4. **Memory Testing**: Valgrind runs on release builds
5. **Stress Testing**: Test with 1000+ status queries
6. **Real Device Testing**: Test with actual CNC/laser equipment

---

## Appendix A: GRBL Status Response Examples

### GRBL v1.0 Format
```
<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100>
<Run|MPos:10.50,5.25,0.00|FS:1500,12000|Ov:100,100,100>
```

### GRBL v1.1 Format (includes WPos, Buf, Line)
```
<Idle|MPos:0.00,0.00,0.00|WPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100|F:0>
<Run|MPos:25.30,10.15,5.00|WPos:25.30,10.15,5.00|FS:1500,12000|Ov:100,100,100|Buf:18|Rx:256|Line:42>
```

### GRBL v1.1 Format (with pin states)
```
<Hold|MPos:15.50,8.25,3.10|FS:0,12000|Ov:120,95,50|Pn:XYZ>
<Alarm|MPos:0.00,0.00,0.00|Pn:X>
```

### GRBL v1.2 Format (enhanced)
```
<Cycle|MPos:50.00,25.00,10.00|FS:3000,18000|Ov:100,100,100|WCO:5.00,10.00,0.00>
```

---

## Appendix B: Testing Checklist

### Unit Tests Required

- [ ] Parser: Valid responses for all GRBL versions
- [ ] Parser: Malformed input handling
- [ ] Parser: Missing optional fields
- [ ] Parser: Boundary values
- [ ] Monitor: Query interval accuracy
- [ ] Monitor: Circular buffer behavior
- [ ] Monitor: Task lifecycle
- [ ] Analytics: Feedrate calculations
- [ ] Analytics: State transitions
- [ ] Integration: GrblCommunication methods
- [ ] UI: Widget rendering
- [ ] UI: Real-time updates

### Integration Tests Required

- [ ] Connect → Monitor starts
- [ ] Disconnect → Monitor stops gracefully
- [ ] Rapid queries → No data loss
- [ ] Device timeout → Graceful recovery
- [ ] Malformed response → Uses cached status
- [ ] Long run → Memory stable
- [ ] UI thread → No blocking

### Performance Tests Required

- [ ] Query latency <50ms
- [ ] UI update rate ≥5 FPS
- [ ] Memory growth <1MB over 1000 queries
- [ ] CPU usage <2%
- [ ] 60 FPS animations maintained

---

## Appendix C: Configuration Schema

```json
{
  "status_monitoring": {
    "enabled": true,
    "query_interval_ms": 250,
    "adaptive_timing": true,
    "history_buffer_size": 300,
    "auto_start_on_connect": true,
    "presets": {
      "responsive": 100,
      "balanced": 250,
      "power_saving": 500
    }
  }
}
```

---

**Document Version**: 1.0  
**Last Updated**: 2024-10-18  
**Author**: AI Assistant (Claude Sonnet 4.5)  
**Status**: Ready for Implementation  

