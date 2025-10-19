# Real-Time Status Monitoring - Technical Architecture

## System Architecture Overview

### Component Interaction Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                     Main Application Thread                      │
│                                                                  │
│  ┌─────────────┐  ┌────────────────┐  ┌──────────────────────┐  │
│  │ egui Context│  │ App State      │  │ Device Connection    │  │
│  │             │  │ - Jobs         │  │ - Serial Port        │  │
│  │ - Frame     │  │ - Materials    │  │ - GRBL Version       │  │
│  │ - Events    │  │ - Settings     │  │ - Status Data        │  │
│  │ - Rendering │  │                │  │                      │  │
│  └─────────────┘  └────────────────┘  └──────────────────────┘  │
│         △                 △                      △               │
│         │                 │                      │               │
│         └─────────────────┼──────────────────────┘               │
│                           │ Updates via                          │
│                    Arc<Mutex<MachineStatus>>                    │
│                           │                                      │
└───────────────────────────┼──────────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────┐
        │   Status Monitor Tokio Task       │ ← Async thread
        │  (Runs on tokio runtime)          │
        │                                   │
        │  1. Sleep 250ms                   │
        │  2. Send "?" to device            │
        │  3. Read response (non-blocking)  │
        │  4. Parse to MachineStatus        │
        │  5. Store in circular buffer      │
        │  6. Update Arc<Mutex<Status>>     │
        │  7. Repeat from step 1            │
        │                                   │
        │  Thread-safe: Arc<Mutex<>>        │
        │  Cancellable: CancellationToken   │
        └───────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────┐
        │     Serial Communication          │
        │  (via serialport crate)           │
        │                                   │
        │  Send: "?\n" (4 bytes)            │
        │  Receive: Status response         │
        │  Timeout: 100ms                   │
        └───────────────────────────────────┘
                            │
                            ▼
        ┌───────────────────────────────────┐
        │      GRBL Device / Controller     │
        │                                   │
        │  Processes "?" command            │
        │  Returns: <State|MPos:...>        │
        │  Response time: <10ms typical     │
        └───────────────────────────────────┘
```

---

## Data Flow Sequence Diagram

### Status Update Cycle (250ms)

```
Main UI Thread              Status Monitor Task        Serial Device
      │                           │                         │
      │                           │ [250ms timer expires]    │
      │                           ├────────────────────────> │ Send "?"
      │                           │                    <─────┤ Response ready
      │                           │<─── Read response ────── │
      │                           │                         │
      │                           │ [Parse response]         │
      │ [Render frame]            │ - Extract fields        │
      │ [Read Arc<Mutex<>>]   <──┤ - Create MachineStatus   │
      │ [Update display] ◄──────┤ - Update Arc<Mutex<>>    │
      │                           │                         │
      │ [Wait for next frame]     │ [Store in history buf]   │
      │                           │ [Calc analytics]        │
      │                           │ [Sleep 250ms]            │
      │                           │                         │
      │◄──────── Repeat cycle ───┤                         │
```

### Connection Lifecycle

```
Application Start
       │
       ▼
User selects device
       │
       ▼
User clicks "Connect"
       │
       ├──> Open serial port
       │
       ├──> Send "!" (version query)
       │
       ├──> Receive version string
       │
       ├──> Create StatusMonitor
       │
       ├──> Spawn async Tokio task
       │         ├─> Start polling "?"
       │         ├─> History buffer initialized
       │         └─> Ready for UI updates
       │
       ├──> Set ConnectionState = Connected
       │
       └──> Display "Connected" in UI
              with live status updates

Device Disconnection
       │
       ├──> Send cancel signal to monitor task
       │
       ├──> Task gracefully stops (cleans resources)
       │
       ├──> Close serial port
       │
       └──> Set ConnectionState = Disconnected
