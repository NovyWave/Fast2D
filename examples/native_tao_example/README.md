# Native Fast2D Desktop Example

A native desktop application demonstrating Fast2D graphics using the Tao windowing library. This example shows the same functionality as the `tauri_example` but runs purely natively without web technologies.

## Features

- **Pure Native**: No webview or browser dependencies  
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Column Layout**: All 3 Fast2D examples displayed in a scrollable column:
  - Simple Rectangle with text
  - Face example (circles and styling)
  - Sine wave visualization
- **Extended Fast2D**: Native WGPU backend with preserved API compatibility
- **Embedded Fonts**: No external font dependencies
- **Mouse Wheel Scrolling**: Navigate between examples

## Architecture

### Extended Fast2D with Native Backend
- **Native WGPU Backend**: Full native support added to Fast2D
- **API Compatibility**: Same `update_objects()` and `resized()` methods as web version
- **Single Surface Design**: One native window with WGPU surface
- **Column Layout**: All examples displayed simultaneously in scrollable view

### Key Components
- **NativeApp**: Main application coordinator with resize handling
- **Fast2D Native Backend**: WGPU-based rendering with font support
- **CanvasWrapper**: Native equivalent of web CanvasWrapper
- **Standard WGPU Patterns**: Community-standard resize and surface error handling

## Running the Example

### Prerequisites
- Rust 1.70+ 
- Platform-specific graphics drivers (Vulkan/Metal/DirectX)

### Development
```bash
# Run the native Fast2D example
cargo run

# Build for release
cargo build --release
```

### Controls
- **Mouse Wheel**: Scroll up/down to see all three examples
- **Window Resize**: Resizable window with stable graphics (work in progress)
- **Close Window**: Exit the application

## Dependencies

- **tao**: Cross-platform windowing library (async-friendly fork of winit)
- **wgpu**: Modern cross-platform graphics API
- **fast2d**: 2D graphics library (extended with native support)
- **tokio**: Async runtime for async/await patterns
- **glyphon**: High-performance text rendering for WGPU
- **lyon**: 2D path tessellation for vector graphics

## Implementation Status

### ✅ Completed  
- [x] **Extended Fast2D** with native WGPU backend
- [x] **Native CanvasWrapper** with identical API to web version
- [x] **Example migration** - same objects and rendering as web version
- [x] **Column layout** showing all three examples simultaneously  
- [x] **Mouse wheel scrolling** to navigate between examples
- [x] **Font loading** with embedded assets (Inter, FiraCode)
- [x] **Standard WGPU resize patterns** for stable window operations
- [x] **Cross-platform support** (Windows, macOS, Linux)

### 🚧 Current Status: Good Enough
- ✅ **Functional native Fast2D example** - all features working
- ⚠️ **Window resize stability** - mostly stable with some white flashing during resize
- ✅ **API compatibility preserved** - same Fast2D API as web version
- ✅ **All examples visible** - Rectangle, Face, Sine Wave in scrollable column

### 📋 Future Improvements
- [ ] Perfect resize stability (eliminate remaining white flashing)
- [ ] Performance optimizations for complex scenes
- [ ] Additional WGPU backend features
- [ ] More comprehensive error handling

## Technical Achievements

### Fast2D Extension
Successfully extended Fast2D library to support native platforms while preserving complete API compatibility:

```rust
// Same API works on both web and native!
canvas_wrapper.update_objects(|objects| {
    *objects = vec![
        Rectangle::new().position(50., 50.).size(200., 100.).into(),
        Text::new().text("Hello World").position(60., 75.).into(),
    ];
});
```

### Native Backend Architecture
- **Feature Flags**: `native` feature enables WGPU backend  
- **Conditional Compilation**: Web and native backends coexist
- **Font Loading**: Embedded assets replace HTTP fetch
- **Surface Management**: Native WGPU surface creation and management
- **Error Handling**: Standard WGPU surface error recovery patterns

### Research-Based Resize Implementation
Implemented the current community-standard WGPU resize pattern (2025):
- Surface error propagation from draw function to event loop
- Automatic recovery through window size queries
- Platform-optimized present modes (Immediate on Windows, Fifo elsewhere)
- Proper separation of resize and render operations

## Contributing

See the main [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for detailed development guidelines.

## License

Same as Fast2D project license.