mod app;
mod input;
mod persistence;

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<app::BongoPenguin>(())
}
