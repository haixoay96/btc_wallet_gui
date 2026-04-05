#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod components;
mod i18n;
mod storage;
mod theme;
mod utils;
mod views;
mod wallet;

use app::App;
use iced::{font, window, Size, Task, Theme};
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
        .theme(|_| Theme::Dark)
        .font(BOOTSTRAP_FONT_BYTES)
        .run_with(App::new)
}
