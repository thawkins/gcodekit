# Real-Time Status Monitoring - Quick Start Guide

## What is Real-Time Status Monitoring?

The system continuously queries your GRBL device using the "?" command (every 250ms by default) to get live machine status. This data includes position, machine state, feed rate, spindle speed, and more.

**Key Benefit**: Real-time feedback on what your CNC/laser is doing - no delays, no guessing.

---

## Architecture at a Glance

### The Five-Layer Stack

```
┌─────────────────────────────┐
│ 1. UI Display Layer         │ ← What you see
│    (Widgets, Graphs, Alerts)│
├─────────────────────────────┤
│ 2. UI Integration Layer     │ ← egui components
│    (Status Display Widget)  │
├─────────────────────────────┤
│ 3. Analytics & History      │ ← Trends & metrics
│    (Circular Buffer, Stats) │
├─────────────────────────────┤
│ 4. Status Monitor Task      │ ← Async polling loop
│    (Tokio, Periodic "?")    │
├─────────────────────────────┤
│ 5. Parser & Data Structures │ ← Decode responses
│    (Parser, Types)          │
└─────────────────────────────┘
         ↓ (via serial)
    ┌─────────────┐
    │ GRBL Device │ ← Provides status data
    └─────────────┘
```

---

## Implementation Summary

### Phase 12.1: Core Infrastructure (Week 1)
**Build the foundation**: Data types, parser, tests

**Files to Create**:
- `src/communication/grbl_status.rs` - Status data structures
- `src/communication/status_parser.rs` - Parse device responses
- Tests for parser

**What it does**: Defines what a "status" looks like and how to decode GRBL's "?" response.

**Example**:
```
Device sends: <Run|MPos:10.50,5.25,2.10|FS:1500,12000|Ov:100,100,100>
Parser outputs: MachineStatus {
    state: Run,
    machine_position: Position { x: 10.50, y: 5.25, z: 2.10 },
    feed_speed: FeedSpeed { feed_rate: 1500, spindle_speed: 12000 },
    ...
}
```

**Deliverables**:
- ✅ Parser handles GRBL v1.0, 1.1, 1.2
- ✅ 100+ test cases
- ✅ Handles malformed data gracefully

---

### Phase 12.2: Monitoring Task (Week 2)
**The polling engine**: Async task that periodically queries device

**Files to Create**:
- `src/communication/status_monitor.rs` - Async polling task
- `src/communication/status_analytics.rs` - Metrics calculation
- Tests for monitor lifecycle

**What it does**: Runs in background, sends "?" every 250ms, stores results in circular buffer.

**Key Code Pattern**:
```rust
loop {
    // 1. Send "?" to device
    device.send_command("?");
    
    // 2. Parse response
    let status = parse_status_response(&response);
    
    // 3. Store in history (circular buffer)
    history.push(status.clone());
    
    // 4. Calculate analytics
    let avg_feedrate = calculate_average(&history);
    
    // 5. Wait 250ms
    tokio::time::sleep(Duration::from_millis(250)).await;
}
```

**Deliverables**:
- ✅ Monitor task starts/stops cleanly
- ✅ History buffer doesn't grow unbounded
- ✅ 50+ integration tests
- ✅ Memory stable over time

---

### Phase 12.3: UI Display (Week 3)
**Show the data**: Real-time UI widgets and animations

**Files to Create/Update**:
- `src/widgets/status_display.rs` - Live status widget
- `src/widgets/state_indicator.rs` - Animated state indicator
- `src/widgets/status_history.rs` - Graph of history
- Update `src/layout/bottom_status.rs` - Status bar

**What it does**: Renders live position, state, feedrate, overrides, buffer status on screen.

**Visual Example**:
```
┌─ LIVE STATUS ──────────────────────────┐
│ State: ● RUNNING                       │
│ Position: X: 123.45  Y: 67.89  Z: 10.00
│ Feed: 1500 mm/min  Spindle: 12000 RPM │
│ Overrides: Feed 100% | Spindle 100%    │
│ Buffer: ████████░░ (80%)               │
│ Pins: Probe □  X-Lim □  Y-Lim □ Z-Lim │
└────────────────────────────────────────┘
```

**Deliverables**:
- ✅ Updates 5+ times per second
- ✅ Smooth position display (no jitter)
- ✅ 60 FPS animations
- ✅ No UI lag

---

### Phase 12.4: Polish & Optimization (Week 4)
**Fine-tune**: Configuration, performance, advanced features

