use crate::backend::{Point, Color};
use super::Object2d;

/// A line shape defined by a sequence of points.
#[derive(Debug, Clone)]
pub struct Line {
    /// The points that define the line path.
    pub(crate) points: Vec<Point>,
    /// The width of the line.
    pub(crate) width: f32,
    /// The color of the line.
    pub(crate) color: Color,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            width: 1.0,
            color: Color::default(),
        }
    }
}

impl Line {
    /// Creates a new line with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the points of the line from a slice of (x, y) tuples.
    pub fn points(mut self, points_tuples: &[(f32, f32)]) -> Self {
        self.points = points_tuples.iter().copied().map(|(x, y)| Point { x, y }).collect();
        self
    }

    /// Sets the width of the line.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self
    }

    /// Sets the color of the line.
    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r, g, b, a);
        self
    }
}

/// Converts a Line into an Object2d.
impl From<Line> for Object2d {
    fn from(line: Line) -> Self {
        Object2d::Line(line)
    }
}
