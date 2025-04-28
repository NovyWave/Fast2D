// --- Compile-time checks for mutually exclusive features ---

// Error if more than one rendering backend is selected
#[cfg(any(
    all(feature = "webgl", feature = "webgpu"),
    all(feature = "webgl", feature = "canvas"),
    all(feature = "webgpu", feature = "canvas")
))]
compile_error!("Only one rendering backend feature ('webgl', 'webgpu', or 'canvas') can be enabled at a time.");

// Error if no rendering backend is selected
#[cfg(not(any(feature = "webgl", feature = "webgpu", feature = "canvas")))]
compile_error!("One rendering backend feature ('webgl', 'webgpu', or 'canvas') must be enabled.");

// --- End of compile-time checks ---

mod backend;
pub use backend::{register_fonts, CanvasWrapper};

mod fetch_file;
pub use fetch_file::fetch_file;

pub mod object2d;
pub use object2d::*;

// --- Conditional Imports ---
// WGPU/WebGL backend moved to backends/backend_wgpu.rs
#[cfg(any(feature = "webgl", feature = "webgpu"))]
pub use crate::backend::{
    Graphics, create_graphics, draw_wgpu, FONT_SYSTEM, FontSystemInitError, CanvasUniforms, ColoredVertex, font_weight_to_glyphon
};





