use iced::{
    widget::{button, column, container, mouse_area, row, stack, text, Space},
    Alignment, Element, Length,
};

use crate::ui::theme::{
    get_theme_colors, popup_dialog_style, popup_overlay_style, text_primary_color,
    text_secondary_color,
};

use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

/// Subtle close button style for modals
fn close_button_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    |theme: &iced::Theme, status: button::Status| {
        let colors = get_theme_colors(theme);
        let (background, text_color) = match status {
            button::Status::Hovered => (
                Some(iced::Background::Color(iced::Color::from_rgba(
                    colors.error.r,
                    colors.error.g,
                    colors.error.b,
                    0.12,
                ))),
                colors.error,
            ),
            _ => (
                Some(iced::Background::Color(iced::Color::from_rgba(
                    colors.text_muted.r,
                    colors.text_muted.g,
                    colors.text_muted.b,
                    0.10,
                ))),
                colors.text_secondary,
            ),
        };

        button::Style {
            background,
            text_color,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
        }
    }
}

/// Tạo Modal Popup chuẩn:
/// - Căn giữa ngang
/// - Cách top 1/4 màn hình dọc
/// - Nền mờ tối (Overlay)
/// - Giao diện Card đẹp mắt + Nút đóng X
///
/// # Arguments
/// * `base` - Nội dung màn hình chính
/// * `title` - Tiêu đề popup
/// * `content` - Nội dung bên trong popup
/// * `on_close` - Message gửi đi khi đóng popup (bấm X hoặc click nền)
/// * `compact` - Chế độ gọn (giảm padding/spacing)
pub fn modal<'a, Message: 'a + Clone>(
    base: Element<'a, Message>,
    title: &'a str,
    content: Element<'a, Message>,
    on_close: Message,
    compact: bool,
) -> Element<'a, Message> {
    // 1. Overlay nền tối
    let overlay = container(
        mouse_area(
            container(Space::with_width(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(on_close.clone()),
    )
    .style(popup_overlay_style())
    .width(Length::Fill)
    .height(Length::Fill);

    let spacing = if compact { 8 } else { 16 };
    let padding = if compact { 16 } else { 24 };
    let title_size = if compact { 16 } else { 18 };

    // 2. Header với nút đóng X
    let header = row![
        text(title).size(title_size).style(text_primary_color()),
        Space::with_width(Length::Fill),
        button(
            text(Bootstrap::X.to_string())
                .size(if compact { 14 } else { 16 })
                .font(BOOTSTRAP_FONT)
                .style(text_secondary_color())
        )
        .on_press(on_close)
        .padding(6)
        .style(close_button_style()),
    ]
    .align_y(Alignment::Center);

    // 3. Card nội dung - Width cố định, căn giữa
    let popup_card = container(
        column![header, Space::with_height(spacing), content]
            .spacing(0)
            .width(Length::Fill),
    )
    .style(popup_dialog_style())
    .padding(padding)
    .width(Length::Fixed(if compact { 400.0 } else { 450.0 }));

    // 4. Wrapper căn giữa: Space -> Card -> Space (tỉ lệ 1:0:3 => cách top 25%)
    let centered_wrapper = row![
        Space::with_width(Length::Fill),
        container(
            column![
                Space::with_height(Length::FillPortion(1)),
                popup_card,
                Space::with_height(Length::FillPortion(3)),
            ]
            .spacing(0)
            .height(Length::Fill),
        )
        .width(Length::Shrink),
        Space::with_width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(0);

    // 5. Stack layering
    stack![base, overlay, centered_wrapper]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