**Files to Create/Update**:
- `src/config/status_monitor_config.rs` - Settings
- Performance optimizations
- Adaptive query timing

**What it does**: Save user preferences, optimize for different scenarios (responsive vs power-saving).

**Deliverables**:
- ✅ Config persists across sessions
- ✅ Presets: Responsive (100ms), Balanced (250ms), Power-saving (500ms)
- ✅ Adaptive timing (faster when Running, slower when Idle)
- ✅ Graceful degradation on errors

---

## Implementation Checklist

### Phase 12.1
- [ ] Define `MachineStatus`, `Position`, `FeedSpeed`, `OverrideState` types
- [ ] Implement status parser for GRBL v1.0, 1.1, 1.2
- [ ] Write parser tests (100+ cases covering edge cases)
- [ ] Add error types for parsing failures
- [ ] Validate types with clippy

### Phase 12.2
- [ ] Create async `StatusMonitor` with `tokio::spawn`
- [ ] Implement circular buffer for history (300 samples)
- [ ] Create analytics module (average feedrate, trends, etc.)
- [ ] Add status callbacks for UI updates
- [ ] Test monitor lifecycle, memory stability
- [ ] Integrate with `GrblCommunication`

### Phase 12.3
- [ ] Create `status_display.rs` widget
- [ ] Add `state_indicator.rs` with animations
- [ ] Create `status_history.rs` visualizer
- [ ] Update bottom status bar
- [ ] Test UI rendering, responsiveness, animations
- [ ] Verify 60 FPS maintained

### Phase 12.4
- [ ] Add configuration struct and persistence
- [ ] Implement adaptive query timing
- [ ] Add status alerts (state changes, alarms)
- [ ] Create preferences dialog
- [ ] Final testing, documentation
- [ ] Performance profiling

---

## Key Design Decisions

### 1. Query Interval: 250ms (default)
- **Why**: 4 updates/sec is responsive without overwhelming device
- **Justification**: GRBL devices can handle high rates; 250ms is proven reliable
- **Tradeoff**: Faster = more load on device; slower = less responsive UI

### 2. Circular Buffer (300 samples)
- **Why**: ~75 seconds of history at 250ms interval
- **Prevents**: Memory leaks from unbounded buffer growth
- **Benefit**: Enough data for trends without excessive memory

### 3. Async Task (Tokio)
- **Why**: Non-blocking polling doesn't freeze UI
- **Alternative**: Spawning threads (harder to manage lifecycle)
- **Benefit**: Tokio handles cleanup gracefully on shutdown

### 4. Graceful Error Handling
- **Missing fields**: Use defaults (e.g., no WPos = use MPos)
- **Parse failures**: Keep using last valid status
- **Device timeout**: Continue with cached data, retry next cycle
- **Principle**: System degrades gracefully, never crashes

### 5. Thread-Safe Sharing
- **Method**: `Arc<Mutex<MachineStatus>>`
- **Why**: Safe access from multiple threads (UI + monitor task)
- **Alternative**: Message passing (more complex)

---

## Testing Strategy

### Unit Tests (Parser)
```rust
#[test]
fn parse_grbl_v1_1_response() {
    let resp = "<Run|MPos:10.5,5.25,2.1|FS:1500,12000|Ov:100,100,100>";
    let status = parse_status_response(resp).unwrap();
    
    assert_eq!(status.state, MachineState::Run);
    assert_eq!(status.machine_position.x, 10.5);
    assert_eq!(status.feed_speed.feed_rate, 1500.0);
}
```

### Integration Tests (Monitor)
```rust
#[tokio::test]
async fn monitor_queries_device_every_250ms() {
    let mut monitor = StatusMonitor::new(default_config());
    monitor.start(device).await;
    
    tokio::time::sleep(Duration::from_secs(1)).await;
    let history = monitor.get_status_history(10);
    
    // Should have ~4-5 samples in 1 second
    assert!(history.len() >= 4);
    assert!(history.len() <= 5);
    
    monitor.stop().await;
}
```

### Performance Tests
```
Query latency: <50ms ✓
UI update rate: 5+ FPS ✓
Memory growth: <1MB/1000 queries ✓
CPU usage: <2% ✓
```

---

## Common Issues & Solutions

### Issue: Parser fails on new GRBL version
**Solution**: Use default values for missing fields; add version detection

### Issue: UI lags during rapid status updates
**Solution**: Throttle UI updates to 5 FPS; use async channels

