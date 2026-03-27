mod app;
mod storage;
mod theme;
mod views;
mod wallet;

use app::App;
use iced::{window, Size, Theme};

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .window_size(Size::new(1200.0, 800.0))
        .window(window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(800.0, 600.0)),
            ..window::Settings::default()
        })
        .theme(|_| Theme::Dark)
        .run_with(App::new)
}
