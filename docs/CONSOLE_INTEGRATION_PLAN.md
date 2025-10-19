# Device Console Integration Plan

## Project Overview

Integrate the device console with device communications to provide comprehensive tracing and logging of all device interactions. The console will display commands sent to the device, filtered responses, and tracing output with configurable verbosity levels.

**Timeline:** 4-5 phases (~15-20 hours)
**Status:** Planning Phase

---

## Phase 13.0 - Architecture & Planning ✅

### Goals
- Define console architecture
- Plan logging strategy
- Design filtering system
- Create data structures

### Deliverables

#### 1. Console Message Types

Define an enum to represent different types of console messages:

```rust
pub enum ConsoleMessageType {
    Command,        // Commands sent to device
    Response,       // Responses from device (excluding "ok")
    Trace(TraceLevel),  // Tracing output
    Info,           // Informational messages
    Error,          // Error messages
    Status,         // Status updates
}

pub enum TraceLevel {
    Debug,
    Info,
    Warning,
    Error,
}
```

#### 2. Console Message Structure

```rust
pub struct ConsoleMessage {
    pub message_type: ConsoleMessageType,
    pub trace_level: Option<TraceLevel>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub source: MessageSource,
    pub raw_data: Option<String>,
}

pub enum MessageSource {
    Device,
    User,
    System,
}
```

#### 3. Console Filtering Configuration

```rust
pub struct ConsoleFilterConfig {
    pub show_commands: bool,           // Show sent commands
    pub show_responses: bool,          // Show device responses
    pub show_ok_responses: bool,       // Show "ok" responses
    pub show_status_queries: bool,     // Show "?" status queries
    pub show_status_responses: bool,   // Show status query responses
    pub trace_filters: TraceFilters,
    pub show_info: bool,
    pub show_debug: bool,
    pub show_warnings: bool,
    pub show_errors: bool,
}

pub struct TraceFilters {
    pub debug_enabled: bool,
    pub info_enabled: bool,
    pub warning_enabled: bool,
    pub error_enabled: bool,
}
```

#### 4. Console State Management

```rust
pub struct ConsoleState {
    pub messages: Vec<ConsoleMessage>,
    pub filter_config: ConsoleFilterConfig,
    pub max_messages: usize,           // Rolling buffer size
    pub auto_scroll: bool,
    pub search_text: String,
    pub selected_message: Option<usize>,
}
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Device Communication                     │
│  (grbl.rs, commands sent, responses received)               │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Sends ConsoleMessage events
                 ▼
┌─────────────────────────────────────────────────────────────┐
│              Console Logger (NEW - Phase 13.1)              │
│  • Filters messages based on configuration                 │
│  • Formats messages with timestamps                        │
│  • Manages message buffer                                  │
│  • Provides filtering interface                            │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Stores ConsoleMessage
                 ▼
┌─────────────────────────────────────────────────────────────┐
│              Console State (Updated MachineState)           │
│  • messages: Vec<ConsoleMessage>                           │
│  • filter_config: ConsoleFilterConfig                      │
│  • max_messages: 2000                                      │
└────────────────┬────────────────────────────────────────────┘
                 │
                 │ Reads for display
                 ▼
┌─────────────────────────────────────────────────────────────┐
│           Console UI (Phase 13.3)                           │
│  • Filter checkboxes                                       │
│  • Message list with syntax highlighting                  │
│  • Search functionality                                    │
│  • Export/Copy functionality                               │
└─────────────────────────────────────────────────────────────┘
```

### Message Flow Examples

#### Example 1: Command Sent
```
User sends: G0 X10 Y20
│
├─ [SENT] G0 X10 Y20
├─ Type: Command
├─ Source: User
└─ Timestamp: 11:34:58
```

#### Example 2: Device Response
```
Device responds: [GC:G0 G54 G17 G21 G90 G94 M5 M9 T0 F0 S0]
│
├─ [RESPONSE] [GC:G0 G54 G17 G21 G90 G94 M5 M9 T0 F0 S0]
├─ Type: Response
├─ Source: Device
└─ "ok" responses are NEVER logged
```

