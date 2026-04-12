use iced::Color;

/// Available tag colors
pub const TAG_COLORS: &[Color] = &[
    Color::from_rgb(0.48, 0.38, 1.0),  // Purple
    Color::from_rgb(0.0, 0.83, 0.67),  // Teal
    Color::from_rgb(0.4, 0.7, 1.0),    // Blue
    Color::from_rgb(1.0, 0.55, 0.0),   // Orange
    Color::from_rgb(1.0, 0.35, 0.35),  // Red
    Color::from_rgb(0.85, 0.25, 0.75), // Pink
    Color::from_rgb(0.4, 0.85, 0.5),   // Green
    Color::from_rgb(0.95, 0.75, 0.0),  // Yellow
];

/// Common tag suggestions
pub const COMMON_TAGS: &[&str] = &[
    "Personal",
    "Business",
    "Savings",
    "Trading",
    "Investment",
    "Hot Wallet",
    "Cold Storage",
    "Test",
];

/// Message for tag picker interactions
#[derive(Debug, Clone)]
pub enum TagMessage {
    InputChanged(String),
    SelectTag(String),
    RemoveTag(String),
    CreateTag(String),
}
