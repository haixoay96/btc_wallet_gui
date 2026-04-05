use iced::{
    widget::{column, container, row, text, Space},
    Color, Element, Length,
};

use crate::theme::{card_style, text_color, Colors};
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

    pub fn progress(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Weak => 0.25,
            Self::Medium => 0.5,
            Self::Strong => 0.75,
            Self::VeryStrong => 1.0,
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

    let bar = container(
        row![container(Space::with_width(Length::Fill))
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
                border: iced::border::rounded(4),
                ..Default::default()
            })
            .width(Length::Fill)]
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.1))),
        border: iced::border::rounded(4),
        ..Default::default()
    });

    let filled_bar = container(Space::with_width(Length::Fill))
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(bar_color)),
            border: iced::border::rounded(4),
            ..Default::default()
        })
        .width(Length::FillPortion((progress * 10.0).round() as u16))
        .height(6);

    let mut content = column![container(filled_bar).width(Length::Fill)];

    if show_label && strength != PassphraseStrength::None {
        let label_row = row![
            text(strength.icon())
                .size(12)
                .font(BOOTSTRAP_FONT)
                .style(text_color(strength.color())),
            Space::with_width(4),
            text(strength.label())
                .size(11)
                .style(text_color(strength.color())),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        content = content.push(Space::with_height(4));
        content = content.push(label_row);
    }

    container(content)
        .width(Length::Fill)
        .into()
}

/// Render strength requirements checklist
pub fn requirements_checklist(passphrase: &str) -> Element<'static, ()> {
    let len = passphrase.len();
    let has_upper = passphrase.chars().any(|c| c.is_uppercase());
    let has_lower = passphrase.chars().any(|c| c.is_lowercase());
    let has_digit = passphrase.chars().any(|c| c.is_ascii_digit());
    let has_special = passphrase.chars().any(|c| !c.is_alphanumeric());

    let requirements = vec![
        (len >= 8, "Ít nhất 8 ký tự"),
        (len >= 12, "Nên có 12+ ký tự"),
        (has_upper, "Có chữ in hoa"),
        (has_lower, "Có chữ thường"),
        (has_digit, "Có số"),
        (has_special, "Có ký tự đặc biệt (!@#$...)"),
    ];

    let mut items = column![];
    for (met, label) in requirements {
        let icon = if met {
            Bootstrap::CheckCircle.to_string()
        } else {
            Bootstrap::XCircle.to_string()
        };

        let icon_color = if met { Colors::SUCCESS } else { Colors::TEXT_MUTED };

        let item = row![
            text(icon)
                .size(11)
                .font(BOOTSTRAP_FONT)
                .style(text_color(icon_color)),
            Space::with_width(6),
            text(label).size(11).style(text_color(if met {
                Colors::SUCCESS
            } else {
                Colors::TEXT_MUTED
            })),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        items = items.push(item);
    }

    container(items)
        .style(card_style())
        .padding(10)
        .width(Length::Fill)
        .into()
}
