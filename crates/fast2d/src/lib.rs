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

mod backends;

// --- Conditional Imports ---
// WGPU/WebGL backend moved to backends/backend_wgpu.rs
#[cfg(any(feature = "webgl", feature = "webgpu"))]
pub use crate::backends::backend_wgpu::{
    Graphics, create_graphics, draw_wgpu, FONT_SYSTEM, FontSystemInitError, CanvasUniforms, ColoredVertex, font_weight_to_glyphon
};

// Canvas backend moved to backends/backend_canvas.rs
#[cfg(feature = "canvas")]
pub(crate) use crate::backends::backend_canvas::draw_canvas;

// --- Shared Structs/Enums ---
// Declare the object_2d module and re-export structs (shared)
mod object_2d;
pub use object_2d::text::Text;
pub use object_2d::rectangle::Rectangle;
pub use object_2d::circle::Circle;
pub use object_2d::line::Line;
pub use object_2d::types::{Color, Point, Size, BorderRadii as ObjBorderRadii}; // Re-export shared types
pub use object_2d::types::Family;
pub use crate::object_2d::text::FontWeight;

#[cfg(any(feature = "webgl", feature = "webgpu"))]
pub use object_2d::FamilyOwned; // Re-export conditionally

// Enum definition remains here (shared)
#[derive(Debug, Clone)]
pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Line),
}

// --- CanvasWrapper moved to canvas_wrapper.rs ---
mod canvas_wrapper;
pub use canvas_wrapper::CanvasWrapper;
mod register_fonts;
pub use register_fonts::register_fonts;
mod fetch_file;
pub use fetch_file::fetch_file;
