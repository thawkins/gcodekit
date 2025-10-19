# Device Console Integration - Quick Reference

## What Gets Logged? ✅ vs ❌

| Message Type | Default | Example |
|--------------|---------|---------|
| ✅ User Commands | SHOW | `[COMMAND] G0 X10 Y20` |
| ✅ Device Responses | SHOW | `[RESPONSE] [GC:G0 G54...]` |
| ❌ "ok" Responses | HIDE | `[RESPONSE] ok` (hidden by default) |
| ❌ Status Queries | HIDE | `[COMMAND] ?` (hidden by default) |
| ❌ Status Responses | HIDE | `<Idle\|MPos:0.000...>` (hidden by default) |
| ⚠️ Warnings | SHOW | `[TRACE:WARNING] Buffer full` |
| ❌ Info Messages | HIDE | `[TRACE:INFO] Connected` (hidden by default) |
| 🐛 Debug Messages | HIDE | `[TRACE:DEBUG] Starting read` (hidden by default) |
| ❌ Error Details | SHOW | `[TRACE:ERROR] Timeout` |

## Architecture Overview

```
Device Communication Layer
        ↓
   [Commands Sent]
   [Responses Recv]
        ↓
   Console Logger
   (Filters messages)
        ↓
   Console State
   (Stores filtered)
        ↓
   Console UI Panel
   (Displays with filters)
```

## Data Structures

### ConsoleMessage
```
ConsoleMessage {
    message_type: ConsoleMessageType,  // Command, Response, Trace, etc.
    trace_level: Option<TraceLevel>,   // Debug, Info, Warning, Error
    content: String,                   // The actual message
    timestamp: DateTime<Utc>,          // When it was logged
    source: MessageSource,             // Device, User, System
    raw_data: Option<String>,          // For debugging
}
```

### ConsoleFilterConfig
```
ConsoleFilterConfig {
    show_commands: bool,           // ✅ default: true
    show_responses: bool,          // ✅ default: true
    show_ok_responses: bool,       // ❌ default: false
    show_status_queries: bool,     // ❌ default: false
    show_status_responses: bool,   // ❌ default: false
    show_info: bool,               // ❌ default: false
    show_debug: bool,              // ❌ default: false
    show_warnings: bool,           // ✅ default: true
    show_errors: bool,             // ✅ default: true
}
```

## UI Layout

```
┌──────────────────────────────────────────────────────────┐
│ Device Console                     [📋 Copy] [🗑️ Clear]   │
├──────────────────────────────────────────────────────────┤
│                                                            │
│ ☑ Commands  ☑ Responses  ☐ OK  ☐ Status  ☐ Queries     │
│ ☑ Warnings  ☑ Errors    ☐ Info   ☐ Debug                │
│                                                            │
│ Search: [Enter text to filter...]              [X Auto]  │
│                                                            │
├──────────────────────────────────────────────────────────┤
│                                                            │
│ 11:34:58 [COMMAND]  G0 X10 Y20                            │
│ 11:34:59 [RESPONSE] [GC:G0 G54 G17...]                   │
│ 11:35:00 [COMMAND]  G1 Z-5 F100                           │
│ 11:35:01 [RESPONSE] [GC:G1 G54 G17...]                   │
│                                                            │
└──────────────────────────────────────────────────────────┘
```

## Color Scheme

| Type | Color | RGB |
|------|-------|-----|
| Command | Blue | (100, 200, 255) |
| Response | Green | (100, 255, 150) |
| Debug | Gray | (150, 150, 150) |
| Info | Yellow | (200, 200, 100) |
| Warning | Orange | (255, 150, 0) |
| Error | Red | (255, 100, 100) |
| Status | Purple | (200, 100, 255) |

## Phase Breakdown

| Phase | Focus | LOC | Tests | Timeline |
|-------|-------|-----|-------|----------|
| 13.0 | Planning & Architecture | - | - | ✅ Complete |
| 13.1 | Core Logger | 200 | 32 | 3-4 hours |
| 13.2 | GRBL Integration | 150 | 32 | 3-4 hours |
| 13.3 | Console UI | 300 | 28 | 4-5 hours |
| 13.4 | Tracing | 150 | 24 | 2-3 hours |
| 13.5 | Polish & Export | 200 | 30 | 3-4 hours |
| **TOTAL** | **All Phases** | **1,200** | **144** | **15-20 hours** |

## Key Implementation Points

### 1. No "?" Logging
```rust
// Automatically filter out status queries
if command == "?" {
    // Don't log to console
    return;
}
```

### 2. No Status Response Logging
```rust
// Automatically filter out status responses
if response.starts_with("<") && response.ends_with(">") {
    // This is a status response, don't log
    return;
}
```

### 3. No "ok" Logging (by default)
```rust
// Check filter before logging "ok"
if response == "ok" && !config.show_ok_responses {
    return;
}
```

### 4. Configurable Filtering
Users can toggle:
- ☑️ Commands visibility
- ☑️ Responses visibility
- ☐ OK responses
- ☐ Status queries
- ⚠️ Warning level
- 🐛 Debug level
- ℹ️ Info level

### 5. Tracing Integration
All `tracing::{debug, info, warn, error}!()` calls are captured and logged to console.

## Usage Examples

### Example 1: Normal Operation
```
11:34:58 [COMMAND] G0 X10 Y20
11:34:59 [RESPONSE] [GC:G0 G54 G17 G21 G90 G94 M5 M9 T0 F0 S0]
11:35:00 [COMMAND] G1 Z-5 F100
11:35:01 [RESPONSE] [GC:G1 G54 G17 G21 G90 G94 M5 M9 T0 F0 S0]
```

### Example 2: With Debug Enabled
```
11:34:58 [TRACE:DEBUG] Starting command send
11:34:58 [COMMAND] G0 X10 Y20
11:34:58 [TRACE:DEBUG] Waiting for response
11:34:59 [RESPONSE] [GC:G0 G54 G17...]
11:34:59 [TRACE:DEBUG] Response parsed successfully
```

### Example 3: Error Case
```
11:34:58 [COMMAND] G0 X10 Y20
11:34:59 [TRACE:WARNING] No response after 2s
11:35:00 [TRACE:ERROR] Command timeout
11:35:00 [RESPONSE] error:5
```

## Performance Considerations

- **Max Messages**: 2,000 (circular buffer)
- **Memory Usage**: ~1MB per 1000 messages
- **Filtering Cost**: O(1) per message
- **Display Update**: Only on new messages
- **Search Performance**: O(n) where n = filtered messages

## File Structure

```
src/
├── communication/
│   ├── console_logger.rs      (Phase 13.1) - NEW
│   ├── console_tracing.rs     (Phase 13.4) - NEW
│   └── grbl.rs                (Phase 13.2) - MODIFIED
├── app/
│   └── state.rs               (Phase 13.2) - MODIFIED
└── ui/
    ├── console_panel.rs       (Phase 13.3) - NEW
    ├── tabs/device_console.rs (Phase 13.3) - MODIFIED
    └── mod.rs                 (Phase 13.3) - MODIFIED
```

## Testing Strategy

- **Unit Tests**: Message filtering logic, config validation
- **Integration Tests**: GRBL communication logging, tracing capture
- **UI Tests**: Filter state, display correctness
- **Performance Tests**: Large message buffers, filtering speed

## Next Steps

1. ✅ Review and approve plan
2. Proceed to Phase 13.1 (Core Logger)
3. Proceed to Phase 13.2 (GRBL Integration)
4. Proceed to Phase 13.3 (Console UI)
5. Proceed to Phase 13.4 (Tracing)
6. Proceed to Phase 13.5 (Polish)

