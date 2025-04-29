use std::borrow::Cow;

/// Represents a font family for text rendering.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Family {
    /// A named font family (e.g., "Arial").
    Name(Cow<'static, str>),
    /// A generic sans-serif font family.
    SansSerif,
    /// A generic serif font family.
    Serif,
    /// A generic monospace font family.
    Monospace,
    /// A generic cursive font family.
    Cursive,
    /// A generic fantasy font family.
    Fantasy,
}

impl Family {
    /// Returns the CSS string representation of the font family.
    pub fn as_str(&self) -> &str {
        match self {
            Family::Name(name) => name.as_ref(),
            Family::SansSerif => "sans-serif",
            Family::Serif => "serif",
            Family::Monospace => "monospace",
            Family::Cursive => "cursive",
            Family::Fantasy => "fantasy",
        }
    }
    /// Creates a named font family from a string.
    pub fn name<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Family::Name(s.into())
    }

}

/// Formats the font family as a string for display purposes.
impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
