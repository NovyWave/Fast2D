# Native Tao + Fast2D Implementation Plan

## Overview
This document outlines the comprehensive plan for creating a native desktop example using Tao windowing library and Fast2D graphics, migrating from the existing `tauri_example` while preserving all functionality.

## Architecture Analysis

### Current Tauri Example Structure
```
tauri_example/
├── frontend/src/main.rs (WASM + Zoon UI framework)
│   ├── 3 example functions: rectangle, face, sine_wave
│   ├── Async font loading from HTTP endpoints
│   ├── Scrollable column layout with panels
│   ├── Each panel: 650px max width, 350px height, gray border
│   └── Fast2D CanvasWrapper per panel
├── backend/ (Moon web server)
├── src-tauri/ (Tauri wrapper)
└── public/fonts/ (Static font files)
```

### Reusable Components Identified
1. **Example Objects**: `example_rectangle()`, `example_face()`, `example_sine_wave()` functions
2. **Font Assets**: Same font files (FiraCode-Regular.ttf, Inter-*.ttf)
3. **Fast2D Object Creation Patterns**: All the graphics creation logic
4. **Layout Specifications**: Panel dimensions, spacing, colors

## Native Architecture Design

### Core Architecture: Single Surface Multi-Region
```
NativeApp
├── Window (Tao async window)
├── Graphics (Single WGPU Surface)
├── LayoutManager
│   ├── ScrollState { offset_y: f32 }
│   ├── PanelLayout { y: f32, height: f32, visible: bool }
│   └── ViewportCalculator
├── CanvasRegion[3]
│   ├── Fast2D Objects (reused from tauri_example)
│   ├── LocalViewport { x, y, width, height }
│   ├── RenderState { needs_redraw: bool }
│   └── InputHitTest
├── InputRouter
│   ├── MouseEventRouter
│   ├── KeyboardEventRouter
│   └── ScrollEventHandler
├── FontManager (embedded fonts)
└── RenderPipeline
    ├── ClearPass
    ├── RegionRenderPass[3]
    └── CompositePass
```

### Key Design Principles

1. **Zero-Copy Example Migration**: Reuse existing example functions without modification
2. **Async-First**: All operations use Tao's async APIs
3. **Performance**: Single surface, batched rendering, minimal allocations
4. **Maintainability**: Clear separation of concerns, modular components
5. **Cross-Platform**: Works on Windows, macOS, Linux

## Detailed Component Design

### 1. Window Management (Tao)
```rust
struct NativeWindow {
    window: tao::window::Window,
    event_loop: tao::event_loop::EventLoop<UserEvent>,
    size: (u32, u32),
}

impl NativeWindow {
    async fn new() -> Result<Self>;
    async fn handle_events(&mut self) -> Result<()>;
    async fn resize(&mut self, size: (u32, u32)) -> Result<()>;
}
```

### 2. Graphics Pipeline (WGPU)
```rust
struct GraphicsContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}

struct RenderPipeline {
    clear_color: wgpu::Color, // Black background
    regions: Vec<CanvasRegion>,
}
```

### 3. Layout Manager
```rust
struct LayoutManager {
    scroll_offset: f32,
    panel_height: f32, // 350px + padding
    panel_spacing: f32, // 10px
    total_height: f32,
    viewport_height: f32,
}

impl LayoutManager {
    fn calculate_panel_positions(&self) -> Vec<PanelLayout>;
    fn handle_scroll(&mut self, delta: f32);
    fn is_panel_visible(&self, panel_index: usize) -> bool;
}
```

### 4. Canvas Regions
```rust
struct CanvasRegion {
    id: usize,
    objects: Vec<fast2d::Object2d>, // Reused from examples
    viewport: Viewport,
    fast2d_wrapper: fast2d::CanvasWrapper,
    needs_redraw: bool,
}

struct Viewport {
    x: f32, y: f32,      // Screen position
    width: f32, height: f32, // 650x350 max
}
```