```

---

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│ src/main.rs & src/app/mod.rs                                │
│ (Application orchestrator)                                  │
├─────────────────────────────────────────────────────────────┤
                        △
                        │ depends on
                        │
        ┌───────────────┼───────────────┐
        │               │               │
        ▼               ▼               ▼
    ┌───────┐  ┌──────────────┐  ┌────────────┐
    │ Widgets│  │ Layout       │  │ Config     │
    │        │  │ - Status bar │  │ - Monitor  │
    └───────┘  └──────────────┘  │   config   │
        △               △         └────────────┘
        │               │                △
        └───────┬───────┘                │
                │ uses                   │
                ▼                        │
        ┌──────────────────────────────┐│
        │ communication/grbl.rs         ││
        │ - GrblCommunication struct    ││
        │ - connection_state            ││
        │ - device I/O                  ││
        └──────────────────────────────┘│
                △                        │
                │ depends on             │
                │                        ▼
        ┌──────────────────────────────┐┘
        │ StatusMonitor (async task)   │
        │ - Periodic queries           │
        │ - Response parsing           │
        │ - History management         │
        └──────────────────────────────┘
                △
                │ uses
                │
        ┌──────────────────────────────┐
        │ communication/status_parser  │
        │ - Parse GRBL response        │
        │ - Extract fields             │
        │ - Validate data              │
        └──────────────────────────────┘
                △
                │ uses
                │
        ┌──────────────────────────────┐
        │ communication/grbl_status    │
        │ - MachineStatus struct       │
        │ - Position struct            │
        │ - FeedSpeed struct           │
        │ - OverrideState struct       │
        └──────────────────────────────┘
```

---

## Type Definitions & Data Structures

### Core Status Type

```rust
/// Real-time machine status snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachineStatus {
    /// Current machine state (Idle, Run, Hold, etc.)
    pub state: MachineState,
    
    /// Machine position (absolute)
    pub machine_position: Position,
    
    /// Work position (relative to WCS)
    pub work_position: Option<Position>,
    
    /// Feed rate and spindle speed
    pub feed_speed: FeedSpeed,
    
    /// Override percentages
    pub overrides: OverrideState,
    
    /// Current line number executing
    pub line_number: Option<u32>,
    
    /// Buffer status
    pub buffer_state: BufferState,
    
    /// Input pin states
    pub pin_states: PinStates,
    
    /// Feedback metrics
    pub feedback: FeedbackMetrics,
    
    /// Timestamp when captured
    pub timestamp: Instant,
}

/// Memory layout: ~150 bytes per status
/// Circular buffer of 300: ~45 KB total
```

### Position Type

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,  // 4 bytes
    pub y: f32,  // 4 bytes
    pub z: f32,  // 4 bytes
    pub a: Option<f32>,  // 5 bytes (1 for discriminant)
    pub b: Option<f32>,  // 5 bytes
    pub c: Option<f32>,  // 5 bytes
}
// Total: ~28 bytes
```

### State Machine

```
     ┌─────────┐
     │  IDLE   │ ◄─────────────┐
     └────┬────┘               │
          │ run job            │ error / stop
          ▼                    │
     ┌─────────┐          ┌────────────┐
     │   RUN   │          │   ALARM    │
     └────┬────┘          └────────────┘
          │ pause               ▲
          ▼                     │
     ┌─────────┐          ┌──────────┐
     │  HOLD   │ ─error──►│ CRITICAL │
     └────┬────┘          └──────────┘
          │ resume
          └─────────────────────┘

State transitions trigger callbacks for UI updates
```

---

## Parser Implementation Details

### Parser State Machine

```
Input: "<Run|MPos:10.5,5.25,2.1|FS:1500,12000|Ov:100,100,100>"

State 1: Extract outer angle brackets
  "<" → Start parsing
  ">" → End parsing
  Content: "Run|MPos:10.5,5.25,2.1|FS:1500,12000|Ov:100,100,100"

State 2: Split by "|" delimiter
  Parts: ["Run", "MPos:10.5,5.25,2.1", "FS:1500,12000", "Ov:100,100,100"]

State 3: Parse each field
  "Run" → MachineState::Run
  "MPos:10.5,5.25,2.1" → Position { x: 10.5, y: 5.25, z: 2.1, ... }
  "FS:1500,12000" → FeedSpeed { feed_rate: 1500.0, spindle_speed: 12000.0 }
  "Ov:100,100,100" → OverrideState { feed: 100, spindle: 100, coolant: 100 }

