use std::borrow::Cow;
use crate::backend::Color;
use super::Object2d;

mod family;
pub use family::Family;

/// Represents a text object for 2D rendering.
///
/// Allows customization of position, font, color, size, style, and bounding box.
#[derive(Debug, Clone)]
pub struct Text {
    /// The text content to render.
    pub(crate) text: Cow<'static, str>,
    /// The left (x) position of the text's anchor point.
    pub(crate) left: f32,
    /// The top (y) position of the text's anchor point.
    pub(crate) top: f32,
    /// The font size in logical pixels.
    pub(crate) font_size: f32,
    /// The line height multiplier (relative to font size).
    pub(crate) line_height_multiplier: f32,
    /// The fill color of the text.
    pub(crate) color: Color,
    /// The font family used for rendering.
    pub(crate) family: Family,
    /// The maximum width for text layout.
    pub(crate) width: f32,
    /// The maximum height for text layout.
    pub(crate) height: f32,
    /// Whether the text is italic.
    pub(crate) italic: bool,
    /// The font weight (thickness).
    pub(crate) weight: FontWeight,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: Cow::Borrowed(""),
            left: 0.0,
            top: 0.0,
            font_size: 16.0,
            line_height_multiplier: 1.0,
            color: Color::default(),
            family: Family::SansSerif,
            width: f32::MAX,
            height: f32::MAX,
            italic: false,
            weight: FontWeight::Regular,
        }
    }
}

impl Text {
    /// Creates a new text object with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the text content.
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.text = text.into();
        self
    }

    /// Sets the position of the text's anchor point.
    pub fn position(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    /// Sets the font size in logical pixels.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Sets the line height multiplier.
    pub fn line_height(mut self, multiplier: f32) -> Self {
        self.line_height_multiplier = multiplier.max(0.0);
        self
    }

    /// Sets the fill color of the text.
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    /// Sets the font family.
    pub fn family(mut self, family: Family) -> Self {
        self.family = family;
        self
    }

    /// Sets the bounding box for text layout.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets whether the text is italic.
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Sets the font weight.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }
}

/// Converts a Text object into an Object2d.
impl From<Text> for Object2d {
    fn from(text: Text) -> Self {
        Object2d::Text(text)
    }
}

/// Represents the weight (thickness) of a font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    /// Thin (100)
    Thin,
    /// Extra Light (200)
    ExtraLight,
    /// Light (300)
    Light,
    /// Regular (400)
    Regular,
    /// Medium (500)
    Medium,
    /// Semi Bold (600)
    SemiBold,
    /// Bold (700)
    Bold,
    /// Extra Bold (800)
    ExtraBold,
    /// Black (900)
    Black,
}
