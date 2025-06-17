# Fast2D Blade Browser Example

A web-based Fast2D application using the Blade Graphics WebGPU backend, combining the browser compatibility of `browser_example` with the modern WebGPU rendering of `blade_example`.

## Overview

This example demonstrates:
- **Blade Graphics Backend**: Using Blade Graphics WebGPU backend for web
- **WebGPU Backend**: Modern GPU API through WebGPU-enabled browsers
- **Fast2D Integration**: 2D graphics primitives with Blade-inspired rendering
- **MoonZoon Framework**: Full-stack Rust web development
- **WebGPU Required**: Only works in WebGPU-enabled browsers (Chrome 113+, Edge 113+)

**Note**: This example implements Blade Graphics WebGPU backend adapted for browser use.

## Features

- **Rectangle Example**: Simple colored shapes with text labels
- **Face Example**: Complex composition with circles, rectangles, and lines
- **Sine Wave Example**: Dynamic mathematical curves
- **Font Rendering**: Multiple font families with various styles
- **Responsive Layout**: Automatic canvas resizing and scrolling

## Browser Requirements

### Supported Browsers
- **Chrome 113+** (Stable - WebGPU enabled by default)
- **Edge 113+** (Stable - WebGPU enabled by default)
- **Firefox 113+** (Experimental - requires manual enablement)
- **Safari 17+** (Experimental - requires manual enablement)

### Enabling WebGPU in Firefox
1. Open `about:config`
2. Set `dom.webgpu.enabled` to `true`
3. Restart Firefox

### Enabling WebGPU in Safari
1. Safari → Settings → Advanced → Feature Flags
2. Enable "WebGPU"
3. Restart Safari

## Quick Start

### Prerequisites
- Rust 1.70+
- [MoonZoon CLI](https://github.com/MoonZoon/MoonZoon#-installation) - **Required**

### Installation of MoonZoon CLI

```bash
# Install MoonZoon CLI (required for building)
cargo install --git https://github.com/MoonZoon/MoonZoon --bin mzoon

# Verify installation
mzoon --version
```

### Development

```bash
# Navigate to the example directory
cd examples/blade_browser_example

# Start development server with hot reload
mzoon start

# Or using cargo-make (if you have cargo-make installed)
cargo make serve
```

Open http://localhost:8087 in a browser.

**Note**: The Blade WebGPU backend includes coordinate normalization fixes and text rendering infrastructure.

### Production Build

```bash
# Build for production
mzoon build --release

# Or with WebGPU unstable features (if needed)
cargo make build-webgpu
```

## Architecture

### Project Structure
```
blade_browser_example/
├── .gitignore         # Excludes pkg/, wasm-bindgen*, build artifacts
├── frontend/          # WASM application (Zoon + Blade)
├── backend/           # Static file server (Moon)
├── shared/            # Common types
├── public/fonts/      # Font assets
└── IMPLEMENTATION_SPEC.md  # Technical specification
```

### Build Artifacts (Git Ignored)
- `frontend/pkg/` - Generated WASM and JS files from wasm-bindgen
- `frontend/wasm-bindgen*` - wasm-bindgen tooling downloads
- `target/` - Rust compilation cache
- `*.log` - Build and development server logs

### Technology Stack
- **Frontend**: Zoon (MoonZoon) + Blade Graphics WebGPU
- **Backend**: Moon (MoonZoon static server)
- **Graphics**: Fast2D + Blade Graphics WebGPU backend
- **Build**: MoonZoon CLI + wasm-bindgen

## Development

### Running Tests
```bash
# Test in Chrome (WebGPU stable)
mzoon start

# Test in Firefox (enable WebGPU first)
# Test in Safari (enable WebGPU first)
```

### Debugging WebGPU

#### Chrome DevTools
1. Open DevTools (F12)
2. Console tab for Rust logs
3. Rendering tab → "WebGPU" for GPU debugging

#### Performance Monitoring
```bash
# Enable timing features
RUSTFLAGS="--cfg=web_sys_unstable_apis" mzoon start
```

### Common Issues

#### "WebGPU Not Supported"
- Ensure browser supports WebGPU
- Check browser flags/settings
- Try Chrome 113+ for best compatibility

#### "Blade WebGPU Initialization Failed"  
- GPU driver may be incompatible
- Try different browser
- Check browser console for detailed errors

#### Canvas Not Rendering
- Verify WebGPU is enabled
- Check console for JavaScript errors
- Ensure canvas element has proper dimensions

## Comparing Backends

### vs `browser_example` (WebGL)
- **Performance**: Potentially faster with WebGPU
- **Features**: More modern GPU features available
- **Compatibility**: More limited browser support
- **Bundle Size**: Larger due to Blade Graphics

### vs `blade_example` (Native)
- **Platform**: Browser instead of desktop
- **Deployment**: Web deployment vs native installation
- **Development**: Hot reload vs rebuild cycle
- **APIs**: WebGPU limitations vs full desktop GPU access

## Performance

### Expected Performance
- **60 FPS** rendering on modern GPUs
- **Smooth resizing** without white flashing
- **Low latency** mouse interaction

### Optimization Tips
- Use Chrome for best WebGPU performance
- Enable hardware acceleration in browser
- Close other GPU-intensive applications

## Troubleshooting

### Build Issues
```bash
# Clean build artifacts
mzoon clean

# Rebuild with verbose output
RUST_LOG=debug mzoon start
```

### Runtime Issues
```bash
# Check browser console for errors
# Enable verbose logging
RUST_LOG=blade_graphics=debug mzoon start
```

### WebGPU Validation Errors
- Common in development builds
- Usually safe to ignore minor validation warnings
- Use `validation: false` in production if needed

## Contributing

### Code Style
- Follow existing patterns from `browser_example` and `blade_example`
- Use `cargo fmt` for formatting
- Test in multiple browsers

### Adding Features
1. Extend examples in `frontend/src/main.rs`
2. Update implementation spec if architectural changes
3. Test WebGPU compatibility across browsers

## License

Same as Fast2D project license.

## Related Examples

- [`browser_example`](../browser_example/) - WebGL version
- [`blade_example`](../blade_example/) - Native desktop version
- [`native_tao_example`](../native_tao_example/) - Native WGPU version