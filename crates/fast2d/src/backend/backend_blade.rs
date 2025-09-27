pub use canvas_wrapper::CanvasWrapper;
pub use color::Color;
pub use register_fonts::register_fonts;

mod canvas_wrapper;
mod color;
mod register_fonts;

use std::sync::{Mutex, OnceLock};

pub static FONT_SYSTEM: OnceLock<Mutex<glyphon::FontSystem>> = OnceLock::new();
