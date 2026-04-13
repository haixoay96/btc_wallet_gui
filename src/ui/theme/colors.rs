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

    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.35);
    pub const BORDER_FOCUSED: Color = Color::from_rgb(0.48, 0.38, 1.0);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.20, 0.20, 0.28);
}

/// Light mode color palette - Soft Warm Grey (Very easy on eyes, Notion-like)
pub struct LightColors;

impl LightColors {
    // Soft Warm Backgrounds (Lower brightness to reduce glare)
    pub const BG_PRIMARY: Color = Color::from_rgb(0.92, 0.92, 0.93); // #EBEBED
    pub const BG_SECONDARY: Color = Color::from_rgb(0.88, 0.88, 0.90); // #E0E0E5
    pub const BG_CARD: Color = Color::from_rgb(0.96, 0.96, 0.97); // #F5F5F8
    pub const BG_INPUT: Color = Color::from_rgb(0.94, 0.94, 0.95); // #F0F0F2
    pub const BG_HOVER: Color = Color::from_rgb(0.82, 0.82, 0.85);

    // Muted, comfortable accents (Less neon)
    pub const ACCENT_PURPLE: Color = Color::from_rgb(0.45, 0.35, 0.85); // #7259D9
    pub const ACCENT_TEAL: Color = Color::from_rgb(0.05, 0.60, 0.45); // #0D9973
    pub const ACCENT_BLUE: Color = Color::from_rgb(0.30, 0.55, 0.85);

    // Charcoal Text (Softer contrast)
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.30, 0.30, 0.32); // #4D4D52
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.45, 0.45, 0.48); // #73737A
    pub const TEXT_MUTED: Color = Color::from_rgb(0.60, 0.60, 0.64); // #9999A3

    // Muted Status Colors
    pub const SUCCESS: Color = Color::from_rgb(0.15, 0.65, 0.50); // #26A680
    pub const ERROR: Color = Color::from_rgb(0.75, 0.30, 0.30); // #BF4D4D
    pub const WARNING: Color = Color::from_rgb(0.85, 0.55, 0.10); // #D98C1A

    // Subtle borders
    pub const BORDER: Color = Color::from_rgb(0.80, 0.80, 0.83); // #CCCCCCD4
    pub const BORDER_FOCUSED: Color = Color::from_rgb(0.45, 0.35, 0.85);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.88, 0.88, 0.91); // #E0E0E8
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

    pub const BORDER: Color = Color::from_rgb(0.60, 0.60, 0.60);
    pub const BORDER_FOCUSED: Color = Color::from_rgb(1.0, 1.0, 1.0);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.40, 0.40, 0.40);
}
