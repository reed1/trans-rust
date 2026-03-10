use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Id,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::En => write!(f, "en"),
            Language::Id => write!(f, "id"),
        }
    }
}

pub struct LanguagePair {
    pub from: Language,
    pub to: Language,
}

impl LanguagePair {
    pub fn supported_pairs() -> Vec<LanguagePair> {
        vec![
            LanguagePair { from: Language::En, to: Language::En },
            LanguagePair { from: Language::Id, to: Language::Id },
            LanguagePair { from: Language::En, to: Language::Id },
            LanguagePair { from: Language::Id, to: Language::En },
        ]
    }
}