#### Example 3: Status Query (FILTERED)
```
System sends: ?
│
├─ NOT LOGGED (filtered out automatically)
└─ Status response also NOT LOGGED

Device responds: <Idle|MPos:0.000,0.000,0.000|...>
│
└─ NOT LOGGED (filtered out automatically)
```

#### Example 4: Tracing Output
```
[DEBUG] Starting communication thread
[INFO] Connected to /dev/ttyUSB0
[WARNING] Buffer nearly full (95%)
[ERROR] Command timeout after 5s
```

### Key Design Principles

1. **Default Behavior**
   - ✅ Show all user commands
   - ✅ Show all device responses (except "ok")
   - ❌ Hide status queries ("?")
   - ❌ Hide status responses
   - ✅ Show warnings and errors
   - ❌ Hide debug by default
   - ❌ Hide info by default

2. **Filtering Strategy**
   - Filter at source (don't store, then hide)
   - Apply filtering before adding to console
   - Allow runtime filter changes
   - Remember filter preferences

3. **Performance Optimization**
   - Circular buffer (max 2000 messages)
   - Efficient filtering logic
   - UI only re-renders on new messages
   - Minimize memory usage

4. **Usability**
   - Clear visual indicators (labels, colors)
   - Easy filtering with checkboxes
   - Search functionality
   - Copy/export capabilities

---

## Phase 13.1 - Core Console Logger ⏳

### Objectives
- Create console message types and structures
- Implement console logger module
- Add filtering logic
- Integrate with app state

### Implementation Details

#### Files to Create
1. `src/communication/console_logger.rs` (200 LOC)
   - ConsoleMessage, ConsoleMessageType structs
   - ConsoleFilterConfig
   - ConsoleLogger trait
   - DefaultConsoleLogger implementation

#### Files to Modify
1. `src/app/state.rs`
   - Replace `console_messages: Vec<String>` with `console_state: ConsoleState`
   - Update `log_console()` method to use new structure
   - Add filter configuration

2. `src/communication/grbl.rs`
   - Add console logging for commands sent
   - Add console logging for responses received
   - Filter status queries and responses

### Key Functions

```rust
pub trait ConsoleLogger: Send + Sync {
    fn log_command(&mut self, cmd: &str);
    fn log_response(&mut self, response: &str);
    fn log_trace(&mut self, level: TraceLevel, message: &str);
    fn get_filtered_messages(&self) -> Vec<&ConsoleMessage>;
    fn set_filter(&mut self, filter: ConsoleFilterConfig);
}

pub struct DefaultConsoleLogger {
    messages: Vec<ConsoleMessage>,
    filter_config: ConsoleFilterConfig,
    max_messages: usize,
}

impl DefaultConsoleLogger {
    pub fn new(max_messages: usize) -> Self { }
    pub fn add_message(&mut self, msg: ConsoleMessage) { }
    pub fn filter_message(&self, msg: &ConsoleMessage) -> bool { }
    pub fn get_filtered(&self) -> Vec<&ConsoleMessage> { }
    pub fn clear(&mut self) { }
}
```

### Tests to Add
- Filter configuration tests (8 tests)
- Message filtering logic (10 tests)
- Command/response logging (8 tests)
- Buffer management (6 tests)

**Total: 32 new tests**

---

## Phase 13.2 - GRBL Integration ⏳

### Objectives
- Integrate console logging into GRBL communication
- Capture all sent commands
- Capture all device responses
- Filter status queries and responses

### Implementation Details

#### Files to Modify
1. `src/communication/grbl.rs`
   - Add ConsoleLogger instance
   - Log commands in `send_command()`
   - Log responses in message processing
   - Auto-filter "?" and status responses

### Key Changes

```rust
pub struct GrblCommunication {
    // ... existing fields ...
    pub console_logger: Box<dyn ConsoleLogger>,
}

impl GrblCommunication {
    pub fn send_command(&mut self, cmd: &str) -> Result<(), Box<dyn Error>> {
        // ... existing code ...
        
        // Log the command
        self.console_logger.log_command(cmd);
        
        // ... rest of send ...
    }
    
    pub fn process_response(&mut self, response: &str) {
        // Filter "?" queries and status responses
        if response == "?" || response.starts_with("<") {
            // Don't log status queries or responses
            return;
        }
        
        if response == "ok" {
            // Check filter before logging
            if self.console_filter.show_ok_responses {
                self.console_logger.log_response(response);
            }
            return;
        }
        
        // Log other responses
        self.console_logger.log_response(response);
    }
}
```

### Tests to Add
- Integration with GRBL communication (10 tests)
- Command filtering (8 tests)
- Response filtering (8 tests)
- Status query filtering (6 tests)

**Total: 32 new tests**

---

## Phase 13.3 - Console UI Panel ⏳

### Objectives
- Create console filtering UI
- Implement message display with syntax highlighting
- Add filter controls
- Add search and export features

### Implementation Details

#### Files to Create
1. `src/ui/console_panel.rs` (300 LOC)
   - ConsolePanelState
   - Filter checkbox UI
   - Message display with colors
   - Search functionality

#### Files to Modify
1. `src/ui/tabs/device_console.rs` (Major rewrite)
   - Replace simple string display with new console panel
   - Add filter controls
   - Add search UI

### UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Device Console                          [📋 Copy] [🗑️ Clear] │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Filters:                                                    │
│  ☑ Commands    ☑ Responses    ☑ Status    ☑ Warnings       │
│  ☑ Errors      ☐ Info         ☐ Debug                      │
│  ☑ OK Responses ☐ Status Queries                           │
│                                                               │
│  Search: [___________]  [X]                                 │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  [11:34:58] [COMMAND] G0 X10 Y20                             │
│  [11:34:59] [RESPONSE] ok                                    │
│  [11:35:00] [TRACE:WARNING] Buffer nearly full               │
│  [11:35:01] [COMMAND] G1 Z-5 F100                            │
│  [11:35:02] [RESPONSE] [GC:G1 G54 G17 G21 G90 ...]          │
│                                                               │
│  ▼ (more messages)                                           │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Color Scheme

```rust
pub const CONSOLE_COLORS: &[(&str, Color32)] = &[
    ("COMMAND",  Color32::from_rgb(100, 200, 255)), // Blue
    ("RESPONSE", Color32::from_rgb(100, 255, 150)), // Green
    ("TRACE:DEBUG",   Color32::from_rgb(150, 150, 150)), // Gray
    ("TRACE:INFO",    Color32::from_rgb(200, 200, 100)), // Yellow
    ("TRACE:WARNING", Color32::from_rgb(255, 150, 0)),   // Orange
    ("TRACE:ERROR",   Color32::from_rgb(255, 100, 100)), // Red
    ("STATUS",  Color32::from_rgb(200, 100, 255)), // Purple
];
```

### UI Components

```rust
pub struct ConsolePanelState {
    pub show_commands: bool,
    pub show_responses: bool,
    pub show_ok: bool,
    pub show_status: bool,
    pub show_warnings: bool,
    pub show_errors: bool,
    pub show_info: bool,
    pub show_debug: bool,
    pub search_text: String,
    pub auto_scroll: bool,
}

pub fn show_console_filters(ui: &mut egui::Ui, state: &mut ConsolePanelState) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.show_commands, "Commands");
        ui.checkbox(&mut state.show_responses, "Responses");
        ui.checkbox(&mut state.show_ok, "OK");
        ui.checkbox(&mut state.show_status, "Status");
        ui.checkbox(&mut state.show_warnings, "⚠️ Warnings");
        ui.checkbox(&mut state.show_errors, "❌ Errors");
        ui.checkbox(&mut state.show_info, "ℹ️ Info");
        ui.checkbox(&mut state.show_debug, "🐛 Debug");
    });
}
```

### Tests to Add
- Filter checkbox state (6 tests)
- Message display logic (8 tests)
- Color assignment (6 tests)
- Search functionality (8 tests)

**Total: 28 new tests**

---

## Phase 13.4 - Tracing Integration ⏳

### Objectives
- Integrate Rust tracing library
- Capture tracing output to console
- Add debug level controls
- Performance optimization

### Implementation Details

#### Files to Create
1. `src/communication/console_tracing.rs` (150 LOC)
   - Custom tracing subscriber
   - Console logging layer
   - Tracing configuration

#### Files to Modify
1. `src/main.rs` or `src/lib.rs`
   - Initialize custom tracing subscriber
   - Route tracing output to console logger

### Implementation

```rust
pub struct ConsoleTracingLayer {
    console_logger: Arc<Mutex<Box<dyn ConsoleLogger>>>,
}

impl<S> Layer<S> for ConsoleTracingLayer 
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = metadata.level();
        
        let trace_level = match *level {
            Level::DEBUG => TraceLevel::Debug,
            Level::INFO => TraceLevel::Info,
            Level::WARN => TraceLevel::Warning,
            Level::ERROR => TraceLevel::Error,
            Level::TRACE => TraceLevel::Debug,
        };
        
        // Get message from event
        let message = format!("{:?}", event);
        
        // Log to console
        if let Ok(mut logger) = self.console_logger.lock() {
            logger.log_trace(trace_level, &message);
        }
    }
}
```

### Tests to Add
- Tracing level filtering (8 tests)
- Trace event capture (10 tests)
- Performance under load (6 tests)

**Total: 24 new tests**

---

## Phase 13.5 - Polish & Final Integration ⏳

### Objectives
- Finalize UI/UX
- Add export functionality
- Performance optimization
- Comprehensive testing

### Implementation Details

#### Features to Add
1. **Export Functions**
   - Export to file (.txt, .csv, .json)
   - Copy selected messages
   - Copy all messages

2. **Advanced Filtering**
   - Filter by time range
   - Filter by message type
   - Regex search
   - Save filter presets

3. **Performance Optimization**
   - Lazy rendering
   - Virtual scrolling for large lists
   - Message deduplication options

4. **UI/UX Improvements**
   - Collapsible filter panel
   - Message details view
   - Timestamp display options
   - Theme support for console colors

### Files to Modify
1. `src/ui/console_panel.rs` (Enhancements)
2. `src/ui/tabs/device_console.rs` (Enhancements)
3. `src/app/state.rs` (Add export methods)

### Tests to Add
- Export functionality (10 tests)
- Advanced filtering (12 tests)
- Performance metrics (8 tests)

**Total: 30 new tests**

---

## Implementation Checklist

### Phase 13.0 ✅
- [x] Architecture design
- [x] Data structure planning
- [x] Filtering strategy
- [x] UI mockups

### Phase 13.1 ⏳
- [ ] ConsoleMessage types
- [ ] ConsoleFilterConfig
- [ ] ConsoleLogger trait
- [ ] DefaultConsoleLogger impl
- [ ] Tests (32)

### Phase 13.2 ⏳
- [ ] GRBL integration
- [ ] Command logging
- [ ] Response logging
- [ ] Status query filtering
- [ ] Tests (32)

### Phase 13.3 ⏳
- [ ] ConsolePanelState
- [ ] Filter UI
- [ ] Message display
- [ ] Search functionality
- [ ] Tests (28)

### Phase 13.4 ⏳
- [ ] Tracing integration
- [ ] TraceLevel support
- [ ] Trace event capture
- [ ] Tests (24)

### Phase 13.5 ⏳
- [ ] Export functionality
- [ ] Advanced filtering
- [ ] Performance optimization
- [ ] UI/UX polish
- [ ] Tests (30)

---

## Summary

**Total Implementation:**
- **5 Phases** over **4-5 weeks**
- **1 developer** (~15-20 hours)
- **~1,200 LOC** production code
- **~144 new tests**
- **5 files created**
- **5 files modified**

**Key Deliverables:**
1. ✅ Comprehensive console logging system
2. ✅ Multi-level filtering (command, response, trace)
3. ✅ Integrated tracing support
4. ✅ Professional console UI
5. ✅ Export and search capabilities

**Architecture Advantages:**
- Clean separation of concerns
- Extensible design
- Thread-safe implementation
- Performance optimized
- Well-tested codebase
- Easy to maintain

---

## References

- GRBL Communication: `src/communication/grbl.rs`
- Current Console UI: `src/ui/tabs/device_console.rs`
- App State: `src/app/state.rs`
- UI Framework: egui 0.33+
- Async Runtime: Tokio
- Tracing: `tracing` crate

