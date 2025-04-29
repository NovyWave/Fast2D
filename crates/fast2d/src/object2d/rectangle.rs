use crate::backend::{Point, Size, Color, RoundedCorners};
use super::Object2d;

/// A rectangle shape with optional border and rounded corners.
#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    /// The position of the rectangle's top-left corner.
    pub(crate) position: Point,
    /// The size (width and height) of the rectangle.
    pub(crate) size: Size,
    /// The fill color of the rectangle.
    pub(crate) color: Color,
    /// The radii for rounded corners.
    pub(crate) border_radii: RoundedCorners,
    /// The width of the border, if any.
    pub(crate) border_width: Option<f32>,
    /// The color of the border, if any.
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
    /// Creates a new rectangle with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the position of the rectangle's top-left corner.
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Point { x, y };
        self
    }

    /// Sets the size (width and height) of the rectangle.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Size { width, height };
        self
    }

    /// Sets the fill color of the rectangle.
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }

    /// Sets the radii for the rectangle's rounded corners.
    pub fn rounded_corners(mut self, top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self {
        self.border_radii = RoundedCorners {
            top_left: top_left.max(0.0),
            top_right: top_right.max(0.0),
            bottom_left: bottom_left.max(0.0),
            bottom_right: bottom_right.max(0.0),
        };
        self
    }

    /// Sets the border width and color.
    pub fn border(mut self, width: f32, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.border_width = Some(width.max(0.0));
        self.border_color = Some(Color::new(r, g, b, a));
        self
    }
}

/// Converts a Rectangle into an Object2d.
impl From<Rectangle> for Object2d {
    fn from(rect: Rectangle) -> Self {
        Object2d::Rectangle(rect)
    }
}
