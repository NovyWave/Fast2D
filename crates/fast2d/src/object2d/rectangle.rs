use crate::backend::{Point, Size, Color, RoundedCorners};
use super::Object2d;

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub(crate) position: Point,
    pub(crate) size: Size,
    pub(crate) color: Color,
    pub(crate) border_radii: RoundedCorners,
    pub(crate) border_width: Option<f32>,
    pub(crate) border_color: Option<Color>,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            position: Point::default(),
            size: Size::default(),
            color: Color::default(),
            border_radii: RoundedCorners::default(),
            border_width: None,
            border_color: None,
        }
    }
}

impl Rectangle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Point { x, y };
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Size { width, height };
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    pub fn rounded_corners(mut self, top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self {
        self.border_radii = RoundedCorners {
            top_left: top_left.max(0.0),
            top_right: top_right.max(0.0),
            bottom_left: bottom_left.max(0.0),
            bottom_right: bottom_right.max(0.0),
        };
        self
    }

    pub fn border(mut self, width: f32, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.border_width = Some(width.max(0.0));
        self.border_color = Some(Color::new(r, g, b, a));
        self
    }
}

impl From<Rectangle> for Object2d {
    fn from(rect: Rectangle) -> Self {
        Object2d::Rectangle(rect)
    }
}
