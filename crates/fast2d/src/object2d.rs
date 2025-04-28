#![allow(unused_imports)] // Allow unused imports in this module definition file

pub mod types; // Declare the new types module
pub mod text;
pub mod rectangle;
pub mod circle;
pub mod line; // Add line module

pub use circle::Circle;
pub use rectangle::Rectangle;
pub use text::Text;
pub use line::Line; // Re-export Line

// Re-export the shared types for easier use
pub use types::{Color, Point, Size, BorderRadii};
#[cfg(not(feature = "canvas"))]
pub use types::FamilyOwned; // Conditionally re-export FamilyOwned