### Issue: Memory grows unbounded
**Solution**: Use circular buffer (discard oldest when full)

### Issue: Status occasionally null/zero
**Solution**: Implement fallback to last valid status

---

## Performance Targets

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Query latency | <50ms | Time from "?" to parsed result |
| Update frequency | 5+ FPS | UI frame rate when monitoring |
| Memory usage | <5MB | `top` / htop during long run |
| CPU usage | <2% | `top` per core |
| Animation FPS | 60 FPS | egui frame counter |
| Parser success | >99.5% | Test with 1000+ responses |

---

## File Structure

```
src/
├── communication/
│   ├── grbl_status.rs        ← Phase 12.1: Types
│   ├── status_parser.rs      ← Phase 12.1: Parser
│   ├── status_monitor.rs     ← Phase 12.2: Monitor task
│   ├── status_analytics.rs   ← Phase 12.2: Analytics
│   └── grbl.rs               ← Update: Add monitor field
│
├── widgets/
│   ├── status_display.rs     ← Phase 12.3: Display widget
│   ├── state_indicator.rs    ← Phase 12.3: Animated indicator
│   └── status_history.rs     ← Phase 12.3: History visualizer
│
├── layout/
│   └── bottom_status.rs      ← Update: Expand status bar
│
└── config/
    └── status_monitor_config.rs  ← Phase 12.4: Configuration

tests/
└── communication/
    ├── status_parser_tests.rs    ← Phase 12.1: 100+ test cases
    └── status_monitor_tests.rs   ← Phase 12.2: Integration tests

docs/
├── REAL_TIME_STATUS_MONITORING_PLAN.md      ← This full plan
├── STATUS_MONITORING_QUICK_START.md         ← This file
└── STATUS_MONITORING_ARCHITECTURE.md        ← Implementation guide
```

---

## Success Criteria

### Minimal Success
- ✅ Device queried every 250ms
- ✅ Status parsed with 99%+ success
- ✅ Position displayed on screen
- ✅ No crashes or memory leaks

### Full Success
- ✅ All phases completed
- ✅ 300+ passing tests
- ✅ Performance targets met
- ✅ Documentation complete
- ✅ User can see real-time status clearly
- ✅ Configuration works, persists
- ✅ Responsive to state changes

### Production Ready
- ✅ All above criteria met
- ✅ Zero clippy warnings
- ✅ Stress tested (1000+ queries)
- ✅ Real device testing complete
- ✅ Edge cases handled
- ✅ Configuration backup/restore works

---

## Reference: GRBL Status Format

### Minimal Response (GRBL v1.0)
```
<Idle|MPos:0.00,0.00,0.00|FS:0,0|Ov:100,100,100>
```

### Full Response (GRBL v1.1+)
```
<Run|MPos:10.5,5.25,2.1|WPos:10.5,5.25,2.1|FS:1500,12000|Ov:100,100,100|Buf:18|Rx:256|Line:42>
```

### With Alarms
```
<Alarm|MPos:0.00,0.00,0.00|Pn:X>
```

### Fields Reference
| Field | Format | Meaning |
|-------|--------|---------|
| State | `<State` | Idle, Run, Hold, Jog, Alarm, Door, Check, Home, Sleep |
| MPos | `MPos:X,Y,Z` | Machine Position (absolute) |
| WPos | `WPos:X,Y,Z` | Work Position (relative) |
| FS | `FS:feed,spindle` | Feed rate (mm/min), Spindle (RPM) |
| Ov | `Ov:feed,spindle,laser` | Override % (100% = normal) |
| Pn | `Pn:XYZ` | Pin states (limit switches, probe, etc.) |
| Buf | `Buf:n` | Planner buffer fill count |
| Rx | `Rx:n` | Serial RX buffer fill count |
| Line | `Line:n` | Current line number executing |

---

## Next Steps

1. **Review & Approve Plan** - Get stakeholder sign-off
2. **Create Issue** - Phase 12.1 core infrastructure
3. **Start Phase 12.1** - Implement types and parser
4. **Continuous Testing** - Run tests after each phase
5. **Documentation** - Update as phases complete
6. **Real Device Testing** - Test on actual CNC/laser
7. **Performance Tuning** - Optimize based on profiling
8. **Release** - Tag and release v0.2.0

---

**Document Version**: 1.0  
**Last Updated**: 2024-10-18  
**Difficulty**: Medium (parser logic, async tasks)  
**Estimated Effort**: 4 weeks full-time  
**Team Size**: 1-2 developers  

