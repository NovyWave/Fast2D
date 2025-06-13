# Fast2D CEF Example - WebGL Graphics with Chromium

This example demonstrates Fast2D graphics using **Chromium Embedded Framework (CEF)** instead of Tauri/WebKitGTK, providing reliable WebGL support on all platforms.

## 🚀 Quick Start

```bash
# 1. Start MoonZoon development server
cd /path/to/Fast2D/examples/tauri_cef_example
makers mzoon start

# 2. In another terminal, run CEF application
cd src-cef
cargo run --bin cef_example
```

A CEF window will open displaying Fast2D WebGL graphics examples with animated rectangles, faces, and sine waves.

## 🎯 Why CEF?

**Problem**: WebKitGTK shows black canvases on Linux + NVIDIA  
**Solution**: Use Chromium engine for guaranteed WebGL support

Benefits:
- ✅ **Reliable WebGL** on all platforms including Linux + NVIDIA
- ✅ **Hardware acceleration** guaranteed  
- ✅ **Chrome DevTools** for debugging
- ✅ **Modern web standards** support
- ⚠️ **Larger binary** (~100MB vs ~10MB)

## 📋 Prerequisites

### System Dependencies

**Linux:**
```bash
sudo apt install libx11-dev libgtk-3-dev libxcb1-dev
```

**macOS:**
```bash
xcode-select --install
```

### Required Tools
- **Rust** with `wasm32-unknown-unknown` target
- **makers** (cargo-make): `cargo install cargo-make`

## 📋 Step-by-Step Setup

### 1. Clone and Setup Fast2D

```bash
git clone https://github.com/MartinKavik/Fast2D.git
cd Fast2D/examples/tauri_cef_example
```

### 2. Install Dependencies

```bash
makers install  # Installs wasm target and mzoon CLI
```

### 3. Start MoonZoon Server

```bash
makers mzoon start
```
This starts the development server on `http://localhost:8080` serving the Fast2D frontend.

### 4. Run CEF Application

```bash
# In another terminal
cd src-cef
cargo run --bin tauri_cef_example
```

The CEF window should appear showing Fast2D graphics.

## 🛠️ Available Commands

| Command | Description |
|---------|-------------|
| `makers install` | Install wasm target and mzoon CLI |
| `makers mzoon start` | Start MoonZoon development server |
| `cargo run --bin cef_example` | Run CEF application (from src-cef/) |
| `cargo build --release` | Build optimized CEF binary |

## 📁 Project Structure

```
tauri_cef_example/
├── frontend/        # Fast2D WASM frontend
├── backend/         # MoonZoon server
├── src-cef/         # CEF desktop wrapper
│   ├── Cargo.toml   # CEF dependencies
│   ├── build.rs     # Build configuration
│   └── src/main.rs  # CEF application
├── shared/          # Common types
└── public/          # Static assets
```

## 🎨 Fast2D Graphics Demonstrated

1. **Rectangle rendering** with colors and transparency
2. **Circle rendering** with face graphics  
3. **Line rendering** with sine wave animation
4. **Text rendering** with multiple fonts and styles
5. **WebGL acceleration** via CEF/Chromium

## 📊 Comparison: CEF vs Tauri

| Feature | Tauri (WebKitGTK) | CEF (This Example) |
|---------|-------------------|--------------------|
| WebGL on Linux+NVIDIA | ❌ Black screens | ✅ Works reliably |
| WebGPU Support | ❌ Limited | ✅ Full support |
| Binary Size | ~10MB | ~100MB |
| Memory Usage | ~50MB | ~150MB |
| Web Standards | Platform-dependent | Chrome-consistent |
| DevTools | Basic | Full Chrome |

## 🐛 Troubleshooting

### "Server not available" error
```bash
# Check if MoonZoon server is running
curl http://localhost:8080

# If not running, start it
makers mzoon start
```

### CEF build fails
```bash
# Install system dependencies
sudo apt install libx11-dev libgtk-3-dev libxcb1-dev  # Linux
xcode-select --install  # macOS
```

### Window doesn't appear
- Check that you're running on the correct display
- Try different DISPLAY values: `DISPLAY=:0` or `DISPLAY=:1`
- Ensure no other instances are running: `pkill cef_example`

### Context menu positioning (multi-monitor)
- Known issue with CEF on multi-monitor setups
- Window automatically positions on primary monitor to minimize issue

## 🔗 Resources

- [Fast2D Documentation](https://github.com/MartinKavik/Fast2D)
- [CEF Project](https://bitbucket.org/chromiumembedded/cef)
- [Tauri CEF Bindings](https://github.com/tauri-apps/cef-rs)
- [MoonZoon Framework](https://github.com/MoonZoon/MoonZoon)

## 📝 Notes

- **CEF binaries** (~100-200MB) download automatically during build
- **Build cache** stored in `src-cef/target/` directory
- **Performance** identical to Chrome browser
- **Multi-monitor** support with automatic primary monitor positioning
- **WebGL** guaranteed to work (no WebKitGTK black screen issues)

## 🏆 Success Criteria

- ✅ **Window appears** showing Fast2D graphics
- ✅ **Animations work** (rectangles, faces, sine waves)  
- ✅ **No black screens** (CEF provides reliable WebGL)
- ✅ **Hardware acceleration** enabled
- ✅ **Cross-platform** compatibility

---

**Alternative to Tauri for reliable WebGL graphics on all platforms, especially Linux + NVIDIA systems.**