use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Family {
    Name(Cow<'static, str>),
    SansSerif,
    Serif,
    Monospace,
    Cursive,
    Fantasy,
}

impl Family {
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
    pub fn name<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Family::Name(s.into())
    }

}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
