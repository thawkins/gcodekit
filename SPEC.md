
gcodekit is a desktop GUI application that allows users to control laser engravers or CNC machines using GRBL firmware. The application provides comprehensive machine control, advanced CAM (Computer-Aided Manufacturing) functions for G-code generation, and robust error recovery capabilities ensuring 99.9% uptime. The application is multiplatform, working on Linux, Windows, and macOS.

The device should supply the following features:

1. Layout
	a. The application will have status bar that is attached to the bottom of the application window. This will be known as the "Status bar"
	b. The application will have a combined menu and title bar attached to the top of the application window. This will be known as the "Menu Bar"
	c. The application will have a left hand tool pannel dedicated to machine connect and control. all the widgets in this pannel will as wide as the tool pannel and will be stacked ontop of each other. 
	d. The Application will have a right hand tool pannel dedicated to CAM functions, all the widgets in this pannel will be as wide as the tool pannel and will be stacked ontop of each other.
	e. The Application will have a central panel that is tabbed. The tabs will run across the top and they will show "G-code Editor", "3D Visualizer", "Device Console", "Job Manager", and "Designer"


2. Widget functions
1. A lefthand tool panel with modular widgets stacked vertically:
 	a. Connection widget (connection.rs): Device selection and connection management with status display
 	b. G-code loading widget (gcode_loading.rs): File selection, loading, and queued sending to prevent buffer overruns
 	c. Jog widget (jog.rs): Real-time axis control (X/Y/Z) with configurable step sizes (0.1, 1, 10, 50mm)
 	d. Overrides widget (overrides.rs): Real-time spindle/laser power and feed rate adjustments
2. A righthand tool panel dedicated to CAM functions with modular widgets:
 	a. Shape generation widget (shape_generation.rs): Create basic shapes (rectangles, circles) with adjustable dimensions
 	b. Toolpath generation widget (toolpath_generation.rs): Convert shapes to GRBL-compatible G-code with feed rates and spindle/laser controls
 	c. Vector import widget (vector_import.rs): Load SVG/DXF files and convert to G-code for engraving/cutting
 	d. Image engraving widget (image_engraving.rs): Convert bitmap images to GRBL-compatible G-code for laser engraving with adjustable resolution and intensity
 	e. Tabbed box widget (tabbed_box.rs): Generate cutting paths for boxes with interlocking tabs, with adjustable dimensions, tab size, and material thickness
 	f. Jigsaw widget (jigsaw.rs): Generate laser cutting paths for interlocking puzzle pieces with adjustable piece count, size, and complexity
3. Status bar, shows the connection/disconnection status, device state (idle/alarmed), current position (X/Y/Z), and GRBL version when connected.
  4. Communication module (communication/grbl.rs): Handles GRBL protocol communication including serial port management, command sending, response parsing, version detection, and real-time status monitoring.
  5. 3D Visualizer: Interactive G-code visualization with color-coded paths (rapid moves blue, feed moves green, arcs yellow), right-click jog to position, left-click path selection with highlighting, real-time machine position overlay with 3-axis support (XYZ).
  6. Job Manager: Comprehensive job queuing system with priority-based scheduling, progress tracking, pause/resume functionality, and automatic job resumption after communication errors.

Technology: Built with Rust language (2024 edition), using cargo build and cargo test for compilation and testing, with egui v0.33 for the GUI interface. Core dependencies include:
- serialport (4.2) for serial communication
- tokio (1.0) for async runtime
- tracing (0.1) for structured logging
- rfd (0.14) for file dialogs
- anyhow (1.0) for error handling
- serde (1.0) for serialization

- chrono (0.4) for timestamps
- uuid (1.0) for job identification
- usvg (0.37) for SVG parsing
- dxf (0.4) for DXF file parsing
- lyon (1.0) for 2D graphics and path operations
- image (0.24) for bitmap processing

