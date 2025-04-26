use std::borrow::Cow;
// Conditionally import glyphon types
#[cfg(not(feature = "canvas"))]
use glyphon::{FamilyOwned, Metrics, TextBounds};
// Import shared Color type
use super::types::Color;

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) text: Cow<'static, str>,
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) font_size: f32,
    pub(crate) line_height_multiplier: f32,
    pub(crate) color: Color, // Use shared Color type
    #[cfg(not(feature = "canvas"))] // Conditionally compile FamilyOwned
    pub(crate) family: FamilyOwned,
    #[cfg(feature = "canvas")] // Placeholder for canvas
    pub(crate) family: String, // Store family name as String for canvas
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Default for Text {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "canvas"))] {
                Self {
                    text: Cow::Borrowed(""),
                    left: 0.0,
                    top: 0.0,
                    font_size: 16.0,
                    line_height_multiplier: 1.0,
                    color: Color::WHITE, // Use shared Color default
                    family: FamilyOwned::SansSerif, // Use glyphon default
                    width: f32::MAX,
                    height: f32::MAX,
                }
            } else {
                 Self {
                    text: Cow::Borrowed(""),
                    left: 0.0,
                    top: 0.0,
                    font_size: 16.0,
                    line_height_multiplier: 1.0,
                    color: Color::WHITE, // Use shared Color default
                    family: "sans-serif".to_string(), // Use string default for canvas
                    width: f32::MAX,
                    height: f32::MAX,
                }
            }
        }
    }
}

impl Text {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.text = text.into();
        self
    }

    pub fn position(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn line_height(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier.max(0.0); // Ensure non-negative
        self
    }

    // Update color method to use shared Color::from_u8
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::from_u8(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8);
        self
    }

    // Update color method to use shared Color::new (f32 version)
    pub fn color_f32(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    #[cfg(not(feature = "canvas"))]
    pub fn family(mut self, family: FamilyOwned) -> Self {
        self.family = family;
        self
    }

    // Add a family method for canvas that takes a string
    #[cfg(feature = "canvas")]
    pub fn family(mut self, family_name: impl Into<String>) -> Self {
        self.family = family_name.into();
        // Basic validation/mapping could be added here if desired
        // e.g., map "serif" to standard CSS "serif"
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl From<Text> for crate::Object2d {
    fn from(text: Text) -> Self {
        crate::Object2d::Text(text)
    }
}