State 4: Optional fields parsing
  "WPos:..." → If present, parse work position
  "Buf:..." → If present, parse buffer status
  "Line:..." → If present, parse line number
  "Pn:..." → If present, parse pin states

State 5: Validate result
  All required fields present? ✓
  Numeric values in valid range? ✓
  Timestamp attached? ✓

Output: MachineStatus { ... }
```

### Parser Error Handling

```
Strategy: Fault-tolerant parsing with fallback

┌─ Parse field ─────────────┐
│                           │
├─ Numeric conversion fails │
│  └─> Use last valid value │
│      or default value      │
│                           │
├─ Field missing            │
│  └─> Mark as None or use  │
│      default              │
│                           │
├─ Invalid state string     │
│  └─> Set state: Unknown   │
│                           │
├─ Coordinate overflow      │
│  └─> Clamp to reasonable  │
│      limits or use cache   │
│                           │
└─ Malformed format         │
   └─> Use previous status  │
       + log warning         │
```

---

## Async Task Implementation

### Status Monitor Task Lifecycle

```rust
// Pseudo-code for monitor task

pub async fn status_monitor_loop(
    mut device: Arc<Mutex<GrblCommunication>>,
    mut status: Arc<Mutex<MachineStatus>>,
    mut history: Arc<Mutex<VecDeque<MachineStatus>>>,
    config: StatusMonitorConfig,
    cancel: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                // Graceful shutdown
                debug!("Status monitor cancelled");
                break;
            }
            
            _ = tokio::time::sleep(Duration::from_millis(
                config.query_interval_ms
            )) => {
                // Time to query device
                match query_device(&device).await {
                    Ok(response) => {
                        match parse_status_response(&response) {
                            Ok(new_status) => {
                                // Update current status
                                {
                                    let mut s = status.lock().unwrap();
                                    *s = new_status.clone();
                                }
                                
                                // Update history
                                {
                                    let mut h = history.lock().unwrap();
                                    h.push_back(new_status);
                                    if h.len() > config.history_buffer_size {
                                        h.pop_front(); // Circular
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Parse error: {}", e);
                                // Keep using previous status
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Query error: {}", e);
                        // Retry next cycle
                    }
                }
            }
        }
    }
}

// Main thread starts monitor:
let cancel = CancellationToken::new();
let handle = tokio::spawn(status_monitor_loop(
    device,
    status,
    history,
    config,
    cancel.clone(),
));

// Later, when disconnecting:
cancel.cancel();
handle.await.ok();
```

---

## Thread Safety & Synchronization

### Shared Data Protection

```
Arc<Mutex<MachineStatus>>
  ├─ Arc: Atomic Reference Count
  │  └─ Owned by multiple threads
  │     (Monitor task + UI task)
  │
  └─ Mutex: Mutual Exclusion
     └─ Lock-based access
        - One reader at a time
        - Writers block readers


Access Pattern:

UI Thread:
  let mut guard = status.lock().unwrap();
  let pos = guard.machine_position;
  drop(guard); // release lock

Monitor Thread:
  let mut guard = status.lock().unwrap();
  guard.machine_position = new_pos;
  drop(guard); // release lock

Result: No data races, but lock contention
```

### Lock Duration Minimization

```rust
// Good: Quick lock, get data, release
{
    let status = status_arc.lock().unwrap();
    let pos = status.machine_position.clone();
} // Lock released here
// Use pos without holding lock
render_position(&pos);

// Bad: Holding lock during expensive operation
{
    let status = status_arc.lock().unwrap();
    render_position(&status.machine_position); // ← Lock held
    update_screen(); // ← Lock still held! ✗
} // Lock released here
```

---

## History Buffer Management

### Circular Buffer Implementation

```rust
pub struct HistoryBuffer {
    buffer: VecDeque<MachineStatus>,
    max_size: usize,
}

impl HistoryBuffer {
    pub fn push(&mut self, status: MachineStatus) {
        self.buffer.push_back(status);
        
        // Circular: discard oldest when full
        if self.buffer.len() > self.max_size {
            self.buffer.pop_front();
        }
    }
}

// Memory usage:
// 300 samples × 150 bytes = 45 KB (stable)
// No unbounded growth ✓

// Visualization:
//
// Sample 1    Sample 2    Sample 3    ...    Sample 300
//  │           │           │                     │
//  ▼           ▼           ▼                     ▼
// ┌───────┬───────┬───────┬─ ─ ─┬───────┐
// │Status │Status │Status │     │Status │
// │  @t1  │  @t2  │  @t3  │     │  @t300│
// └───────┴───────┴───────┴─ ─ ─┴───────┘
//  ▲                                       ▲
//  ├──────── 75 seconds history ─────────┘
//     (at 250ms per sample)
//
// When new sample arrives:
// 1. Append to back
// 2. If size > 300: remove from front
// 3. Result: Sliding window of last 300 samples
```

---

## Performance Considerations

### Query Timing Analysis

```
Timeline for single query cycle:

t=0ms      Send "?" to device
           Transmission time: ~1ms

t=1ms      Device receives "?"
           Processing time: <10ms

t=10ms     Device sends response
           (typical ~50 bytes)
           Transmission time: ~5ms

t=15ms     Response received
           Parse response: ~5ms
           Update Arc<Mutex<>>: <1ms

t=21ms     Complete!
           Total latency: ~21ms
           Plus padding to 250ms: 229ms sleep

t=250ms    Next cycle begins
```

### Adaptive Query Timing

```
When state = Run:
  Query faster (100ms)
  ├─ User sees position updates in real-time
  └─ Total CPU: ~2%

When state = Idle:
  Query slower (500ms)
  ├─ Position not changing, less critical
  └─ Total CPU: <0.5%

Adaptive algorithm:
  if machine_state == Run {
      query_interval = 100ms
  } else {
      query_interval = 500ms
  }

Benefit: Responsive during work, power-efficient at rest
```

---

## Error Recovery Flow

### Parser Error Handling

```
Parse Status Response
  ↓
├─ Extract outer <...>
│  └─ Fail? Use last valid status
│
├─ Split by |
│  └─ Fail? Use last valid status
│
├─ Parse state
│  └─ Unrecognized? Set to "Unknown"
│
├─ Parse position
│  └─ NaN/overflow? Use previous position
│
├─ Parse optional fields
│  └─ Missing? Leave as None
│
└─ Validate result
   └─ Out of range? Clamp to limits

Success: Return new MachineStatus
Failure: Return cached MachineStatus + warning
```

### Communication Error Handling

```
Query Device
  ├─ Send "?" ────────────┐
  │                       │
  ├─ Read response (100ms timeout)
  │  ├─ Success → Parse
  │  ├─ Timeout → Keep previous status, log warning
  │  ├─ Error → Keep previous status, retry next cycle
  │  └─ Disconnected → Stop monitor gracefully
  │
  └─ On repeated errors:
     ├─ Count consecutive failures
     ├─ After 3 failures: log "device may be disconnected"
     └─ After 10 failures: suggest reconnect
```

---

## UI Integration Points

### Status Display Widget Update Flow

```
egui::Window("Status") {
    // Get latest status (non-blocking)
    let current = {
        let guard = status_arc.lock().unwrap();
        guard.clone()  // Quick clone
    };
    
    // Render status
    ui.label(format!("State: {:?}", current.state));
    ui.label(format!("Pos: {:.2}, {:.2}, {:.2}",
        current.machine_position.x,
        current.machine_position.y,
        current.machine_position.z,
    ));
    
    // Show buffer indicator
    ui.horizontal(|ui| {
        let fill = current.buffer_state.planner_buffer as f32 / 128.0;
        ui.add(ProgressBar::new(fill));
    });
}
```

### Animation Update

```
State Indicator (animated):
  ├─ If state == Run:
  │  └─ Draw pulsing circle (pulse frequency: 2 Hz)
  ├─ If state == Hold:
  │  └─ Draw slow pulse (0.5 Hz)
  ├─ If state == Alarm:
  │  └─ Draw blinking (5 Hz)
  └─ Else:
     └─ Draw static indicator

Animation driver:
  pub fn get_animation_frame(state: MachineState, t: f32) -> f32 {
      match state {
          MachineState::Run => {
              let cycle = ((t * 2.0) % 1.0);
              1.0 - (4.0 * cycle * (1.0 - cycle))  // Sine-like pulse
          }
          MachineState::Hold => {
              ((t * 0.5) % 1.0)  // Slower pulse
          }
          MachineState::Alarm => {
              (t * 5.0) % 2.0 > 1.0  // Binary blink
          }
          _ => 0.0
      }
  }
```

---

## Configuration Schema

### Status Monitor Config File

Location: `~/.config/gcodekit/status_monitor.json`

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
    },
    "alerts": {
      "enable_state_change_alert": true,
      "enable_alarm_alert": true,
      "enable_buffer_full_alert": false
    }
  }
}
```

---

## Testing Architecture

### Test Categories

```
Unit Tests (communication/grbl_status.rs)
  ├─ Parser tests
  │  ├─ Valid responses (GRBL v1.0, 1.1, 1.2)
  │  ├─ Malformed input
  │  ├─ Missing fields
  │  ├─ Boundary values
  │  └─ Edge cases
  └─ Type tests
     ├─ Serialization
     ├─ Default values
     └─ Conversions

Integration Tests (status_monitor.rs)
  ├─ Monitor lifecycle
  │  ├─ Start/stop
  │  ├─ Task cancellation
  │  └─ Cleanup
  ├─ Query accuracy
  │  ├─ Interval timing
  │  ├─ Response parsing
  │  └─ History buffer
  ├─ Concurrency
  │  ├─ Lock contention
  │  ├─ Race conditions
  │  └─ Memory safety
  └─ Error scenarios
     ├─ Malformed response
     ├─ Timeout
     ├─ Disconnection
     └─ Rapid reconnect

Performance Tests
  ├─ Latency: <50ms per query
  ├─ Memory: <5MB total, no growth
  ├─ CPU: <2% per core
  └─ Frame rate: 60 FPS maintained
```

---

## Deployment Considerations

### Backward Compatibility

```
Existing code using GrblCommunication:
  let mut comm = GrblCommunication::new();
  comm.connect(port).await;
  // Status monitor auto-starts if enabled ✓
  
  // Works as before, but now with live status
  let status = comm.get_current_status(); // New method
  let history = comm.get_status_history(10); // New method
  
Result: 100% backward compatible
```

### Feature Flags (Optional)

```
[features]
default = ["status-monitoring"]
status-monitoring = []
minimal = []  # Excludes monitor for embedded use

In code:
#[cfg(feature = "status-monitoring")]
pub fn get_status_history(&self) -> Vec<MachineStatus> { ... }
```

---

## Monitoring & Debugging

### Debug Logging

```rust
// Enable with RUST_LOG=gcodekit::communication=debug

debug!("Query sent: '{}'", query);
debug!("Response: '{}'", response);
debug!("Parsed: {:?}", status);
debug!("History size: {}", history.len());
debug!("Parse latency: {}ms", elapsed.as_millis());

// Warnings
warn!("Parse error: {}", error);
warn!("Device timeout");
warn!("Buffer near full: {}", buffer_fill);

// Info
info!("Monitor started");
info!("Monitor stopped");
```

### Metrics Collection

```
pub struct MonitorMetrics {
    pub queries_sent: u64,
    pub queries_successful: u64,
    pub parse_errors: u64,
    pub total_latency: Duration,
    pub peak_latency: Duration,
}

Calculated values:
- Success rate: queries_successful / queries_sent * 100%
- Average latency: total_latency / queries_sent
- Error frequency: parse_errors / queries_sent * 100%

Exposed via:
let metrics = comm.get_status_monitor_metrics();
info!("Success: {:.1}%", metrics.success_rate());
```

---

**Document Version**: 1.0  
**Last Updated**: 2024-10-18  
**Technical Level**: Intermediate/Advanced  
**Audience**: Developers implementing the system  

