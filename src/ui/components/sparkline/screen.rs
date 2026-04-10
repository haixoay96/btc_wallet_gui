use crate::ui::components::sparkline::structure::BalancePoint;
use crate::ui::theme::{card_style, text_muted_color};
use iced::widget::{column, container, row, text, tooltip, Space};
use iced::{Color, Element, Length};

/// Render sparkline as a row of mini bars (bottom-up)
///
/// Bar height = (value / max_value) * max_height
/// Scales proportionally from 0 to max for accurate visual comparison
pub fn sparkline_view<Message: Clone + 'static>(
    points: &[BalancePoint],
    line_color: Color,
    fill_alpha: f32,
) -> Element<'static, Message> {
    if points.is_empty() {
        return Space::with_height(0).into();
    }

    // Find max absolute balance for scaling
    let max_balance = points
        .iter()
        .map(|p| p.balance_sat.abs())
        .max()
        .unwrap_or(1);
    let max_f = max_balance as f32;

    let bar_max_height = 40.0;
    let gap = 3.0;

    let mut bars = row![];
    for (i, point) in points.iter().enumerate() {
        // Proportional scaling: z = (value / max_value) * x
        let abs_val = point.balance_sat.unsigned_abs();
        let bar_h = if max_f > 0.0 {
            ((abs_val as f32 / max_f) * bar_max_height).max(2.0)
        } else {
            2.0
        };

        let btc_value = point.balance_sat as f64 / 100_000_000.0;
        let tooltip_text = format!("{:.8} BTC", btc_value);

        // Bottom-up: spacer pushes bar to bottom
        let spacer_h = (bar_max_height - bar_h).max(0.0);
        let bar_content = column![
            Space::with_height(spacer_h),
            container(Space::with_height(bar_h))
                .width(Length::Fixed(12.0))
                .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color {
                        r: line_color.r,
                        g: line_color.g,
                        b: line_color.b,
                        a: fill_alpha,
                    })),
                    border: iced::Border {
                        radius: 3.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                }),
        ];

        let bar_slot = container(bar_content)
            .width(Length::Fixed(12.0))
            .height(Length::Fixed(bar_max_height));

        let bar_with_tooltip = tooltip(
            bar_slot,
            text(tooltip_text).size(10).style(text_muted_color()),
            tooltip::Position::Top,
        )
        .gap(4)
        .style(card_style());

        bars = bars.push(bar_with_tooltip);
        if i < points.len() - 1 {
            bars = bars.push(Space::with_width(gap));
        }
    }

    column![container(bars)
        .height(Length::Fixed(bar_max_height))
        .width(Length::Fill),]
    .into()
}