### 5. Input Routing
```rust
struct InputRouter {
    layout_manager: Arc<Mutex<LayoutManager>>,
    canvas_regions: Arc<Mutex<Vec<CanvasRegion>>>,
}

impl InputRouter {
    fn route_mouse_event(&self, event: MouseEvent) -> Option<usize>;
    fn handle_scroll(&self, delta: f32);
    fn handle_resize(&self, size: (u32, u32));
}
```

## Migration Strategy

### Phase 1: Direct Code Reuse (Zero Changes)
1. Copy `example_rectangle()`, `example_face()`, `example_sine_wave()` exactly
2. Copy font files to `assets/fonts/`
3. Reuse Fast2D object creation patterns
4. Preserve all visual specifications (colors, sizes, positions)

### Phase 2: Native Adaptations
1. Replace HTTP font loading with embedded assets using `include_bytes!`
2. Replace zoon UI layout with native layout calculations
3. Replace DOM event handling with Tao event handling
4. Replace Canvas DOM elements with WGPU surfaces

### Phase 3: Enhancement Opportunities
1. Add native menu bar
2. Add keyboard shortcuts
3. Add window icon
4. Add native file dialogs for export functionality
5. Better font rendering with system fonts fallback

## Font Management Strategy

### Current (HTTP-based)
```rust
let fonts = try_join_all([
    fast2d::fetch_file("/_api/public/fonts/FiraCode-Regular.ttf"),
    // ...
]).await.unwrap_throw();
```

### Native (Embedded)
```rust
const FIRA_CODE: &[u8] = include_bytes!("../assets/fonts/FiraCode-Regular.ttf");
const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");
const INTER_BOLD_ITALIC: &[u8] = include_bytes!("../assets/fonts/Inter-BoldItalic.ttf");

async fn load_embedded_fonts() -> Result<()> {
    let fonts = vec![
        FIRA_CODE.to_vec(),
        INTER_REGULAR.to_vec(),
        INTER_BOLD.to_vec(),
        INTER_BOLD_ITALIC.to_vec(),
    ];
    fast2d::register_fonts(fonts)?;
    Ok(())
}
```

## Project Structure

```
native_tao_example/
├── Cargo.toml
├── IMPLEMENTATION_PLAN.md (this file)
├── README.md
├── src/
│   ├── main.rs              # Entry point, Tao setup
│   ├── app.rs               # Main application struct
│   ├── window.rs            # Window management
│   ├── graphics.rs          # WGPU setup and rendering
│   ├── layout.rs            # Layout management
│   ├── input.rs             # Input handling
│   ├── canvas_region.rs     # Canvas region implementation
│   └── examples.rs          # Reused example functions
├── assets/
│   └── fonts/
│       ├── FiraCode-Regular.ttf
│       ├── Inter-Regular.ttf
│       ├── Inter-Bold.ttf
│       └── Inter-BoldItalic.ttf
└── tests/
    ├── integration/
    │   ├── rendering_tests.rs
    │   ├── input_tests.rs
    │   └── layout_tests.rs
    └── unit/
        ├── examples_tests.rs
        └── layout_tests.rs
```

