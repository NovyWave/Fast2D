# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is an **experimental Servo-based alternative** to the `tauri_example` and `cef_example`, designed to explore Servo's embedding capabilities for graphics applications. **IMPORTANT**: This is a research project, not a production solution.

### ⚠️ Experimental Status
- **Servo**: Experimental browser engine, not production-ready as of mid-2025
- **API Instability**: Servo embedding API constantly changing, requires frequent updates
- **Complexity**: Requires ~200 lines of complex integration code vs CEF's ~50 lines
- **Production Alternative**: Use `cef_example` for reliable graphics applications

### Core Components
- **Frontend**: Rust/WASM frontend using MoonZoon/Zoon framework with Fast2D graphics rendering (identical to cef_example)
- **Backend**: MoonZoon backend server for development (identical to cef_example)
- **Servo Wrapper**: Experimental desktop application using Servo browser engine
- **Shared**: Common types and logic shared between frontend and backend (identical to cef_example)
- **Fast2D Integration**: 2D graphics library with WebGL backend (experimental with Servo)

### Key Differences from cef_example
- **Servo instead of CEF**: Uses experimental Servo browser engine
- **Complex API**: Much more complex embedding than CEF's straightforward API
- **Experimental Status**: Not suitable for production use
- **Pure Rust**: All-Rust stack vs CEF's C++ bindings
- **Research Focus**: Learning and exploration rather than production deployment

### Current Implementation Status
- **Basic Structure**: ✅ Complete
- **Servo Integration**: ⚠️ Placeholder implementation (incomplete)
- **WebGL Support**: ❓ Untested/unknown functionality
- **WebGPU Support**: 🧪 Experimental at best
- **Build System**: ✅ Basic structure created

## Development Commands

### Initial Setup
```bash
# Install dependencies (may be incomplete for Servo)
makers install
```

### Development (Experimental)
```bash
# Check if Servo compiles (likely to fail)
makers servo_check

# Start MoonZoon server (same as other examples)
makers mzoon start  

# Attempt Servo app (experimental, may fail)
makers servo
```

### Expected Issues
- **Compilation Failures**: Servo has complex dependencies that may fail to build
- **Runtime Crashes**: Servo embedding API is experimental and unstable
- **Missing Features**: Many web platform APIs not implemented in Servo
- **API Changes**: Code may break with Servo updates

## Key Configuration Files

- `MoonZoon.toml`: MoonZoon development server configuration (identical to cef_example)
- `Makefile.toml`: Build system with Servo-specific tasks (experimental)
- `src-servo/Cargo.toml`: Servo application dependencies (unstable, frequently changing)
- Workspace `Cargo.toml`: Defines Fast2D dependency from local path `../../../crates/fast2d`
- MoonZoon dependencies: Uses GitHub repository with locked commit hash for `moon` and `zoon` crates

## Fast2D Integration Pattern

The frontend demonstrates Fast2D canvas integration (identical to cef_example and tauri_example):
1. Load fonts asynchronously using `fast2d::fetch_file()` and `fast2d::register_fonts()`
2. Create Zoon Canvas elements and extract DOM canvas
3. Wrap with `fast2d::CanvasWrapper` for 2D object rendering
4. Update objects using `canvas_wrapper.update_objects()` with collections of `fast2d::Object2d`

## Servo-Specific Implementation

### Servo Application (`src-servo/src/main.rs`)
- **Complex Integration**: Requires ~200 lines of code vs CEF's simpler approach
- **Placeholder Implementation**: Current code is a structural placeholder
- **Threading Complexity**: Manual cross-thread communication required
- **OpenGL Context**: Manual context management (more complex than CEF)

### Build Configuration (`src-servo/build.rs`)
- **Platform-specific linking**: Links required system libraries for Servo
- **GPU libraries**: Links OpenGL/EGL on Linux for graphics support
- **Complex Dependencies**: May require additional system packages

