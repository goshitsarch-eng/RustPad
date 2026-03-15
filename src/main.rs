mod app;
mod file_ops;
mod menu;
mod message;
mod theme;

use app::RustPad;

fn main() -> iced::Result {
    iced::application(RustPad::new, RustPad::update, RustPad::view)
        .title(RustPad::title)
        .theme(RustPad::theme)
        .subscription(RustPad::subscription)
        .exit_on_close_request(false)
        .centered()
        .run()
}
