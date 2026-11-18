mod app;
mod config;
mod disk;
mod system;

use app::SystemMonitorApplet;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando System Monitor Applet para COSMIC Desktop");

    cosmic::applet::run::<SystemMonitorApplet>(())
}
