
gcodekit is a desktop gui application that allows a user to control a laser engraver or CNC machine that uses GRBL to control it. The application includes both machine control and basic CAM (Computer-Aided Manufacturing) functions for generating G-code. The application is to be multiplatform working on linux, windows and MacOSX.

The device should supply the following features:

1. Layout
	a. The application will have status bar that is attached to the bottom of the application window. This will be known as the "Status bar"
	b. The application will have a combined menu and title bar attached to the top of the application window. This will be known as the "Menu Bar"
	c. The application will have a left hand tool pannel dedicated to machine connect and control. all the widgets in this pannel will as wide as the tool pannel and will be stacked ontop of each other. 
	d. The Application will have a right hand tool pannel dedicated to CAM functions, all the widgets in this pannel will be as wide as the tool pannel and will be stacked ontop of each other.
	e. The Application will have a centeral pannel that is tabbed the tabs will run across the top and they will show "gcode editor", "3d Visualizer and "device console"


2. Widget functions
1. A lefthand tool panel that allows has a number of "Widgets: stacked inside of it that do the following: 
	a. a "Connection" widget that allows the user to select a device and connect to it, the connection status will be shown in a status bar anchored to the bottomr. 	 
	b. a gcode selection and loading widget, it will allow the user to select a gcode file. load into the gcode editor widget, and send it to the device if one is connected, all text sent to the device will be queued so we dont get character overruns or underruns. 
	c. a Jog widget, this widjet allows the user to control the device, moving it in each axis X.Y and Z. by pressing left/right, forward/bacl and updown buttins. the pannel maintains a drop down of mm values (0.1. 1, 10, 50) that are the stepsize used for each job.
	d. an overrides pannel, this pannel allows spindle speed and feed rate to be altered in real time, for a laser engraver, the spindle speed is the laser intensity.
2. A righthand tool panel dedicated to CAM functions with the following widgets:
	a. Shape generation widget: create basic shapes (rectangles, circles) with adjustable dimensions
	b. Toolpath generation widget: convert shapes to GRBL-compatible G-code with feed rates and spindle/laser controls
	c. Vector import widget: load SVG/DXF files and convert to G-code for engraving/cutting
	d. Image engraving widget: convert bitmap images to GRBL-compatible G-code for laser engraving with adjustable resolution and intensity
	e. Tabbed box widget: generate cutting paths for boxes with interlocking tabs, with adjustable dimensions, tab size, and material thickness
	f. Jigsaw widget: generate laser cutting paths for interlocking puzzle pieces with adjustable piece count, size, and complexity
3. Status bar, shows the connection/disconection status, wether the devic is locked, {alarmed} and if the device is connected, its current position and GRBL version. 

Technology: Use the Rust language, use cargo build and cargo test, use egui version 0.33 crate for the GUI interface.

Additional Requirements:
1. GRBL Version Support: Prioritize GRBL v1.1 and v1.2 features including real-time overrides and jogging
2. Device Compatibility: Support only GRBL devices (not other controllers like TinyG or G2core)
3. Menu Structure: Follow Universal G-Code Sender (UGS) menu structure with File, Machine, View, Tools, and Help menus
4. Machine Types: Support both laser engraver and CNC machine commands with automatic mode detection
5. G-code Compatibility: Implement only G-code features supported by GRBL firmware
6. CAM Functions: Include basic Computer-Aided Manufacturing capabilities for generating G-code from shapes and images
7. Version Detection: Capture and display GRBL firmware version on the status bar during connection

References
1. The existing application called "Candle" written in C++ can be found at: https://github.com/Denvi/Candle
2. The firmware for the GRBL controler which interprets the gcode used on the devices.  https://github.com/grbl/grbl 
3. A simular app to Candel written in Java = Universal Gcode Sender https://github.com/winder/Universal-G-Code-Sender

