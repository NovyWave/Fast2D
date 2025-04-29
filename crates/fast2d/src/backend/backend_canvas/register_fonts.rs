use web_sys::{window, FontFace, FontFaceDescriptors};
use crate::backend::RegisterFontsError;
use ttf_parser::{Face, name_id};

/// Registers fonts for the Canvas backend.
///
/// You can call this fuction multiple times to add more fonts.
///
/// # Arguments
/// * `fonts` - A slice of font data, each as a Vec<u8> (e.g., TTF or OTF).
///
/// # Returns
/// * `Ok(())` if at least one valid font is loaded or added.
/// * `Err(RegisterFontsError)` if no valid font is loaded, no fonts are provided, or browser APIs are unavailable.
pub fn register_fonts(fonts: &[Vec<u8>]) -> Result<(), RegisterFontsError> {
    if fonts.is_empty() {
        return Err(RegisterFontsError::NoFontsProvided);
    }

    let window = window().ok_or(RegisterFontsError::NoWindow)?;
    let document = window.document().ok_or(RegisterFontsError::NoDocument)?;
    let font_face_set = document.fonts();
    let mut any_loaded = false;

    for font_bytes in fonts {
        let face = Face::parse(font_bytes, 0)
            .map_err(|_| RegisterFontsError::FontParseFailed)?;

        let mut family = None;
        let mut weight = None;
        let mut style = None;
        for name in face.names() {
            if name.name_id == name_id::FAMILY && family.is_none() {
                family = name.to_string();
            }
            if name.name_id == name_id::SUBFAMILY && style.is_none() {
                let subfamily = name.to_string().unwrap_or_default().to_lowercase();
                style = Some(if subfamily.contains("italic") { "italic" } else { "normal" });
                weight = Some(
                    if subfamily.contains("bold") {
                        "bold"
                    } else if subfamily.contains("light") {
                        "300"
                    } else if subfamily.contains("medium") {
                        "500"
                    } else if subfamily.contains("semibold") {
                        "600"
                    } else if subfamily.contains("black") {
                        "900"
                    } else {
                        "400"
                    }
                );
            }
        }
        let family = family.ok_or(RegisterFontsError::FontParseFailed)?;
        let style = style.ok_or(RegisterFontsError::FontParseFailed)?;
        let weight = weight.ok_or(RegisterFontsError::FontParseFailed)?;

        let buffer = web_sys::js_sys::Uint8Array::from(font_bytes.as_slice());
        let array_buffer = buffer.buffer();
        let descriptors = FontFaceDescriptors::new();
        descriptors.set_style(style);
        descriptors.set_weight(weight);
        let font_face = FontFace::new_with_array_buffer_and_descriptors(&family, &array_buffer, &descriptors)
            .map_err(|error| RegisterFontsError::FontFaceError(format!("{:?}", error)))?;
        font_face_set.add(&font_face)
            .map_err(|error| RegisterFontsError::AddFontError(format!("{:?}", error)))?;
        any_loaded = true;
    }

    if !any_loaded && font_face_set.size() == 0 {
        return Err(RegisterFontsError::NoValidFontLoaded);
    }
    Ok(())
}
