use iced_fonts::Bootstrap;

pub struct HelpTopic {
    pub id: String,
    pub icon: Bootstrap,
    pub title_vi: &'static str,
    pub title_en: &'static str,
    pub description_vi: &'static str,
    pub description_en: &'static str,
    pub detail_vi: Option<&'static str>,
    pub detail_en: Option<&'static str>,
}
