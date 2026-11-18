mod app;
mod config;
mod disk;

use app::DiskApplet;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando Disk Usage Applet para COSMIC Desktop");

    cosmic::applet::run::<DiskApplet>(())
}
