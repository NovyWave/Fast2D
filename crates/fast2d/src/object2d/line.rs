// Conditionally import lyon Point
#[cfg(not(feature = "canvas"))]
use lyon::math::Point as LyonPoint;
// Import shared Point and Color
use super::types::{Point, Color};
use crate::Object2d;

#[derive(Debug, Clone)]
pub struct Line {
    pub(crate) points: Vec<Point>, // Use shared Point
    pub(crate) width: f32,
    pub(crate) color: Color, // Use shared Color
}

impl Default for Line {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            width: 1.0,
            color: Color::WHITE, // Default to white
        }
    }
}

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    // Modify points to accept slice of tuples
    pub fn points(mut self, points_tuples: &[(f32, f32)]) -> Self {
        self.points = points_tuples.iter().map(|(x, y)| Point::new(*x, *y)).collect();
        self
    }

    // Keep add_point for convenience
    pub fn add_point(mut self, point: Point) -> Self {
        self.points.push(point);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0); // Ensure non-negative
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
}

impl From<Line> for Object2d {
    fn from(line: Line) -> Self {
        Object2d::Line(line)
    }
}
