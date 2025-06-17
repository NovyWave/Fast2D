pub use canvas_wrapper::CanvasWrapper;
pub use register_fonts::register_fonts;
pub use color::Color;

mod color;
mod canvas_wrapper;
mod register_fonts;

use std::sync::{OnceLock, Mutex};

pub static FONT_SYSTEM: OnceLock<Mutex<glyphon::FontSystem>> = OnceLock::new();