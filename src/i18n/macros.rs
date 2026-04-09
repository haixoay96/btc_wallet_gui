use crate::i18n::structure::AppLanguage;
use std::sync::atomic::{AtomicU8, Ordering};

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
