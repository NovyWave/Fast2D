use crate::backend::{Point, Color};
use super::Object2d;

#[derive(Clone, Debug)]
pub struct Circle {
    pub(crate) center: Point,
    pub(crate) radius: f32,
    pub(crate) color: Color,
    pub(crate) border_width: Option<f32>,
    pub(crate) border_color: Option<Color>,
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            center: Point::default(),
            radius: 0.0,
            color: Color::default(),
            border_width: None,
            border_color: None,
        }
    }
}

impl Circle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn center(mut self, x: f32, y: f32) -> Self {
        self.center = Point { x, y };
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a);
        self
    }

    pub fn border(mut self, width: f32, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.border_width = Some(width.max(0.0));
        self.border_color = Some(Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a));
        self
    }
}

impl Into<Object2d> for Circle {
    fn into(self) -> Object2d {
        Object2d::Circle(self)
    }
}