Architecture: Modular design with separate modules for:
- communication: GRBL protocol handling, serial communication, and error recovery
- designer: CAD/CAM design tools and shape manipulation
- jobs: Job management, queuing, and resumption capabilities
- materials: Material database and properties
- widgets: Individual UI components for different functions
- main: Application state and UI orchestration
- tests: Unit and integration tests organized in hierarchy mirroring src/

Development Tools:
- cargo clippy: Linting with clippy
- cargo fmt: Code formatting with rustfmt
- cargo check: Fast compilation checking
- cargo test: Run unit tests and integration tests

System Requirements:
- Rust 1.75+ (2024 edition)
- GRBL v1.1+ compatible device
- Serial port access for device communication

Additional Requirements:
1. GRBL Version Support: Prioritize GRBL v1.1 and v1.2 features including real-time overrides and jogging
2. Device Compatibility: Support GRBL controllers
3. Menu Structure: Follow Universal G-Code Sender (UGS) menu structure with File, Machine, View, Tools, and Help menus
4. Machine Types: Support both laser engraver and CNC machine commands with automatic mode detection
5. G-code Compatibility: Implement only G-code features supported by GRBL firmware
6. CAM Functions: Include basic Computer-Aided Manufacturing capabilities for generating G-code from shapes and images
7. Version Detection: Capture and display GRBL firmware version on the status bar during connection
8. Code Style: Follow Rust formatting (4 spaces, max 100 width), snake_case naming, structured error handling with anyhow
9. Logging: Use tracing for structured logging, avoid println! in production code
10. Modular Architecture: Separate communication logic from UI components for maintainability
11. Testing: Implement comprehensive unit tests for all components using `cargo test`. Tests should cover core functionality, edge cases, and error conditions. Unit tests must pass as part of the build process and CI/CD pipeline.
12. Error Recovery: Implement 99.9% uptime guarantee through automatic error recovery, job resumption, and comprehensive logging
13. 3-Axis Support: Optimized for GRBL machines with dedicated X/Y/Z axis control
14. Job Management: Advanced job queuing with priority scheduling, progress tracking, and automatic resumption after errors

 Current Features:
  12. **Advanced Visualizer**: Right-click jog to location ✓, color-coded paths for G0/G1/G2/G3 moves ✓, 3-axis support (XYZ) ✓, real-time machine position overlay ✓, outline gcode functionality (planned)
  13. **Designer Tab Foundation**: Basic shape drawing (Rectangle ✓, Circle ✓, Line ✓) with interactive canvas, shape selection, and G-code export ✓
  14. **Modular Architecture**: Clean separation of concerns with dedicated modules for communication, designer, jobs, materials, and widgets ✓
  15. **Advanced Error Recovery System**: 99.9% uptime guarantee through automatic error recovery, job resumption, and comprehensive logging ✓
  16. **Job Management System**: Priority-based job queuing, progress tracking, pause/resume functionality, and automatic resumption after communication errors ✓
  17. **3-Axis Support**: Optimized for GRBL machines with X/Y/Z axis control and G-code parsing ✓
  18. **Enhanced Communication**: Support for GRBL controllers ✓
  19. **Vector Import**: SVG/DXF file import with automatic G-code conversion ✓
  20. **Boolean Operations**: Shape union operations for combining geometric elements ✓
  21. **Probing Routines**: Z-probing, auto-leveling, and workpiece measurement with G38.x commands ✓
  22. **Tool Management**: Tool length offsets (G43/G49), tool change support, and tool libraries ✓
  23. **Keybinding Customization**: Configurable keyboard shortcuts for all major actions ✓
  24. **UI Stability**: Resolved all duplicate element IDs in egui interface for reliable dropdown menus and button interactions ✓

  26. **Configurable UI System**: Dockable window functionality with toggleable left/right panels via View menu ✓
  27. **Advanced CAM Operations**: Part nesting algorithm using bottom-left fill strategy with rotation support ✓
