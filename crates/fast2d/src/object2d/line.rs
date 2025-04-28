use crate::backend::{Point, Color};
use super::Object2d;

#[derive(Debug, Clone)]
pub struct Line {
    pub(crate) points: Vec<Point>,
    pub(crate) width: f32,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn points(mut self, points_tuples: &[(f32, f32)]) -> Self {
        self.points = points_tuples.iter().copied().map(|(x, y)| Point { x, y }).collect();
        self
    }

    pub fn add_point(mut self, point: Point) -> Self {
        self.points.push(point);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0); // Ensure non-negative
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8, a: f32) -> Self {
        self.color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a);
        self
    }
}

impl From<Line> for Object2d {
    fn from(line: Line) -> Self {
        Object2d::Line(line)
    }
}
