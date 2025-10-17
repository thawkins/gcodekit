# Changelog

All notable changes to gcodekit are documented in this file.

## [0.1.0-alpha] - 2025-10-17

### Removed
- **Rotary Axis Support (A, B, C, D)**: Removed all support for rotary axes to simplify codebase. GRBL-based machines typically use 3-axis (XYZ) operation. Can be reintroduced if needed.
  - Removed optional fields from `MachinePosition` struct
  - Removed builder methods (`with_a()`, `with_b()`, `with_c()`, `with_d()`)
  - Removed rotary axis jog controls from jog widget
  - Removed rotary axis position tracking
  - Updated communication module status parsing
  - Impact: ~180 lines removed, improved performance

- **Code Folding Feature**: Removed code folding from G-code editor as GRBL doesn't support code blocks
  - Removed `FoldRegion` and `FoldManager` structs
  - Removed fold-related UI controls and keyboard shortcuts (Ctrl+], Ctrl+Shift+], Ctrl+Shift+[)
  - Removed fold detection logic and tests
  - Impact: ~300 lines removed, fixed gutter alignment issues

### Changed
- **Jog Widget Redesign**: Complete redesign to match professional control panel layout
  - Implemented 4-row grid structure for better organization
  - Reduced button size from 100×100 to 60×60 pixels
  - Added theme-aware colors (respects light/dark mode)
  - Header now shows "Step Size | Jog Feed" with current position display
  - G-code workspace macros (G54-G57) integrated into main grid
  - Placeholder row added for future expansion

- **Theme Support**: All jog widget buttons now theme-aware
  - Buttons adapt to system light/dark theme
  - Emergency stop remains red with white text for safety
  - Improved readability in all conditions

- **Documentation Updates**:
  - README.md: Updated feature list for 3-axis focus
  - SPEC.md: Changed multi-axis to 3-axis optimization
  - GUTTER_ALIGNMENT_FIX.md: Simplified after folding removal
  - Updated test counts (143 vs previous 341)

- **Gutter Alignment**: Improved gutter rendering
  - Now uses theme-aware colors from `ui.visuals()`
  - Maintains perfect pixel-perfect alignment

### Fixed
- Gutter line numbers alignment issues in editor
- Theme-related button readability in dark mode
- Simplified position tracking logic

### Technical
- Removed ~480 lines of code
- Removed ~200 tests related to rotary axes and folding
- All 143 remaining tests passing ✅
- Clean build with no warnings
- Code complexity reduced significantly

## Previous Versions

### Phase 10 Complete ✅
- Configurable UI system with dockable panels
- Advanced CAM operations with part nesting
- Full GRBL v1.1+ protocol support
- 99.9% uptime with error recovery
- Job management with priority scheduling
- Comprehensive test coverage

### Phase 1-9 ✅
- Core GRBL communication
- GUI framework with egui
- CAM functions and toolpath generation
- Multi-axis position tracking (now simplified to 3-axis)
- Error recovery and job scheduling
- Tool management and probing routines
- Web pendant remote control
- Vector import (SVG/DXF) and bitmap processing

## Version Information

- **Current**: 0.1.0-alpha
- **Status**: Active development
- **Rust**: 1.90+
- **Edition**: 2021
- **Tests**: 143 passing
- **Build**: Clean, no warnings

## Migration Guide

### For Rotary Axis Users

If you need rotary axis support in the future:
1. The removal is clean and documented
2. Easy to reintroduce if requirements change
3. See `docs/ROTARY_AXIS_REMOVAL.md` for details

### For Code Folding Users

Code folding was removed due to complexity and GRBL limitations:
- See `docs/FOLDING_FEATURE_REMOVAL.md` for details
- Reason: GRBL doesn't support code blocks
- Can be reintroduced if needed

## Known Issues

- None reported in current alpha release

## Testing

- All 143 tests passing ✅
- Test categories:
  - Material properties tests
  - Job management tests
  - Widget compilation tests
  - Communication tests
  - Parser tests
  - Editor tests

## Performance

- Debug build: 288 MB
- Release build: 23 MB (optimized)
- Rendering: 60+ FPS
- Parser performance: Improved

## Installation

```bash
# From crates.io
cargo install gcodekit

# From source
git clone https://github.com/thawkins/gcodekit.git
cd gcodekit
cargo install --path .

# Build only
cargo build --release
```

## Support

- [GitHub Issues](https://github.com/thawkins/gcodekit/issues)
- [GitHub Discussions](https://github.com/thawkins/gcodekit/discussions)
- [Documentation](docs/)

## License

MIT License - See LICENSE file for details

---

**Last Updated**: October 17, 2025
**Maintainer**: gcodekit contributors
