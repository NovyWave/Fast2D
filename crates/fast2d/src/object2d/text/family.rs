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
    pub fn sans_serif() -> Self { Family::SansSerif }
    pub fn serif() -> Self { Family::Serif }
    pub fn monospace() -> Self { Family::Monospace }
    pub fn cursive() -> Self { Family::Cursive }
    pub fn fantasy() -> Self { Family::Fantasy }
}

impl From<&'static str> for Family {
    fn from(s: &'static str) -> Self {
        match s {
            "sans-serif" => Family::SansSerif,
            "serif" => Family::Serif,
            "monospace" => Family::Monospace,
            "cursive" => Family::Cursive,
            "fantasy" => Family::Fantasy,
            _ => Family::Name(Cow::Borrowed(s)),
        }
    }
}

impl From<String> for Family {
    fn from(s: String) -> Self {
        Family::Name(Cow::Owned(s))
    }
}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
