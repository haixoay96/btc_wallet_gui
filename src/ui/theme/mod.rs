// Theme module - chỉ import và export
// Cấu trúc: structure.rs (structs/enums) → colors.rs (palettes) → palette.rs (theme logic) → styles (UI)

mod button_styles;
mod colors;
mod container_styles;
mod input_styles;
mod palette;
mod structure;
mod text_styles;

// Export từ structure
pub use structure::{Colors, NoticeTone};

// Export từ palette
pub use palette::{get_theme_colors, set_font_scale, set_high_contrast};

// Export button styles
pub use button_styles::{
    danger_button_style, gradient_button_style, info_style, muted_button_style,
    primary_button_style, secondary_button_style, selected_button_style, warning_style,
};

// Export container styles
pub use container_styles::{
    card_style, notice_style, popup_dialog_style, popup_overlay_style, screen_background_style,
    sidebar_style,
};

// Export input styles
pub use input_styles::{input_style, pick_list_menu_style, pick_list_style};

// Export text styles
pub use text_styles::{
    text_color, text_muted_color, text_primary_color, text_scaled, text_secondary_color,
};
