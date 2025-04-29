#[cfg(any(
    all(feature = "webgl", feature = "webgpu"),
    all(feature = "webgl", feature = "canvas"),
    all(feature = "webgpu", feature = "canvas")
))]
compile_error!("Only one rendering backend feature ('webgl', 'webgpu', or 'canvas') can be enabled at a time.");

#[cfg(not(any(feature = "webgl", feature = "webgpu", feature = "canvas")))]
compile_error!("One rendering backend feature ('webgl', 'webgpu', or 'canvas') must be enabled.");

mod backend;
pub use backend::{register_fonts, CanvasWrapper, RegisterFontsError};

mod fetch_file;
pub use fetch_file::{fetch_file, FetchFileError};

pub mod object2d;
pub use object2d::*;
