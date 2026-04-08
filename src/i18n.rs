use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguage {
    #[default]
    English,
    Vietnamese,
}

impl AppLanguage {
    pub const ALL: [Self; 2] = [Self::Vietnamese, Self::English];

    fn as_u8(self) -> u8 {
        match self {
            Self::Vietnamese => 0,
            Self::English => 1,
        }
    }

    fn from_u8(raw: u8) -> Self {
        match raw {
            1 => Self::English,
            _ => Self::Vietnamese,
        }
    }
}

impl fmt::Display for AppLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Vietnamese => "Tiếng Việt",
            Self::English => "English",
        };
        f.write_str(label)
    }
}

static CURRENT_LANGUAGE: AtomicU8 = AtomicU8::new(1);

pub fn set_current_language(language: AppLanguage) {
    CURRENT_LANGUAGE.store(language.as_u8(), Ordering::Relaxed);
}

pub fn current_language() -> AppLanguage {
    AppLanguage::from_u8(CURRENT_LANGUAGE.load(Ordering::Relaxed))
}

pub fn t<'a>(vi: &'a str, en: &'a str) -> &'a str {
    match current_language() {
        AppLanguage::Vietnamese => vi,
        AppLanguage::English => en,
    }
}
