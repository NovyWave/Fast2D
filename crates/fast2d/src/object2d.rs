mod text;
pub use text::{Text, FontWeight, Family};

mod rectangle;
pub use rectangle::Rectangle;

mod circle;
pub use circle::Circle;

mod line; 
pub use line::Line;

// Enum definition remains here (shared)
#[derive(Debug, Clone)]
pub enum Object2d {
    Text(Text),
    Rectangle(Rectangle),
    Circle(Circle),
    Line(Line),
}
