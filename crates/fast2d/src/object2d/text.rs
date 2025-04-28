use std::borrow::Cow;
use crate::backend::Color;
use super::Object2d;

mod family;
pub use family::Family;

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) text: Cow<'static, str>,
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) font_size: f32,
    pub(crate) line_height_multiplier: f32,
    pub(crate) color: Color,
    pub(crate) family: Family,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) italic: bool,
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
            family: Family::sans_serif(),
            width: f32::MAX,
            height: f32::MAX,
            italic: false,
            weight: FontWeight::Regular,
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
        self.line_height_multiplier = multiplier.max(0.0);
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::from_u8(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8);
        self
    }

    pub fn color_f32(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    pub fn family(mut self, family: Family) -> Self {
        self.family = family;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl From<Text> for Object2d {
    fn from(text: Text) -> Self {
        Object2d::Text(text)
    }
}
