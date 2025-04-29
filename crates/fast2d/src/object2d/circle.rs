use crate::backend::{Point, Color};
use super::Object2d;

/// A circle shape with optional border.
#[derive(Clone, Debug)]
pub struct Circle {
    /// The center point of the circle.
    pub(crate) center: Point,
    /// The radius of the circle.
    pub(crate) radius: f32,
    /// The fill color of the circle.
    pub(crate) color: Color,
    /// The width of the border, if any.
    pub(crate) border_width: Option<f32>,
    /// The color of the border, if any.
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
    /// Creates a new circle with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the center point of the circle.
    pub fn center(mut self, x: f32, y: f32) -> Self {
        self.center = Point { x, y };
        self
    }

    /// Sets the radius of the circle.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    /// Sets the fill color of the circle.
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    /// Sets the border width and color.
    pub fn border(mut self, width: f32, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.border_width = Some(width.max(0.0));
        self.border_color = Some(Color::new(r, g, b, a));
        self
    }
}

/// Converts a Circle into an Object2d.
impl Into<Object2d> for Circle {
    fn into(self) -> Object2d {
        Object2d::Circle(self)
    }
}
