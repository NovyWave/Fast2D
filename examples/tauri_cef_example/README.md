# Fast2D CEF Example - WebGL Graphics with Chromium

This example demonstrates Fast2D graphics using **Chromium Embedded Framework (CEF)** instead of Tauri/WebKitGTK, providing reliable WebGL support on Linux + NVIDIA systems.

## 🚀 Quick Start

```bash
# 1. Install dependencies
makers install

# 2. Enable CEF (edit src-cef/Cargo.toml)
# Uncomment: cef = { git = "https://github.com/tauri-apps/cef-rs", branch = "dev" }

# 3. Start development server + CEF app  
makers cef_dev
```

The CEF window will open displaying Fast2D WebGL graphics examples.

## 🎯 Why CEF?

**Problem**: WebKitGTK shows black canvases on Linux + NVIDIA  
**Solution**: Use Chromium engine for guaranteed WebGL support

Benefits:
- ✅ **Reliable WebGL** on all platforms including Linux + NVIDIA
- ✅ **Hardware acceleration** guaranteed  
- ✅ **Chrome DevTools** for debugging
- ✅ **Modern web standards** support
- ⚠️ **Larger binary** (~100MB vs ~10MB)

## 📋 Step-by-Step Setup

### 1. Enable CEF Dependencies

Edit `src-cef/Cargo.toml` and uncomment:

```toml
# Uncomment these lines:
cef = { git = "https://github.com/tauri-apps/cef-rs", branch = "dev" }

[build-dependencies]  
download-cef = { git = "https://github.com/tauri-apps/cef-rs", branch = "dev" }
```

### 2. Install System Dependencies

**Linux:**
```bash
sudo apt install libx11-dev libgtk-3-dev libxcb1-dev
```

**macOS:**
```bash
xcode-select --install
```

### 3. Build and Run

```bash
# Build CEF application (downloads CEF runtime on first build)
cargo build --bin tauri_cef_example

# Start both server and CEF app
makers cef_dev
```

## 🛠️ Available Commands

| Command | Description |
|---------|-------------|
| `makers install` | Install all dependencies |
| `makers cef_dev` | Start dev server + CEF app |
| `makers cef` | Run CEF app only |
| `makers cef_build` | Build release binary |
| `makers mzoon start` | Start MoonZoon server |

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

### CEF build fails
```bash
# Linux: Install CEF dependencies
sudo apt install libx11-dev libgtk-3-dev libxcb1-dev

# macOS: Install Xcode tools
xcode-select --install
```

### Server not running
```bash
# Check if server is running
curl http://localhost:8080

# Start server if needed
makers mzoon start
```

### Black screen issues
- CEF should work out-of-the-box (no WebKitGTK issues)
- Check browser console (F12) for errors
- Verify GPU drivers are installed

## 🔗 Resources

- [Fast2D Documentation](https://github.com/MartinKavik/Fast2D)
- [CEF Project](https://bitbucket.org/chromiumembedded/cef)
- [Tauri CEF Bindings](https://github.com/tauri-apps/cef-rs)
- [MoonZoon Framework](https://github.com/MoonZoon/MoonZoon)

## 📝 Notes

- CEF binaries (~100-200MB) download automatically on first build
- Binaries cached in `target/` directory (excluded from git)
- For production deployment, bundle CEF runtime with application
- Performance identical to Chrome browser

---

**Created to solve WebGL compatibility issues with WebKitGTK on Linux + NVIDIA systems.**