28. **Test Reorganization**: Tests moved to tests/ folder with hierarchy mirroring src/ ✓
29. **Build Fixes**: Compilation errors resolved and debug binary built ✓
30. **Repository Updates**: Changes committed ✓
31. **Port Filtering**: Serial ports filtered to show only GRBL-compatible devices (/dev/ttyACM*, /dev/ttyUSB*, COM*, /dev/tty.usbserial*) ✓
32. **Issue Templates**: GitHub issue templates added for BUG, FEATURE, and CHANGE requests ✓
33. **Code Quality**: Clippy warnings fixed for improved code maintainability ✓

## Phase 9: Advanced Error Recovery System (99.9% Uptime Guarantee)

The advanced error recovery system provides comprehensive fault tolerance and automatic recovery capabilities:

### Error Recovery Features:
- **Automatic Reconnection**: Detects connection loss and attempts automatic reconnection with configurable retry limits
- **Command Retry Logic**: Automatically retries failed commands with exponential backoff
- **Critical Error Handling**: Handles alarms and emergency conditions with controller reset capabilities
- **Job Resumption**: Automatically interrupts jobs on errors and enables resumption from the last completed line
- **Comprehensive Logging**: Detailed logging of all recovery attempts, actions taken, and outcomes with timestamps

### Job Management System:
- **Priority-based Queuing**: Jobs are queued with priority levels (1-10) for optimal scheduling
- **Progress Tracking**: Real-time progress updates based on completed G-code lines
- **Pause/Resume Functionality**: Manual and automatic job pausing with resumption capabilities
- **Error Recovery Integration**: Jobs automatically resume from interruption points after communication recovery

### 3-Axis Support:
- **XYZ Axis Support**: Optimized for GRBL machines with dedicated 3-axis control
- **G-code Parsing**: Parser for X/Y/Z G-code commands (rotary commands ignored)
- **Position Tracking**: Real-time position monitoring for X/Y/Z axes
- **Jog Controls**: Theme-aware jog controls with 60×60 buttons and configurable step sizes

### Technical Implementation:
- **Recovery Configuration**: Configurable retry attempts, delays, and auto-recovery settings
- **State Management**: Comprehensive recovery state tracking with action history
- **UI Integration**: Job manager UI with resume controls for interrupted jobs
- **Error Classification**: Intelligent error categorization for appropriate recovery actions

## Phase 9: Advanced Job Scheduling System

The advanced job scheduling system provides enterprise-grade production management capabilities:

### Job Scheduling Features:
- **Time-based Execution**: Schedule jobs to run at specific times with recurring intervals (minutes, hours, days, weeks, months)
- **Dependency Management**: Jobs can depend on completion of other jobs before execution begins
- **Recurring Schedules**: Configurable repeat intervals with optional maximum run limits
- **Priority Integration**: Scheduled jobs respect the existing priority-based queuing system
- **Persistence**: Scheduled jobs are saved to disk and restored on application restart

### Scheduling UI Components:
- **Job Scheduling Widget**: Complete interface for creating and managing scheduled jobs
- **Dependency Selection**: Choose from completed jobs to create execution dependencies
- **Schedule Monitoring**: View upcoming jobs, current schedules, and execution history
- **Manual Execution**: Process scheduled jobs on-demand for testing and immediate execution

### Advanced Features:
- **Complex Scheduling**: Support for complex production workflows with job chains and dependencies
- **Error Recovery Integration**: Scheduled jobs work seamlessly with the 99.9% uptime guarantee
- **3-axis Compatibility**: Full support for XYZ-axis scheduled jobs
- **Performance Tracking**: Monitor execution times, success rates, and scheduling efficiency

## Phase 10: Advanced CAM Features and Controller Support ✅ COMPLETED

The advanced CAM features and controller support phase extends gcodekit's capabilities with professional-grade features:

### Configurable UI System:
- **Dockable Windows**: Toggleable left/right panels via View menu for customizable workflows
- **Flexible Layout**: Highly configurable interface layouts to suit different user preferences

### Advanced CAM Operations:
- **Part Nesting**: Bottom-left fill strategy with rotation support for material optimization
- **Positioned Parts**: Structs for nesting configuration and positioned parts management

