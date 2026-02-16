mod app;
mod views;

use app::SettingsApp;
use tracing::info;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet_settings=debug".parse().unwrap()),
        )
        .init();

    info!("Settings process starting...");

    iced::application(SettingsApp::new, SettingsApp::update, SettingsApp::view)
        .title("Pet Settings")
        .theme(SettingsApp::theme)
        .window_size(iced::Size::new(720.0, 500.0))
        .run()
}
