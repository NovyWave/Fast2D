# Fast2D CEF Example

This example demonstrates Fast2D graphics rendering using **CEF (Chromium Embedded Framework)** instead of Tauri's default WebKitGTK backend. This solves WebGL compatibility issues on Linux with NVIDIA GPUs.

## Why CEF?

- **✅ WebGL works reliably** on Linux + NVIDIA
- **✅ WebGPU support** (future-ready)
- **✅ Same Chromium engine** as Chrome browser
- **✅ Cross-platform consistency**
- **⚠️ Larger binary size** (~100MB vs ~10MB)

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CEF Desktop   │───▶│  MoonZoon Dev   │───▶│   Fast2D        │
│   Application   │    │     Server      │    │   Graphics      │
│                 │    │ (localhost:8080)│    │  (WebGL/WebGPU) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Prerequisites

1. Install Rust: https://rustup.rs/
2. Install cargo-make: `cargo install cargo-make`
3. Install dependencies: `makers install`

## Development

```bash
# Start MoonZoon server + CEF app
makers cef_dev

# Or run components separately:
makers mzoon start    # Terminal 1: Start MoonZoon dev server
makers cef           # Terminal 2: Start CEF application
```

## Building

```bash
# Build for release
makers cef_build

# The binary will be in: src-cef/target/release/tauri_cef_example
```

## Project Structure

- **frontend/** - MoonZoon frontend with Fast2D (WebGL)
- **backend/** - MoonZoon backend server
- **shared/** - Shared types between frontend/backend
- **src-cef/** - CEF desktop application wrapper
- **public/fonts/** - Font assets for Fast2D text rendering

## Features Demonstrated

1. **Rectangle rendering** with colors and transparency
2. **Circle rendering** with face graphics
3. **Line rendering** with sine wave animation
4. **Text rendering** with multiple fonts and styles
5. **WebGL acceleration** via CEF/Chromium

## Comparison with tauri_example

| Feature | tauri_example | tauri_cef_example |
|---------|---------------|-------------------|
| WebView Engine | WebKitGTK | CEF/Chromium |
| Linux WebGL | ❌ NVIDIA issues | ✅ Works reliably |
| WebGPU Support | ❌ Not available | ✅ Available |
| Binary Size | ~10MB | ~100MB |
| Compatibility | Platform-dependent | Chrome-consistent |

## Troubleshooting

### Linux Dependencies
If you get build errors, install CEF dependencies:
```bash
sudo apt install libx11-dev libxcomposite-dev libxcursor-dev \
  libxdamage-dev libxext-dev libxfixes-dev libxi-dev \
  libxrandr-dev libxrender-dev libxss-dev libxtst-dev \
  libgtk-3-dev libgdk-pixbuf2.0-dev
```

### MoonZoon Server Not Found
Make sure the MoonZoon dev server is running on port 8080:
```bash
makers mzoon start
```

### CEF Runtime Missing
The CEF runtime will be downloaded automatically on first build. If this fails, check your internet connection and proxy settings.

## Next Steps

- Try switching between WebGL and WebGPU in `frontend/Cargo.toml`
- Compare performance with the original `tauri_example`
- Experiment with more complex Fast2D graphics