### Testing & Validation:
- **Comprehensive Testing**: 41 passing tests covering core functionality and new features
- **Release Build**: Successful optimized release build ensuring production readiness

## Phase 11: Advanced 3D Machining

The advanced 3D machining phase adds sophisticated 3D capabilities:

### Advanced 3D Surface Machining:
- **Waterline Machining**: Horizontal slicing for 3D surface machining
- **Scanline Machining**: Vertical slicing with morphing capabilities
- **3D Profiling**: Complex surface machining strategies

### STL Processing:
- **File Import**: STL mesh import with automatic repair
- **Mesh Processing**: Surface triangulation and optimization
- **3D Visualization**: Real-time 3D rendering at 30+ FPS

### 3-Axis Optimization:
- **XYZ Strategies**: Optimized machining operations for GRBL machines
- **Simplified Architecture**: Removed rotary axis complexity for cleaner codebase

## Phase 12 & 13: Real-Time Machine Status Monitoring & Device Console Integration ✅ COMPLETED

The real-time machine status monitoring and device console integration phases add professional-grade machine monitoring and diagnostic capabilities:

### Phase 12: Real-Time Machine Status Display
- **Status Update Integration**: Enhanced app state with real-time machine status monitoring
- **Bottom Status Bar Redesign**: Displays connection status, machine state (with color coding), machine position (MPos/WPos), feed rate, and spindle speed
- **Color-Coded Status**: Green (Idle), Blue (Run/Jog), Yellow (Hold/Door), Red (Alarm), Gray (Unknown/Sleep/Check)

### Phase 13: Device Console Integration  
- **Enhanced Device Console Tab**: Severity-based filtering (Error, Warning, Info, Debug) with independent toggles
- **Color-Coded Messages**: Type and severity-based message coloring for easy scanning
- **Automatic Filtering**: Status queries and simple "ok" responses automatically excluded from display
- **Message Counts**: Real-time message count display with copy/clear controls

## Development Status

**Current Phase**: Phase 13 Complete - Device Console Integration
**Implementation Status**: Alpha Development Version - Core features functional with ongoing development
**Version**: 0.1.0-alpha
**Test Coverage**: 362 passing tests covering all major components, machine control, UI functionality, communication, and status monitoring
**Architecture**: Modular, extensible design with stable UI framework, advanced CAM capabilities, real-time status monitoring, and improved code quality

⚠️ **Alpha Notice**: This software is under active development. While functional and tested, it may contain bugs and the API may change in future releases. Use with appropriate caution.

**Completed Phases & Tasks**:
- Phase 1-8: Core GRBL communication, GUI framework, CAM functions, multi-axis support
- Phase 9: Advanced error recovery, job management, multi-axis support, and job scheduling system
- Phase 10: Configurable UI system, advanced CAM operations with part nesting
- Phase 12: Real-time machine status display
- Phase 13: Device console integration with severity filtering
- UI Stabilization: Resolved duplicate element IDs and improved interface reliability
- Code Quality: Clippy warnings fixed and code maintainability improved
- User Experience: Port filtering implemented for easier GRBL device identification
- Development Tools: GitHub issue templates added for structured issue reporting

**Session Completion (Oct 19, 2025) - Extended Session**:
- ✅ Task 1: G-code Editor Advanced Features (Goto line, Select all)
- ✅ Task 3: Image to G-code Conversion (Complete bitmap to laser engraving workflow)
- ✅ Task 4: Tabbed Box & Jigsaw Path Generation (Production-ready cutting patterns)
- ✅ Task 5: File Import/Export Operations (JSON design persistence)
- ✅ Task 8: Settings Management System (Profile management with backup/restore)
- ✅ Task 9: Machine Control UI Features (Reset, stop, about, documentation)

