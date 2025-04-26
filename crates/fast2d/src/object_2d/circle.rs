use super::types::{Point, Color};
use crate::Object2d;

#[derive(Clone, Debug)]
pub struct Circle {
    pub(crate) center: Point,
    pub(crate) radius: f32,
    pub(crate) color: Color,
    pub(crate) border_width: Option<f32>,
    pub(crate) border_color: Option<Color>,
}

impl Circle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn center(mut self, x: f32, y: f32) -> Self {
        self.center = Point::new(x, y);
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    // Modify color to accept f32 alpha
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::from_u8(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8);
        self
    }

    pub fn color_f32(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    // Modify border to accept f32 alpha
    pub fn border(mut self, width: f32, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.border_width = Some(width.max(0.0));
        self.border_color = Some(Color::from_u8(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8));
        self
    }

    pub fn border_f32(mut self, width: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.border_width = Some(width.max(0.0));
        self.border_color = Some(Color::new(r, g, b, a));
        self
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            center: Point::default(),
            radius: 0.0,
            color: Color::WHITE,
            border_width: None,
            border_color: None,
        }
    }
}

impl Into<Object2d> for Circle {
    fn into(self) -> Object2d {
        Object2d::Circle(self)
    }
}
