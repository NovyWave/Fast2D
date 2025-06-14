# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a **CEF-based alternative** to the `tauri_example`, designed to solve graphics compatibility issues on Linux + NVIDIA systems by using Chromium Embedded Framework instead of Tauri's WebKitGTK backend.

### Core Components
- **Frontend**: Rust/WASM frontend using MoonZoon/Zoon framework with Fast2D graphics rendering
- **Backend**: MoonZoon backend server for development 
- **CEF Wrapper**: Desktop application using CEF instead of Tauri  
- **Shared**: Common types and logic shared between frontend and backend
- **Fast2D Integration**: 2D graphics library with WebGPU backend (will work reliably with CEF)

### Current Status
**✅ DEPENDENCY RESOLUTION COMPLETE**: CEF integration dependency conflicts have been resolved:
- Updated MoonZoon `rustls-pemfile: 2.0.0 → 2.2.0` 
- Verified compatibility with Fast2D project
- Connected Fast2D to use local MoonZoon repository
- CEF dependencies ready for integration

### Key Differences from tauri_example (when completed)
- **CEF instead of Tauri**: Uses official `tauri-apps/cef-rs` bindings
- **Chromium WebView**: Full Chrome engine instead of WebKitGTK
- **Graphics guaranteed**: No NVIDIA + Linux compatibility issues
- **Larger binary**: ~100MB runtime vs ~10MB with Tauri

### Workspace Structure
- `frontend/`: WASM frontend with Fast2D canvas examples (rectangles, faces, sine waves)
- `backend/`: MoonZoon server for development and serving static assets
- `src-cef/`: CEF desktop application wrapper with Chromium engine
- `shared/`: Shared Rust code (currently minimal)
- `public/fonts/`: Font assets (FiraCode, Inter family) loaded asynchronously

## Development Commands

### Initial Setup
```bash
# Install dependencies (CEF deps, wasm target, mzoon CLI)
makers install
```

### Development
```bash
# Start CEF app with MoonZoon server (recommended)
makers cef_dev

# Alternative: Start components separately
makers mzoon start  # Terminal 1
makers cef         # Terminal 2
```

### CEF Binary Management
**Recommended**: CEF binaries are **downloaded during build**, not stored in git:
- ✅ Faster cloning (no large binaries)
- ✅ Always latest compatible version
- ✅ Platform-specific downloads
- ✅ No Git LFS costs or complexity

CEF binaries (~100-200MB) are automatically downloaded by the build system and cached locally.

### Building
```bash
# Build for release
makers cef_build

# Binary location: src-cef/target/release/cef_example
```

### Available Maker Tasks
- `makers install` - Install all dependencies (wasm target, mzoon CLI, CEF deps)
- `makers cef_dev` - Start MoonZoon server + CEF app concurrently
- `makers cef` - Run CEF application (expects server running)
- `makers cef_build` - Build CEF application for release
- `makers mzoon [args]` - Run MoonZoon CLI

## Key Configuration Files

- `MoonZoon.toml`: MoonZoon development server configuration (port 8080, watch paths)
- `Makefile.toml`: Build system configuration with CEF-specific tasks
- `src-cef/Cargo.toml`: CEF application dependencies (`cef-rs`, `winit`, `tokio`)
- Workspace `Cargo.toml`: Defines Fast2D dependency from local path `../../../crates/fast2d`

## Fast2D Integration Pattern

The frontend demonstrates Fast2D canvas integration (identical to `tauri_example`):
1. Load fonts asynchronously using `fast2d::fetch_file()` and `fast2d::register_fonts()`
2. Create Zoon Canvas elements and extract DOM canvas
3. Wrap with `fast2d::CanvasWrapper` for 2D object rendering
4. Update objects using `canvas_wrapper.update_objects()` with collections of `fast2d::Object2d`

## CEF-Specific Implementation

### CEF Application (`src-cef/src/main.rs`)
- **Async server check**: Waits for MoonZoon dev server before starting CEF
- **Hardware acceleration**: Enables GPU acceleration
- **Chrome runtime**: Uses full Chrome features for best compatibility

### Build Configuration (`src-cef/build.rs`)
- **Platform-specific linking**: Links required system libraries
- **GPU libraries**: Links OpenGL/EGL on Linux for hardware acceleration

### Dependencies (`src-cef/Cargo.toml`)
- **cef-rs**: CEF (Chromium Embedded Framework) bindings
- **tokio**: Async runtime for server checking
- **reqwest**: HTTP client for health checks

## Benefits Over tauri_example

1. **Graphics reliability**: Works on Linux + NVIDIA without driver issues
2. **WebGPU support**: Access to modern graphics APIs
3. **Chrome consistency**: Same rendering as Chrome browser
4. **Future-proof**: Will work with upcoming web standards

## Trade-offs

1. **Binary size**: ~100MB vs ~10MB (CEF runtime included)
2. **Memory usage**: Higher memory footprint
3. **Startup time**: Slightly slower due to CEF initialization
4. **Deployment**: Need to bundle CEF runtime

## Troubleshooting

### Common Issues
- **CEF dependencies missing**: Run `makers install` to install platform libraries
- **Server not found**: Ensure MoonZoon server is running on port 8080
- **Build failures**: Check that all CEF system dependencies are installed

### Linux-Specific
- Install development headers: `libx11-dev`, `libgtk-3-dev`, etc.
- GPU drivers should work out-of-the-box (no WebKitGTK issues)

### Performance
- CEF performs identically to Chrome browser
- Graphics rendering should work smoothly on all platforms
- No need for software rendering fallbacks