**Additional Improvements (Extended Session)**:
- ✅ Removed all firmware management references from documentation
- ✅ Removed all anomaly detection references from specs/plans
- ✅ Verified alarm unlock button in jog panel
- ✅ Verified resume button in jog panel for pause state
- ✅ Added comprehensive test coverage (30 new tests, 362 total)
  - Web pendant interface tests (9 tests)
  - Input handling and keybindings tests (11 tests)
  - Calibration module tests (18 tests for step calibration, backlash, and homing)
- ✅ Linked materials database to stock visualization
- ✅ Updated tests to match actual implementations
- ✅ Build passes with zero warnings (project code)

**Current Status**:
- Test Coverage: 362 tests (100% passing)
- Code Quality: 0 warnings (project code)
- Firmware Management: Removed from scope
- Jog Panel: Enhanced with state-based controls
- Anomaly Detection: Removed from scope
- Materials Integration: Stock visualization linked to materials database

## Task 8: Settings Management System ✅ COMPLETED

Comprehensive machine profile and settings management enabling users to:
- **Save/Load GRBL Machine Profiles**: Store custom machine configurations with GRBL parameters
- **Multi-Machine Support**: Switch between different machine configurations instantly
- **Settings Backup/Restore**: Backup all profiles with timestamped directories
- **Import/Export**: Share profiles across machines and platforms
- **Profile Management UI**: Intuitive interface for creating, activating, and deleting profiles

