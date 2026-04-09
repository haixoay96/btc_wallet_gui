use super::structure::LanguageSelector;

use crate::i18n::{current_language, AppLanguage};
use crate::ui::theme::{pick_list_menu_style, pick_list_style};
use iced::{widget::pick_list, Element, Length};

const APP_LANGUAGES: [AppLanguage; 2] = AppLanguage::ALL;

impl LanguageSelector {
    pub fn new() -> Self {
        Self
    }

    pub fn view<Message: Clone + 'static>(
        &self,
        on_change: impl Fn(AppLanguage) -> Message + 'static,
    ) -> Element<'_, Message> {
        pick_list(APP_LANGUAGES, Some(current_language()), on_change)
            .width(Length::Shrink)
            .padding(6)
            .text_size(13)
            .style(pick_list_style())
            .menu_style(pick_list_menu_style())
            .into()
    }
}
