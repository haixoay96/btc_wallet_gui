#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod error;
mod i18n;
mod storage;
mod theme;
mod utils;
mod views;
mod wallet;

use app::App;
use iced::{window, Size};
use iced_fonts::BOOTSTRAP_FONT_BYTES;

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .window_size(Size::new(1200.0, 800.0))
        .window(window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(800.0, 600.0)),
            ..window::Settings::default()
        })
        .theme(|state: &App| state.current_theme())
        .font(BOOTSTRAP_FONT_BYTES)
        .run_with(App::new)
}
