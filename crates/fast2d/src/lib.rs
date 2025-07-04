#[cfg(any(
    all(feature = "webgl", feature = "webgpu"),
    all(feature = "webgl", feature = "canvas"),
    all(feature = "webgl", feature = "native"),
    all(feature = "webgl", feature = "webgpu-blade"),
    all(feature = "webgpu", feature = "canvas"),
    all(feature = "webgpu", feature = "native"),
    all(feature = "webgpu", feature = "webgpu-blade"),
    all(feature = "canvas", feature = "native"),
    all(feature = "canvas", feature = "webgpu-blade"),
    all(feature = "native", feature = "webgpu-blade")
))]
compile_error!("Only one rendering backend feature ('webgl', 'webgpu', 'webgpu-blade', 'canvas', or 'native') can be enabled at a time.");

#[cfg(not(any(feature = "webgl", feature = "webgpu", feature = "webgpu-blade", feature = "canvas", feature = "native")))]
compile_error!("One rendering backend feature ('webgl', 'webgpu', 'webgpu-blade', 'canvas', or 'native') must be enabled.");

mod backend;
pub use backend::{register_fonts, CanvasWrapper, RegisterFontsError};

#[cfg(feature = "web")]
mod fetch_file;
#[cfg(feature = "web")]
pub use fetch_file::{fetch_file, FetchFileError};

pub mod object2d;
pub use object2d::*;
