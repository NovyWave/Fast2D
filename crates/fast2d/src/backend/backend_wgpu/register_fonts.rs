use std::sync::Mutex;
use glyphon::FontSystem;
use crate::backend::{FONT_SYSTEM, RegisterFontsError};
use web_sys::wasm_bindgen::UnwrapThrowExt;

/// Registers fonts for the WGPU backend.
///
/// You can call this function multiple times to add more fonts.
///
/// # Arguments
/// * `fonts` - Font data as a Vec of Vec<u8> (e.g., TTF or OTF).
///
/// # Returns
/// * `Ok(())` if at least one valid font is loaded or added.
/// * `Err(RegisterFontsError)` if no valid font is loaded, or if no fonts are provided.
pub fn register_fonts(fonts: Vec<Vec<u8>>) -> Result<(), RegisterFontsError> {
    if fonts.is_empty() {
        return Err(RegisterFontsError::NoFontsProvided);
    }

    // If already initialized, just add new fonts
    if let Some(font_system_mutex) = FONT_SYSTEM.get() {
        let mut font_system = font_system_mutex.lock().unwrap_throw();
        let db = font_system.db_mut();
        for font_data in fonts {
            db.load_font_data(font_data);
        }
        if db.faces().next().is_none() {
            return Err(RegisterFontsError::NoValidFontLoaded);
        }
        return Ok(());
    }

    // Not initialized yet: create and load
    let mut font_system = FontSystem::new();
    let db = font_system.db_mut();
    for font_data in fonts {
        db.load_font_data(font_data);
    }
    if db.faces().next().is_none() {
        return Err(RegisterFontsError::NoValidFontLoaded);
    }
    FONT_SYSTEM.set(Mutex::new(font_system)).unwrap_throw();
    Ok(())
}
