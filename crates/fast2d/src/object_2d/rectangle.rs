use super::types::{Position, Size, Color, BorderRadii};
use crate::Object2d; // Import Object2d enum

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub(crate) position: Position, // Uses updated Position (f32)
    pub(crate) size: Size,         // Uses updated Size (f32)
    pub(crate) color: Color,
    pub(crate) border_radii: BorderRadii, // Uses updated BorderRadii (f32)
    pub(crate) border_width: Option<f32>, // Change to f32
    pub(crate) border_color: Option<Color>,
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
            position: Position::default(),
            size: Size::default(),
            color: Color::default(),
            border_radii: BorderRadii::default(),
            border_width: None,
            border_color: None,
        }
    }

    pub fn position(mut self, x: f32, y: f32) -> Self { // Accept f32
        self.position = Position { x, y };
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self { // Accept f32
        self.size = Size { width, height };
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self { // Keep f64 for alpha consistency? Or f32?
        self.color = Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a,
        };
        self
    }

    pub fn rounded_corners(mut self, top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self { // Accept f32
        self.border_radii = BorderRadii { top_left, top_right, bottom_left, bottom_right };
        self
    }

    // Renamed from inner_border for clarity
    pub fn border(mut self, width: f32, r: u8, g: u8, b: u8, a: f64) -> Self { // Accept f32 for width
        self.border_width = Some(width);
        self.border_color = Some(Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a,
        });
        self
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Object2d> for Rectangle {
    fn into(self) -> Object2d {
        Object2d::Rectangle(self)
    }
}
