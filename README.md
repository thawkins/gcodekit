# gcodekit - GRBL Controller

A desktop GUI application for controlling GRBL-based laser engravers and CNC machines, built with Rust and egui.

## Features

- **Machine Control**: Connect to and control GRBL devices via serial communication
- **CAM Functions**: Generate G-code from shapes, images, and custom designs
- **G-code Editor**: View and edit G-code files with syntax highlighting
- **3D Visualization**: Preview toolpaths (framework ready for enhancement)
- **Real-time Control**: Jog controls, spindle/laser overrides, and status monitoring
- **Cross-platform**: Works on Linux, Windows, and MacOSX

## Requirements

- Rust 1.70+
- GRBL v1.1+ compatible device

## Building

```bash
cargo build --release
```

## Usage

Connect your GRBL device and use the intuitive GUI to:
- Load and send G-code files
- Control machine movement with jog controls
- Generate G-code from shapes and images
- Monitor device status and communication

## License

MIT License
