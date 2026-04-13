use crate::ui::components::price_widget::structure::PriceWidgetMessage;
use crate::ui::i18n::t;
use crate::ui::theme::{
    card_style, text_accent_teal_color, text_error_color, text_muted_color, text_primary_color,
    text_secondary_color, text_success_color,
};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};
use iced_fonts::{Bootstrap, BOOTSTRAP_FONT};

/// Render the BTC price widget
pub fn price_widget_view(
    price: Option<crate::infra::price_api::BtcPriceData>,
    total_balance_sat: i64,
    is_refreshing: bool,
) -> Element<'static, PriceWidgetMessage> {
    let mut content = column![];

    // Title row
    let title_row = row![
        text(t("Giá BTC", "BTC Price"))
            .size(14)
            .style(text_secondary_color()),
        Space::with_width(Length::Fill),
        button(
            text(Bootstrap::ArrowClockwise.to_string())
                .size(12)
                .font(BOOTSTRAP_FONT)
                .style(text_accent_teal_color()),
        )
        .on_press(PriceWidgetMessage::RefreshPrice)
        .padding(4)
        .style(crate::ui::theme::secondary_button_style()),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    content = content.push(title_row);
    content = content.push(Space::with_height(8));

    // Price display
    if let Some(price_data) = &price {
        let formatted_price = format_price_usd(price_data.price_usd);

        // Bootstrap icon for change direction (rendered with BOOTSTRAP_FONT)
        let change_icon = if price_data.change_24h >= 0.0 {
            Bootstrap::ArrowUp
        } else {
            Bootstrap::ArrowDown
        }
        .to_string();

        // BTC price row: price + icon + percentage
        let icon_style = if price_data.change_24h >= 0.0 {
            text_success_color()
        } else {
            text_error_color()
        };
        let text_style = if price_data.change_24h >= 0.0 {
            text_success_color()
        } else {
            text_error_color()
        };
        let price_row = row![
            text(formatted_price).size(20).style(text_primary_color()),
            Space::with_width(8),
            text(change_icon)
                .size(13)
                .font(BOOTSTRAP_FONT)
                .style(icon_style),
            Space::with_width(2),
            text(format!("{:.2}% (24h)", price_data.change_24h.abs()))
                .size(13)
                .style(text_style),
        ]
        .align_y(Alignment::Center);

        content = content.push(price_row);

        // Balance in USD
        let balance_usd = (total_balance_sat as f64 / 100_000_000.0) * price_data.price_usd;
        let balance_text = format!(
            "{}: ${:.2} USD",
            t("Tổng balance", "Total balance"),
            balance_usd
        );
        content = content.push(Space::with_height(4));
        content = content.push(text(balance_text).size(12).style(text_muted_color()));
    } else {
        // Error / unavailable state
        let hint = if is_refreshing {
            t("Đang tải giá...", "Loading price...")
        } else {
            t("Giá không khả dụng", "Price unavailable")
        };
        content = content.push(text(hint).size(16).style(text_muted_color()));
    }

    container(content.padding(12))
        .style(card_style())
        .width(Length::Fill)
        .into()
}

fn format_price_usd(price: f64) -> String {
    let formatted = format!("{:.2}", price);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_part = parts[0];
    let mut result = String::new();
    for (i, c) in int_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    if parts.len() > 1 {
        result.push('.');
        result.push_str(parts[1]);
    }
    format!("1 BTC = ${} USD", result)
}
