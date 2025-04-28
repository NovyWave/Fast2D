#[cfg(not(feature = "canvas"))]
use std::sync::Mutex;
#[cfg(not(feature = "canvas"))]
use glyphon::FontSystem;
#[cfg(not(feature = "canvas"))]
use web_sys::wasm_bindgen::JsValue;
#[cfg(feature = "canvas")]
use web_sys::{window, FontFace, FontFaceDescriptors};
#[cfg(feature = "canvas")]
use ttf_parser::{Face, name_id};
#[cfg(not(feature = "canvas"))]
use crate::backends::backend_wgpu::{FONT_SYSTEM, FontSystemInitError};

#[cfg(any(feature = "webgl", feature = "webgpu"))]
/// Registers font data for use in Fast2D text rendering (WebGL/WebGPU only).
/// This should be called before any text rendering, and before creating canvases.
/// On backends that do not require explicit font registration, this function is not available.
pub fn register_fonts(fonts: &[Vec<u8>]) -> Result<(), FontSystemInitError> {
    if fonts.is_empty() {
        return Err(FontSystemInitError::NoFontsProvided);
    }
    let mut font_system = FontSystem::new();
    let db = font_system.db_mut();
    for data in fonts {
        db.load_font_data(data.clone());
    }
    // Validate that a default font is available
    if db.faces().next().is_none() {
        #[cfg(target_arch = "wasm32")]
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

#[cfg(feature = "canvas")]
use web_sys::wasm_bindgen::JsValue;

#[cfg(feature = "canvas")]
pub fn register_fonts(fonts: &[Vec<u8>]) -> Result<(), String> {
    let win = window().ok_or("No window")?;
    let doc = win.document().ok_or("No document")?;
    let fonts_set = doc.fonts();
    for font_bytes in fonts {
        let face = Face::parse(font_bytes, 0).map_err(|_| "Failed to parse font data")?;
        // Extract family, weight, and style
        let mut family = None;
        let mut weight = None;
        let mut style = None;
        for name in face.names() {
            if name.name_id == name_id::FAMILY && family.is_none() {
                family = name.to_string();
            }
            if name.name_id == name_id::SUBFAMILY && style.is_none() {
                let subfamily = name.to_string().unwrap_or_default().to_lowercase();
                if subfamily.contains("italic") {
                    style = Some("italic");
                } else {
                    style = Some("normal");
                }
                if subfamily.contains("bold") {
                    weight = Some("bold");
                } else if subfamily.contains("light") {
                    weight = Some("300");
                } else if subfamily.contains("medium") {
                    weight = Some("500");
                } else if subfamily.contains("semibold") {
                    weight = Some("600");
                } else if subfamily.contains("black") {
                    weight = Some("900");
                } else {
                    weight = Some("400");
                }
            }
        }
        let family = family.unwrap_or_else(|| {
            web_sys::console::warn_1(&JsValue::from_str("Warning: Could not extract font family name from font data. Using 'CustomFont'."));
            "CustomFont".to_string()
        });
        let style = style.unwrap_or("normal");
        let weight = weight.unwrap_or("400");
        let buffer = web_sys::js_sys::Uint8Array::from(font_bytes.as_slice());
        let array_buffer = buffer.buffer();
        let descriptors = FontFaceDescriptors::new();
        descriptors.set_style(style);
        descriptors.set_weight(weight);
        let font_face = FontFace::new_with_array_buffer_and_descriptors(&family, &array_buffer, &descriptors)
            .map_err(|e| format!("FontFace error: {:?}", e))?;
        fonts_set.add(&font_face).map_err(|e| format!("Add font error: {:?}", e))?;
    }
    Ok(())
}
