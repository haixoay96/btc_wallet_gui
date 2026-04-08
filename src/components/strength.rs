use iced::{
    widget::{column, container, row, Space},
    Color, Element, Length,
};

use crate::i18n::t;
use crate::theme::{text_scaled, text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

/// Passphrase strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassphraseStrength {
    None,
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl PassphraseStrength {
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Weak => "Yếu",
            Self::Medium => "Trung bình",
            Self::Strong => "Mạnh",
            Self::VeryStrong => "Rất mạnh",
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Weak => "Weak",
            Self::Medium => "Medium",
            Self::Strong => "Strong",
            Self::VeryStrong => "Very Strong",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::None => Colors::TEXT_MUTED,
            Self::Weak => Colors::ERROR,
            Self::Medium => Colors::WARNING,
            Self::Strong => Color::from_rgb8(0xF5, 0x9E, 0x0B),
            Self::VeryStrong => Colors::SUCCESS,
        }
    }

    pub fn progress(&self) -> u16 {
        match self {
            Self::None => 0,
            Self::Weak => 1,
            Self::Medium => 2,
            Self::Strong => 3,
            Self::VeryStrong => 4,
        }
    }

    pub fn icon(&self) -> String {
        match self {
            Self::None | Self::Weak => Bootstrap::XCircle,
            Self::Medium => Bootstrap::ExclamationTriangle,
            Self::Strong | Self::VeryStrong => Bootstrap::CheckCircle,
        }
        .to_string()
    }
}

/// Calculate passphrase strength
pub fn calculate_strength(passphrase: &str) -> PassphraseStrength {
    if passphrase.is_empty() {
        return PassphraseStrength::None;
    }

    let len = passphrase.len();
    let has_upper = passphrase.chars().any(|c| c.is_uppercase());
    let has_lower = passphrase.chars().any(|c| c.is_lowercase());
    let has_digit = passphrase.chars().any(|c| c.is_ascii_digit());
    let has_special = passphrase.chars().any(|c| !c.is_alphanumeric());

    let score = [
        (len >= 8) as u8,
        (len >= 12) as u8,
        (len >= 16) as u8,
        has_upper as u8,
        has_lower as u8,
        has_digit as u8,
        has_special as u8,
    ]
    .into_iter()
    .sum::<u8>();

    match score {
        0..=2 => PassphraseStrength::Weak,
        3..=4 => PassphraseStrength::Medium,
        5..=6 => PassphraseStrength::Strong,
        _ => PassphraseStrength::VeryStrong,
    }
}

/// Render strength indicator bar
pub fn strength_bar(strength: PassphraseStrength, show_label: bool) -> Element<'static, ()> {
    let bar_color = strength.color();
    let progress = strength.progress();

    let filled_bar = container(Space::with_width(Length::Fill))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(bar_color)),
            border: iced::border::rounded(3),
            ..Default::default()
        })
        .width(iced::Length::FillPortion(if progress == 0 { 1 } else { progress }))
        .height(6);

    let empty_bar = container(Space::with_width(Length::Fill))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.15))),
            border: iced::border::rounded(3),
            ..Default::default()
        })
        .width(iced::Length::FillPortion(if progress == 4 { 1 } else { 4 - progress }))
        .height(6);

    let mut content = column![
        row![
            filled_bar,
            if progress < 4 { empty_bar } else { 
                container(Space::with_width(0)).into() 
            }
        ]
        .spacing(2)
    ];

    if show_label && strength != PassphraseStrength::None {
        let label = strength.label();
        let label_en = strength.label_en();
        let icon = strength.icon();
        let color = strength.color();
        
        let label_row = row![
            text_scaled(icon, 11)
                .font(BOOTSTRAP_FONT)
                .style(text_color(color)),
            Space::with_width(4),
            text_scaled(t(label, label_en), 11)
                .style(text_color(color)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        content = content.push(Space::with_height(3));
        content = content.push(label_row);
    }

    container(content)
        .width(Length::Fill)
        .into()
}
