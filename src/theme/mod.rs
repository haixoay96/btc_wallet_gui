// Theme module - chỉ import và export
// Cấu trúc: structure.rs (structs/enums) → colors.rs (palettes) → palette.rs (theme logic) → styles (UI)

mod structure;
mod colors;
mod palette;
mod button_styles;
mod container_styles;
mod input_styles;
mod text_styles;

// Export từ structure
pub use structure::{
    NoticeTone,
    color_with_alpha,
    Colors,
    ButtonStyleFn,
    ContainerStyleFn,
    MenuStyleFn,
    PickListStyleFn,
    TextInputStyleFn,
    TextStyleFn,
};

// Export từ colors
pub use colors::{DarkColors, LightColors, HighContrastColors};

// Export từ palette
pub use palette::{ThemeColorPalette, get_theme_colors, set_high_contrast, is_high_contrast, set_font_scale, get_font_scale};

// Export button styles
pub use button_styles::{
    primary_button_style,
    gradient_button_style,
    selected_button_style,
    muted_button_style,
    secondary_button_style,
    info_style,
    warning_style,
    danger_button_style,
};

// Export container styles
pub use container_styles::{
    card_style,
    screen_background_style,
    popup_overlay_style,
    popup_dialog_style,
    notice_style,
    sidebar_style,
    bg_primary_style,
    bg_secondary_style,
};

// Export input styles
pub use input_styles::{
    input_style,
    pick_list_style,
    pick_list_menu_style,
};

// Export text styles
pub use text_styles::{
    text_color,
    text_primary_color,
    text_secondary_color,
    text_muted_color,
    text_scaled,
};
