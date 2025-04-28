use std::sync::Mutex;
use glyphon::FontSystem;
use web_sys::wasm_bindgen::JsValue;
use crate::backend::backend_wgpu::{FONT_SYSTEM, FontSystemInitError};

pub fn register_fonts(fonts: &[Vec<u8>]) -> Result<(), FontSystemInitError> {
    if fonts.is_empty() {
        return Err(FontSystemInitError::NoFontsProvided);
    }
    let mut font_system = FontSystem::new();
    let db = font_system.db_mut();
    for data in fonts {
        db.load_font_data(data.clone());
    }
    if db.faces().next().is_none() {
        web_sys::console::warn_1(&JsValue::from_str(
            "Warning: No valid font loaded. The chosen font may not be available."
        ));
        return Err(FontSystemInitError::DatabaseError("No valid font loaded".to_string()));
    }
    FONT_SYSTEM.set(Mutex::new(font_system))
        .map_err(|_| {
            web_sys::console::warn_1(&JsValue::from_str("Warning: FontSystem already initialized."));
            FontSystemInitError::AlreadyInitialized
        })
}
