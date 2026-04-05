use iced::{
    widget::{column, container, row, Space},
    Color, Element, Length,
};

use crate::theme::{card_style, Colors};

/// Skeleton loader types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonType {
    Circle,
    Rectangle,
    Text,
    Card,
}

/// Create a shimmer animation effect placeholder
pub fn skeleton(skeleton_type: SkeletonType, size: (f32, f32)) -> Element<'static, ()> {
    match skeleton_type {
        SkeletonType::Circle => {
            let (width, height) = size;
            let radius = (width.min(height)) / 2.0;
            container(Space::with_width(width).height(height))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
                    border: iced::border::rounded(radius),
                    ..Default::default()
                })
                .into()
        }
        SkeletonType::Rectangle => {
            let (width, height) = size;
            container(Space::with_width(width).height(height))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
                    border: iced::border::rounded(4),
                    ..Default::default()
                })
                .into()
        }
        SkeletonType::Text => {
            let (width, height) = size;
            container(Space::with_width(width).height(height))
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.15))),
                    border: iced::border::rounded(4),
                    ..Default::default()
                })
                .into()
        }
        SkeletonType::Card => {
            let (width, height) = size;
            container(
                column![
                    skeleton(SkeletonType::Text, (width * 0.4, 20)),
                    Space::with_height(10),
                    skeleton(SkeletonType::Text, (width * 0.7, 30)),
                    Space::with_height(10),
                    skeleton(SkeletonType::Text, (width * 0.5, 14)),
                ],
            )
            .style(card_style())
            .padding(16)
            .width(width)
            .height(height)
            .into()
        }
    }
}

/// Skeleton for a list of items
pub fn skeleton_list(count: usize, item_height: f32, spacing: f32) -> Element<'static, ()> {
    let mut items = column![];
    for _ in 0..count {
        items = items.push(skeleton(SkeletonType::Text, (300.0, item_height)));
        items = items.push(Space::with_height(spacing));
    }
    items.into()
}

/// Skeleton for wallet cards
pub fn skeleton_wallet_cards(count: usize) -> Element<'static, ()> {
    let mut items = column![];
    for _ in 0..count {
        items = items.push(skeleton(SkeletonType::Card, (400.0, 120.0)));
        items = items.push(Space::with_height(12));
    }
    items.into()
}

/// Skeleton for transaction rows
pub fn skeleton_transactions(count: usize) -> Element<'static, ()> {
    skeleton_list(count, 60.0, 12.0)
}
