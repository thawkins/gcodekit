# Comprehensive Implementation Plan: Phases 12 & 13

## Current Status Assessment

**Completed**:
- Phase 12.1: Status query infrastructure (status_monitor.rs exists)
- Phase 12.2: Status update integration (app/state.rs has realtime_status field)
- Phase 12.3: Status analytics (status_analytics.rs with anomaly detection)
- Phase 12.4: Status display (bottom_status.rs shows real-time data)
- Phase 13.1: Console message architecture (device_logger.rs with filtering)
- Phase 13.2: Basic console UI (device_console.rs tab exists)

**Current Issues**:
- status_manager.rs has unused_must_use warning (clear_history is async but not awaited)
- Anomaly detection UI not integrated yet
- Need to verify device console is properly capturing commands/responses
- Need to remove anomaly detection UI elements per request

---

## Remaining Work

### Phase 12.2-12.4 Verification & Enhancement
1. Fix status_manager.rs warning
2. Verify status updates flow correctly to app state
3. Verify bottom_status.rs displays all required information
4. Test status display in actual UI

### Phase 13.1-13.2 Verification & Enhancement
1. Verify console captures all commands correctly
2. Verify "?" commands are filtered out
3. Verify "ok" responses are filtered out
4. Verify all other responses show
5. Verify tracing output logs to console
6. Verify filter checkboxes work correctly

### Anomaly Detection Removal
1. Remove AnomalyType enum from status_analytics.rs
2. Remove Anomaly struct from status_analytics.rs
3. Remove detect_anomalies() function
4. Remove any UI components showing anomalies
5. Keep position/state/feedrate history (useful for debugging)

### New Implementation: Phase 13.3 (Tracing Integration)
1. Verify tracing output goes to device console
2. Test warning/info/debug message capture
3. Ensure console displays structured tracing data

---

## Implementation Order

1. **Fix compilation warnings**
   - status_manager.rs: Fix unused_must_use

2. **Verify existing implementations**
   - Check status_monitor integration
   - Check device_logger filtering
   - Check console UI display

3. **Enhance device console**
   - Verify all command/response capture
   - Verify filtering works
   - Test with actual device communication

4. **Remove anomaly detection UI**
   - Remove from status_analytics.rs (keep history tracking)
   - Remove from any UI components

5. **Implement tracing to console**
   - Add tracing subscribers that write to device_logger
   - Verify log levels work correctly

---

## Testing Strategy

### Unit Tests
- Device logger filtering (verify "?" and "ok" excluded)
- Status parser (verify correct format parsing)
- Severity filtering (verify each level filters correctly)

### Integration Tests
- Full device communication flow with logging
- Status update pipeline
- Console filtering in UI

### Manual Tests
- Connect to device, run commands, verify console
- Toggle filter checkboxes, verify display changes
- Verify "?" queries don't appear in console
- Verify "ok" responses don't appear in console

---

## Success Criteria

- ✓ No compilation warnings
- ✓ Status displays real-time (position, state, feed, spindle)
- ✓ Device console shows all meaningful device communication
- ✓ Status queries ("?") never appear in console
- ✓ "ok" responses never appear in console
- ✓ All other device responses show in console
- ✓ Tracing output logs to console
- ✓ Severity filters work independently
- ✓ 60 FPS UI rendering maintained
- ✓ No anomaly detection UI elements

