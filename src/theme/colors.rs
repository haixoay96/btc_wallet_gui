use iced::Color;

/// Dark mode color palette - inspired by Exodus wallet
pub struct DarkColors;

impl DarkColors {
    pub const BG_PRIMARY: Color = Color::from_rgb(0.09, 0.09, 0.14);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.12, 0.12, 0.18);
    pub const BG_CARD: Color = Color::from_rgb(0.15, 0.15, 0.22);
    pub const BG_INPUT: Color = Color::from_rgb(0.18, 0.18, 0.26);
    pub const BG_HOVER: Color = Color::from_rgb(0.22, 0.22, 0.32);

    pub const ACCENT_PURPLE: Color = Color::from_rgb(0.48, 0.38, 1.0);
    pub const ACCENT_TEAL: Color = Color::from_rgb(0.0, 0.83, 0.67);
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.4, 0.7, 1.0);

    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.95);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.65, 0.65, 0.75);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.45, 0.45, 0.55);

    pub const SUCCESS: Color = Color::from_rgb(0.0, 0.83, 0.67);
    pub const ERROR: Color = Color::from_rgb(1.0, 0.35, 0.35);
    pub const WARNING: Color = Color::from_rgb(1.0, 0.75, 0.0);
    pub const CONFIRMED_LOW: Color = Color::from_rgb(1.0, 0.6, 0.0);
    pub const CONFIRMED_PARTIAL: Color = Color::from_rgb(0.4, 0.85, 0.5);

    pub const GRADIENT_START: Color = Color::from_rgb(0.48, 0.38, 1.0);

    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.35);
    pub const BORDER_FOCUSED: Color = Color::from_rgb(0.48, 0.38, 1.0);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.20, 0.20, 0.28);
}

/// High contrast mode color palette
pub struct HighContrastColors;

impl HighContrastColors {
    pub const BG_PRIMARY: Color = Color::from_rgb(0.0, 0.0, 0.0);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.10, 0.10, 0.10);
    pub const BG_CARD: Color = Color::from_rgb(0.15, 0.15, 0.15);
    pub const BG_INPUT: Color = Color::from_rgb(0.20, 0.20, 0.20);
    pub const BG_HOVER: Color = Color::from_rgb(0.30, 0.30, 0.30);

    pub const ACCENT_PURPLE: Color = Color::from_rgb(0.60, 0.50, 1.0);
    pub const ACCENT_TEAL: Color = Color::from_rgb(0.0, 1.0, 0.80);
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.50, 0.80, 1.0);

    pub const TEXT_PRIMARY: Color = Color::from_rgb(1.0, 1.0, 1.0);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.85, 0.85, 0.85);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.70, 0.70, 0.70);

    pub const SUCCESS: Color = Color::from_rgb(0.0, 1.0, 0.0);
    pub const ERROR: Color = Color::from_rgb(1.0, 0.0, 0.0);
    pub const WARNING: Color = Color::from_rgb(1.0, 1.0, 0.0);

    pub const GRADIENT_START: Color = Color::from_rgb(0.60, 0.50, 1.0);

    pub const BORDER: Color = Color::from_rgb(0.60, 0.60, 0.60);
    pub const BORDER_FOCUSED: Color = Color::from_rgb(1.0, 1.0, 1.0);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.40, 0.40, 0.40);
}
