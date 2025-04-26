use super::types::{Position, Color};
use crate::Object2d;

#[derive(Clone, Debug)]
pub struct Circle {
    pub(crate) center: Position, // Uses updated Position (f32)
    pub(crate) radius: f32,      // Change to f32
    pub(crate) color: Color,
    pub(crate) border_width: Option<f32>, // Change to f32
    pub(crate) border_color: Option<Color>,
}

impl Circle {
    pub fn new() -> Self {
        Self {
            center: Position::default(),
            radius: 0.0, // Default to 0.0 f32
            color: Color::default(),
            border_width: None,
            border_color: None,
        }
    }

    pub fn center(mut self, x: f32, y: f32) -> Self { // Accept f32
        self.center = Position { x, y };
        self
    }

    pub fn radius(mut self, radius: f32) -> Self { // Accept f32
        self.radius = radius;
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f64) -> Self {
        self.color = Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a,
        };
        self
    }

    // Renamed from inner_border
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

impl Default for Circle {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Object2d> for Circle {
    fn into(self) -> Object2d {
        Object2d::Circle(self)
    }
}