## Testing Strategy

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_functions_unchanged() {
        // Verify example functions produce identical objects
        let rect = example_rectangle();
        assert_eq!(rect.len(), 2); // Rectangle + text
        // ... detailed object validation
    }
    
    #[test]
    fn test_layout_calculations() {
        let layout = LayoutManager::new(1200, 800);
        let positions = layout.calculate_panel_positions();
        assert_eq!(positions.len(), 3);
        // ... layout validation
    }
}
```

### 2. Integration Tests
```rust
#[tokio::test]
async fn test_full_rendering_pipeline() {
    let app = NativeApp::new().await.unwrap();
    
    // Test that all 3 canvases render without errors
    for i in 0..3 {
        let result = app.render_region(i).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_input_routing() {
    let app = NativeApp::new().await.unwrap();
    
    // Test mouse events route to correct canvas
    let mouse_event = MouseEvent::new(100.0, 200.0);
    let target_region = app.input_router.route_mouse_event(mouse_event);
    assert_eq!(target_region, Some(0)); // First canvas
}
```

### 3. Visual Regression Tests
```rust
#[tokio::test]
async fn test_visual_output_matches_web() {
    // Render each example to image buffer
    // Compare with reference images from web version
    // Allows for minor platform differences
}
```

### 4. Performance Tests
```rust
#[tokio::test]
async fn test_rendering_performance() {
    let mut app = NativeApp::new().await.unwrap();
    
    let start = Instant::now();
    for _ in 0..100 {
        app.render_frame().await.unwrap();
    }
    let duration = start.elapsed();
    
    // Should render 100 frames in less than 1 second
    assert!(duration < Duration::from_secs(1));
}
```

## Dependencies Strategy

### Core Dependencies
```toml
[dependencies]
# Windowing
tao = "0.16" # Latest async-friendly version

# Graphics  
wgpu = "0.18"
fast2d = { path = "../../crates/fast2d", features = ["webgpu"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Utilities
anyhow = "1.0"
thiserror = "1.0"
bytemuck = "1.14"

[dev-dependencies]
# Testing
tokio-test = "0.4"
image = "0.24" # For visual testing
criterion = "0.5" # For benchmarking
```

### Feature Flags
```toml
[features]
default = ["webgpu"]
webgpu = ["fast2d/webgpu", "wgpu"]
vulkan = ["wgpu/vulkan"]
dx12 = ["wgpu/dx12"]
metal = ["wgpu/metal"]
```

## Build & Distribution

### Development
```bash
# Standard development
cargo run

# With debug graphics
cargo run --features debug-graphics

# Performance profiling
cargo run --release --features profiling
```

### Distribution
```bash
# Cross-platform release builds
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-apple-darwin
```

### Assets Bundling
- Fonts embedded at compile time using `include_bytes!`
- No external dependencies at runtime
- Single executable distribution

## Risk Mitigation

### Technical Risks
1. **Fast2D WGPU Backend Compatibility**
   - *Risk*: Fast2D WGPU backend might not support multiple regions
   - *Mitigation*: Test early, fallback to multiple CanvasWrappers if needed

2. **Tao Async Stability**
   - *Risk*: Tao is less mature than winit
   - *Mitigation*: Comprehensive error handling, fallback to winit if needed

3. **Performance with 3 Canvases**
   - *Risk*: Multiple Fast2D instances might be slow
   - *Mitigation*: Profile early, optimize rendering pipeline

### Project Risks
1. **Breaking Changes in Dependencies**
   - *Mitigation*: Pin exact versions, comprehensive CI testing

2. **Platform-Specific Issues**
   - *Mitigation*: Test on all target platforms, platform-specific code paths

## Success Criteria

### Functional Requirements ✅
- [ ] All 3 examples render identically to web version
- [ ] Smooth scrolling between panels
- [ ] Responsive window resizing
- [ ] Proper font rendering
- [ ] Cross-platform compatibility

### Performance Requirements ✅
- [ ] 60 FPS rendering at 1920x1080
- [ ] < 50MB memory usage
- [ ] < 1 second startup time
- [ ] < 10MB binary size

### Quality Requirements ✅
- [ ] > 90% test coverage
- [ ] No memory leaks
- [ ] No panic conditions
- [ ] Clean, maintainable code

## Timeline

### Week 1: Foundation
- Set up project structure
- Implement basic Tao window
- Set up WGPU context
- Test Fast2D integration

### Week 2: Core Features
- Implement layout manager
- Add input routing
- Migrate example functions
- Implement font loading

### Week 3: Integration
- Connect all components
- Implement rendering pipeline
- Add error handling
- Performance optimization

### Week 4: Testing & Polish
- Write comprehensive tests
- Performance testing
- Cross-platform testing
- Documentation

## Future Enhancements

### Immediate (Next Release)
- Add native menu bar
- Add export functionality
- Add keyboard shortcuts
- Better error messages

### Medium Term
- Add more Fast2D examples
- Plugin system for custom renderers
- Theme support
- Configuration system

### Long Term
- Mobile support (iOS/Android)
- Multi-window support
- Advanced graphics features
- Integration with other Rust GUI frameworks

---

*This plan serves as the blueprint for creating a high-quality, maintainable native desktop application that preserves all functionality from the web version while leveraging the performance and integration benefits of native development.*