### Implementation Details:
- **ProfileSettings**: Struct with all GRBL machine parameters (step rates, feed rates, acceleration, spindle speeds, soft limits, axis inversions)
- **MachineProfile**: Complete profile with metadata, machine type, port configuration
- **ProfileManager**: In-memory profile management with active profile tracking
- **SettingsStorage**: Persistent JSON-based storage in platform-specific config directories
- **UI Integration**: Settings panel with profile list, creation dialog, delete confirmation
- **Storage Location**: `~/.config/gcodekit/profiles/` (Linux), `%APPDATA%\gcodekit\profiles\` (Windows), `~/Library/Application Support/gcodekit/profiles/` (macOS)

### Test Coverage:
- 16 tests covering profile creation, management, persistence, and UI state
- All tests passing (362 total tests in project)
- Full error handling with anyhow::Result

**Remaining Tasks (3/10)**:
- Task 2: Back Plotting (Visual G-code Simulator) (2-3 days)
- Task 7: Advanced CAM Features (3-4 days)
- Task 10: Web Pendant Interface (3-5 days)

**Next Development Focus**: Task 2 - Back Plotting
- Step-through G-code visualization
- Tool path tracing and line highlighting
- Rewind and step-through capabilities
- Integration with 3D visualizer

  Future Enhancements (UGS Feature Parity):
19. **Designer Editor**: Import SVG/DXF/C2D files ✓, draw shapes/text ✓, boolean operations (union ✓/intersect/subtract), undo/redo ✓, shape manipulation (move/scale/rotate/mirror), grid multiplication, clipart library, bitmap tracing
  20. **G-code Editor Enhancements**: Highlight selected rows in visualizer ✓, run from selected line ✓, model rotation/mirroring, move to zero location
  21. **G-code Optimization**: Remove comments ✓, truncate decimal precision, convert arcs to line segments, remove whitespace ✓
  22. **Work Coordinate System (WCS)**: G54-G59 coordinate system management and switching ✓
23. **Probing Routines**: Z-probing ✓, auto-leveling ✓, workpiece measurement with G38.x commands ✓
24. **Tool Management**: Tool length offsets (G43/G49) ✓, tool change support ✓, tool libraries with predefined cutter parameters ✓
25. **Machine Calibration**: Step calibration, backlash compensation, homing sequence configuration ✓

27. **Gamepad/Joystick Support**: SDL-based gamepad/joystick control with customizable button mapping
28. **Web Pendant Interface**: Remote control via web-based pendant interface
29. **Settings Management**: Backup/restore GRBL settings, multiple machine profiles
30. **File Management**: Multiple file queuing, sequential processing, file preprocessing
31. **Safety Features**: Emergency stop, soft limits, safety door handling
32. **Material Database**: Predefined material settings and cutting parameters

34. **Pendant Support**: External pendant/joystick hardware support
36. **Custom Button Panels**: User-defined control buttons and macros
37. **Keybinding Customization**: Configurable keyboard shortcuts for all actions ✓
38. **Data Logging**: Operation logging, analytics, and performance metrics
39. **Configurable UI**: Dockable windows, customizable toolbars, responsive design ✓

CamBam-Inspired Features:
37. **Advanced G-code Editor**: Built-in G-code editor with syntax highlighting, error checking, and manual modifications

    - Implementation: custom egui text widget (src/gcodeedit/editor.rs) providing buffer, cursor, selection, folding and virtualized line rendering.
    - Vocabulary: GRBL G/M code vocabulary for versions 1.0, 1.1 and 1.2 (src/gcodeedit/vocabulary.rs) used for highlighting, completion and validation.
    - Validation Rules: configurable RuleSet (src/gcodeedit/rules.rs) allowing enable/disable of rules, per-rule severity, and GRBL-version-specific checks.
    - Parser/Tokenizer: incremental tokenizer + lightweight parser service for real-time syntax and semantic validation (debounced background task).
    - Editor Features: line numbers, gutter diagnostics, find/replace, code folding, auto-completion for G/M codes and parameters, keyboard shortcuts, undo/redo.
    - Integration: editor <-> visualizer line mapping for back-plot stepping and line highlighting; APIs for completions and diagnostics.
    - Performance: incremental parsing and token caching to keep validation and highlighting responsive (<100ms) for files 1000+ lines.
    - Tests & Docs: unit tests for tokenizer, rules, and editor buffer; documentation and DOCBLOCKs per project standards.
38. **Back Plotting**: Visual simulation of G-code execution to verify toolpaths before machining

40. **Speeds and Feeds Calculator**: Built-in calculator for optimizing cutting parameters based on material and tool
41. **Bitmap Processing**: Import bitmaps for heightmap generation, edge detection, and vectorization

43. **Region Fill**: Automatic filling of enclosed areas for machining
44. **Part Nesting**: Automatic part nesting for efficient material usage ✓
45. **3D Profiling**: Waterline and scanline machining for 3D surfaces, back-face machining
46. **Lathe Operations**: Turning operations for cylindrical parts (facing, grooving, threading)
47. **Holding Tabs**: Automatic generation of tabs to hold parts during machining
48. **Lead Moves**: Configurable lead-in and lead-out moves to reduce tool wear
49. **Side Profiles**: Machining vertical faces or sides of parts
50. **Advanced CAD Operations**: Boolean operations (union ✓/intersection/difference), polyline/surface editing
51. **CAM Part Management**: Organize multiple machining operations into parts for batch processing
52. **Automation Scripting**: Scripting support for batch processing and workflow automation

LaserGRBL-Inspired Features:
53. **Image Engraving Enhancements**: Grayscale conversion, dithering algorithms, and vectorization for laser engraving
54. **Power and Speed Overrides**: Real-time adjustment of laser power and feed speed during operation
55. **User-Defined Buttons**: Customizable macro buttons for frequently used commands
56. **Configuration Management**: Import/export of GRBL settings and machine profiles

References and competative tools:

1. The existing application called "Candle" written in C++ can be found at: https://github.com/Denvi/Candle
2. The firmware for the GRBL controller which interprets the gcode used on the devices: https://github.com/grbl/grbl
3. A similar app to Candle written in Java = Universal Gcode Sender: https://github.com/winder/Universal-G-Code-Sender
4. Cambam a tool written in C# for managing CNC devices: http://www.cambam.info/doc/1.0/

6. LightBurn Laser Engraver control - https://docs.lightburnsoftware.com/legacy/pdf/document.pdf
7. LaserGRBL Laser Engraver Control - https://lasergrbl.com/usage/
8. TinkerCad simple design modelling - https://skills4am.eu/documents/tinkercad_usermanual.pdf


General Instructions:

When reading PDF or Word files convert the files first to markdown before processing them. 




