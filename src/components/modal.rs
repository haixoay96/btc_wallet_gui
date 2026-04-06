use iced::{
    widget::{button, column, container, mouse_area, row, text, Space, stack},
    Alignment, Element, Length,
};

use crate::theme::{popup_dialog_style, popup_overlay_style, text_color, Colors};
use iced_fonts::{BOOTSTRAP_FONT, Bootstrap};

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
pub fn modal<'a, Message: 'a + Clone>(
    base: Element<'a, Message>,
    title: &'a str,
    content: Element<'a, Message>,
    on_close: Message,
) -> Element<'a, Message> {
    // 1. Overlay nền tối
    let overlay = container(
        mouse_area(container(Space::with_width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill))
        .on_press(on_close.clone()),
    )
    .style(popup_overlay_style())
    .width(Length::Fill)
    .height(Length::Fill);

    // 2. Header với nút đóng X
    let header = row![
        text(title).size(18).style(text_color(Colors::TEXT_PRIMARY)),
        Space::with_width(Length::Fill),
        button(
            text(Bootstrap::X.to_string())
                .size(16)
                .font(BOOTSTRAP_FONT)
                .style(text_color(Colors::TEXT_SECONDARY))
        )
        .on_press(on_close)
        .padding(6),
    ]
    .align_y(Alignment::Center);

    // 3. Card nội dung
    let popup_card = container(
        column![
            header,
            Space::with_height(16),
            content
        ]
        .spacing(0),
    )
    .style(popup_dialog_style())
    .padding(24)
    .width(Length::Fixed(450.0)); // Width cố định đẹp mắt

    // 4. Layout positioning: Top 1/4 màn hình
    // FillPortion(1) -> Content -> FillPortion(3) => Content nằm ở vạch 25%
    let layout = container(
        column![
            Space::with_height(Length::FillPortion(1)),
            container(popup_card).align_x(Alignment::Center),
            Space::with_height(Length::FillPortion(3)),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    // 5. Stack layering
    stack![base, overlay, layout]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
