# Blade Browser Example - Implementation Specification

## Overview

The `blade_browser_example` combines the browser compatibility of `browser_example` with the Blade Graphics backend from `blade_example`. This creates a web-based Fast2D application that uses WebGPU through the Blade Graphics abstraction layer.

## Architecture

### Browser Application Structure
Following the `browser_example` pattern:
- **Frontend**: WASM application using MoonZoon framework
- **Backend**: Rust server for serving static assets  
- **Shared**: Common types and utilities
- **WebGPU Integration**: Blade Graphics WebGPU backend

### Blade Graphics Integration
Using the same Blade integration as `blade_example`:
- **Blade Graphics**: WebGPU backend from MartinKavik/blade (commit: `711f926`)
- **Fast2D Backend**: Browser-compatible Fast2D with Blade Graphics
- **Canvas Integration**: WebGPU surface from HTML5 Canvas

## Technical Specifications

### Dependencies

**Blade Graphics WebGPU Backend**:
```toml
blade-graphics = { git = "https://github.com/MartinKavik/blade", rev = "711f926" }
```

**Browser Framework**:
```toml
# MoonZoon workspace dependencies
zoon = { git = "https://github.com/MoonZoon/MoonZoon", rev = "7c5178d891cf4afbc2bbbe864ca63588b6c10f2a" }
moon = { git = "https://github.com/MoonZoon/MoonZoon", rev = "7c5178d891cf4afbc2bbbe864ca63588b6c10f2a" }
```

**Fast2D Integration**:
```toml
fast2d = { path = "../../crates/fast2d", features = ["browser"] }
```

### Browser Requirements

**WebGPU Support**:
- Chrome 113+ (Stable)
- Firefox 113+ (Behind flag)
- Safari 17+ (Behind flag)
- Edge 113+ (Stable)

**Build Flags**:
```
RUSTFLAGS="--cfg=web_sys_unstable_apis"
```

### Canvas Integration

**HTML Canvas Element**:
- Canvas ID: `"blade-canvas"` 
- WebGPU Context: `canvas.getContext("webgpu")`
- Surface Creation: Blade Graphics WebGPU surface

**Resize Handling**:
- Monitor canvas size changes
- Reconfigure WebGPU surface
- Update Fast2D canvas wrapper

## Implementation Plan

### Phase 1: Project Structure
- [x] Create `blade_browser_example` directory
- [ ] Copy MoonZoon workspace structure from `browser_example`
- [ ] Adapt Cargo.toml files with Blade dependencies
- [ ] Set up asset pipeline for fonts

### Phase 2: WebGPU Integration
- [ ] Create WebGPU canvas wrapper using Blade Graphics
- [ ] Implement Fast2D browser backend with Blade
- [ ] Port rendering examples from `browser_example`
- [ ] Test WebGPU surface initialization

### Phase 3: MoonZoon Frontend
- [ ] Port UI layout from `browser_example`
- [ ] Integrate Blade canvas with MoonZoon elements
- [ ] Implement async font loading
- [ ] Handle canvas resize events

### Phase 4: Testing & Optimization
- [ ] Test in multiple WebGPU-enabled browsers
- [ ] Optimize WASM bundle size
- [ ] Add error handling for WebGPU fallbacks
- [ ] Performance benchmarking vs WebGL

## File Structure

```
blade_browser_example/
├── Cargo.toml                    # Workspace root
├── IMPLEMENTATION_SPEC.md        # This document
├── README.md                     # Usage instructions
├── Makefile.toml                 # Build automation
├── MoonZoon.toml                 # MoonZoon configuration
├── frontend/                     # WASM frontend
│   ├── Cargo.toml
│   └── src/
│       └── main.rs              # Main frontend logic
├── backend/                      # Static server
│   ├── Cargo.toml
│   └── src/
│       └── main.rs              # Static file server
├── shared/                       # Common types
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── public/                       # Static assets
    └── fonts/                    # Font files
        ├── FiraCode-Regular.ttf
        ├── Inter-Regular.ttf
        ├── Inter-Bold.ttf
        └── Inter-BoldItalic.ttf
```

## Key Differences from Parent Examples

### vs `browser_example`
- **Graphics Backend**: WebGPU via Blade instead of WebGL via Fast2D
- **Rendering**: Direct Blade Graphics commands instead of Fast2D abstraction
- **Performance**: Potential WebGPU performance benefits
- **Browser Support**: More limited (WebGPU required vs WebGL universal)

### vs `blade_example`  
- **Platform**: Browser/WASM instead of native desktop
- **Windowing**: HTML Canvas instead of Tao/winit
- **Build Target**: `wasm32-unknown-unknown` instead of native
- **Framework**: MoonZoon web framework instead of direct event loop

## Rendering Examples

The same three examples from both parent projects:

1. **Rectangle Example**: Simple colored rectangle with text label
2. **Face Example**: Complex shape with circles, rectangles, and lines  
3. **Sine Wave Example**: Dynamic line with mathematical curve

Each example demonstrates different Fast2D primitives rendered through Blade Graphics WebGPU backend.

## Build Process

### Development
```bash
mzoon start
```

### Production Build
```bash
mzoon build --release
```

### WebGPU Feature Flags
```bash
RUSTFLAGS="--cfg=web_sys_unstable_apis" mzoon build --release
```

## Testing Strategy

### Browser Compatibility
- Test in Chrome Canary with WebGPU enabled
- Test in Firefox Nightly with `dom.webgpu.enabled`
- Test in Safari Technology Preview
- Fallback error messages for unsupported browsers

### Performance Validation
- Measure frame rates vs `browser_example`
- Monitor WASM bundle size
- Profile WebGPU vs WebGL performance
- Test resize handling stability

## Known Limitations

1. **WebGPU Availability**: Limited browser support compared to WebGL
2. **Experimental Features**: Requires unstable web-sys APIs
3. **Debug Complexity**: WebGPU debugging tools less mature
4. **Bundle Size**: Blade Graphics may increase WASM size

## Success Criteria

- [ ] Renders all three examples correctly in WebGPU browsers
- [ ] Smooth resize handling without flashing
- [ ] Font loading and text rendering works
- [ ] Performance meets or exceeds WebGL version
- [ ] Clear error messages for unsupported browsers
- [ ] Documentation for running and debugging

---

**Status**: Implementation in progress  
**Last Updated**: 2025-06-17  
**Blade Commit**: `711f926` (WebGPU support)