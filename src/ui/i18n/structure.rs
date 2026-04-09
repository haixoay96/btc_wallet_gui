use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguage {
    #[default]
    English,
    Vietnamese,
}

impl AppLanguage {
    pub const ALL: [Self; 2] = [Self::Vietnamese, Self::English];

    pub fn as_u8(self) -> u8 {
        match self {
            Self::Vietnamese => 0,
            Self::English => 1,
        }
    }

    pub fn from_u8(raw: u8) -> Self {
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
