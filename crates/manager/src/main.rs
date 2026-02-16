mod app;
mod meshy;
mod views;

use app::PetManager;
use tracing::info;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet_manager=debug".parse().unwrap()),
        )
        .init();

    info!("Manager process starting...");

    iced::application(PetManager::new, PetManager::update, PetManager::view)
        .title("Pet Manager")
        .window_size(iced::Size::new(600.0, 500.0))
        .exit_on_close_request(false)
        .subscription(PetManager::subscription)
        .theme(PetManager::theme)
        .run()
}