### Dependencies (`src-servo/Cargo.toml`)
- **servo**: Experimental Servo browser engine (git dependency, unstable)
- **winit**: Window management (manual setup required)
- **glutin**: OpenGL context creation (complex integration)
- **tokio**: Async runtime for server checking (same as cef_example)
- **crossbeam-channel**: Cross-thread communication (required for Servo)

## Research Value

### What This Project Teaches
1. **Browser Engine Complexity**: Understanding why CEF and WebKitGTK are popular
2. **Rust Graphics Programming**: Complex patterns for OpenGL and threading
3. **WebGPU Concepts**: Next-generation graphics API principles
4. **API Design**: Importance of stable, well-documented interfaces
5. **Pure Rust Benefits**: Potential advantages of all-Rust graphics stack

### Expected Learning Outcomes
- Experience with experimental browser engine embedding
- Understanding of cross-thread synchronization in GUI applications
- Knowledge of WebGPU implementation details
- Appreciation for the complexity of browser engine integration
- Comparison of different embedding approaches

## Troubleshooting

### Common Issues
- **Servo Won't Compile**: Missing system dependencies, Rust version issues
- **API Compatibility**: Servo embedding API changes breaking compilation
- **Runtime Failures**: Servo crashes or fails to render content
- **Missing Features**: Web platform APIs not implemented in Servo

### Linux-Specific
- Install extensive development headers for Servo compilation
- May require cutting-edge Rust nightly toolchain
- Graphics drivers compatibility unknown with Servo

### Performance
- Servo compilation is very slow and resource-intensive
- Runtime performance unknown compared to CEF
- May have significant memory overhead

## Development Philosophy

### Research-Oriented Approach
- **Experimentation**: Try things, document failures and successes
- **Learning Focus**: Understanding is more valuable than working code
- **Future Investment**: Preparing for potential Servo maturity
- **Community Value**: Document challenges for Servo embedding community

### Realistic Expectations
- **Expect Failures**: Compilation and runtime issues are normal
- **Frequent Updates**: Servo API changes will break code regularly
- **Incomplete Features**: Many standard web features missing
- **Long Timeline**: Servo may take years to become production-ready

## Comparison with Alternatives

### vs cef_example (Production Solution)
- **CEF**: ✅ Production-ready, reliable, well-documented
- **Servo**: ❌ Experimental, unstable, complex API
- **Recommendation**: Use cef_example for real applications

### vs tauri_example (Original)
- **Tauri**: ❌ WebGL broken on Linux+NVIDIA
- **Servo**: ❓ Unknown WebGL compatibility, experimental
- **Recommendation**: Use cef_example for reliable graphics

## Future Monitoring

### Servo Milestones to Watch
- Embedding API stabilization (reduced complexity)
- Production-ready status declaration
- Major web platform feature completions
- Successful real-world embedding examples
- Performance benchmarks vs established engines

### Re-evaluation Criteria
- API complexity reduced from ~200 to ~50 lines
- Servo team declares production-ready status
- Multiple successful embedding examples published
- Performance competitive with CEF/WebKitGTK

## Important Notes for Development

### Code Updates
- **Pin Servo Version**: Use specific git commit hash in Cargo.toml
- **Monitor Changes**: Watch Servo releases for API changes
- **Update Frequently**: Code may break with Servo updates
- **Document Issues**: Record compilation and runtime problems

### Testing Strategy
- **Start Simple**: Test basic HTML rendering before graphics
- **Incremental**: Add complexity gradually
- **Document Everything**: Record what works and what doesn't
- **Compare**: Benchmark against cef_example when possible

### Production Deployment
- **Not Recommended**: This is a research project only
- **Use Alternatives**: cef_example for production applications
- **Future Planning**: Consider Servo when it matures (2-3+ years)

## Bottom Line

**servo_example** is valuable for:
- Learning browser engine architecture
- Exploring pure Rust graphics stacks
- Preparing for potential Servo future
- Contributing to Servo embedding community

**servo_example** is NOT suitable for:
- Production applications
- Reliable graphics rendering
- Time-sensitive projects
- Systems requiring stability

**For production graphics applications, use cef_example.**