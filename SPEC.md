
gcodekit is a desktop gui application that allows a user to control a laser engraver or CNC machine that uses GRBL to control it. The application includes both machine control and basic CAM (Computer-Aided Manufacturing) functions for generating G-code. The application is to be multiplatform working on linux, windows and MacOSX.

The device should supply the following features:

1. Layout
	a. The application will have status bar that is attached to the bottom of the application window. This will be known as the "Status bar"
	b. The application will have a combined menu and title bar attached to the top of the application window. This will be known as the "Menu Bar"
	c. The application will have a left hand tool pannel dedicated to machine connect and control. all the widgets in this pannel will as wide as the tool pannel and will be stacked ontop of each other. 
	d. The Application will have a right hand tool pannel dedicated to CAM functions, all the widgets in this pannel will be as wide as the tool pannel and will be stacked ontop of each other.
	e. The Application will have a centeral pannel that is tabbed the tabs will run across the top and they will show "gcode editor", "3d Visualizer and "device console"


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
4. Communication module (communication/grbl.rs): Handles all GRBL protocol communication including serial port management, command sending, response parsing, version detection, and real-time status monitoring. 

Technology: Use the Rust language (2024 edition), use cargo build and cargo test for compilation and testing, use egui version 0.33 crate for the GUI interface. Additional dependencies include:
- serialport (4.2) for serial communication
- tokio (1.0) for async runtime
- tracing (0.1) and tracing-subscriber (0.3) for logging
- rfd (0.14) for file dialogs
- anyhow (1.0) for error handling
- serde (1.0) and serde_json (1.0) for serialization
- chrono (0.4) for timestamps

Architecture: Modular design with separate modules for:
- communication: GRBL protocol handling and serial communication
- widgets: Individual UI components for different functions
- main: Application state and UI orchestration

Development Tools:
- cargo clippy: Linting with clippy
- cargo fmt: Code formatting with rustfmt
- cargo check: Fast compilation checking
- cargo test: Run unit tests and integration tests

System Requirements:
- Rust 1.70+ (2024 edition)
- GRBL v1.1+ compatible device

Additional Requirements:
1. GRBL Version Support: Prioritize GRBL v1.1 and v1.2 features including real-time overrides and jogging
2. Device Compatibility: Support only GRBL devices (not other controllers like TinyG or G2core)
3. Menu Structure: Follow Universal G-Code Sender (UGS) menu structure with File, Machine, View, Tools, and Help menus
4. Machine Types: Support both laser engraver and CNC machine commands with automatic mode detection
5. G-code Compatibility: Implement only G-code features supported by GRBL firmware
6. CAM Functions: Include basic Computer-Aided Manufacturing capabilities for generating G-code from shapes and images
7. Version Detection: Capture and display GRBL firmware version on the status bar during connection
8. Code Style: Follow Rust formatting (4 spaces, max 100 width), snake_case naming, structured error handling with anyhow
9. Logging: Use tracing for structured logging, avoid println! in production code
10. Modular Architecture: Separate communication logic from UI components for maintainability
11. Testing: Implement comprehensive unit tests for all components using `cargo test`. Tests should cover core functionality, edge cases, and error conditions. Unit tests must pass as part of the build process and CI/CD pipeline.

References
1. The existing application called "Candle" written in C++ can be found at: https://github.com/Denvi/Candle
2. The firmware for the GRBL controler which interprets the gcode used on the devices.  https://github.com/grbl/grbl 
3. A simular app to Candel written in Java = Universal Gcode Sender https://github.com/winder/Universal-G-Code-Sender

