use crate::ui::components::network_status::structure::{DashboardNetworkMessage, NetworkStatus};
use crate::ui::i18n::t;
use crate::ui::theme::{
    text_error_color, text_secondary_color, text_success_color, text_warning_color,
};
use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Element, Length, widget::tooltip};
use iced_fonts::{BOOTSTRAP_FONT, bootstrap::advanced_text};

impl NetworkStatus {
    pub fn icon_char(&self) -> String {
        match self {
            NetworkStatus::Connected { .. } => advanced_text::wifi().0,
            NetworkStatus::Disconnected => advanced_text::wifi_off().0,
            NetworkStatus::Checking => advanced_text::arrow_clockwise().0,
        }
    }

    pub fn label(&self) -> String {
        match self {
            NetworkStatus::Connected { block_height } => {
                format!("{}: #{}", t("Đã kết nối", "Connected"), block_height)
            }
            NetworkStatus::Disconnected => t("Mất kết nối", "Disconnected").to_string(),
            NetworkStatus::Checking => t("Đang kiểm tra...", "Checking...").to_string(),
        }
    }
}

/// Render the network status indicator widget
pub fn network_status_indicator(
    status: NetworkStatus,
    is_clickable: bool,
) -> Element<'static, DashboardNetworkMessage> {
    // Dùng style function động để lấy màu từ theme hiện tại
    let icon_style = match status {
        NetworkStatus::Connected { .. } => text_success_color(),
        NetworkStatus::Disconnected => text_error_color(),
        NetworkStatus::Checking => text_warning_color(),
    };

    let icon = text(status.icon_char())
        .size(14)
        .font(BOOTSTRAP_FONT)
        .style(icon_style);

    let label = text(status.label()).size(11).style(text_secondary_color());

    let content = row![icon, Space::new().width(4), label].align_y(Alignment::Center);

    if is_clickable {
        let btn = button(content)
            .padding([8, 12])
            .height(Length::Fixed(36.0))
            .style(crate::ui::theme::secondary_button_style())
            .on_press(DashboardNetworkMessage::CheckConnection);

        container(
            tooltip(
                btn,
                t("Click để kiểm tra kết nối", "Click to check connection"),
                tooltip::Position::Bottom,
            )
            .gap(4)
            .style(crate::ui::theme::card_style()),
        )
        .height(Length::Fixed(36.0))
        .into()
    } else {
        container(content)
            .height(Length::Fixed(36.0))
            .align_y(iced::Alignment::Center)
            .into()
    }
}
