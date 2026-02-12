mod menu;

use common::config::{AppConfig, AppState};
use common::paths::AppPaths;
use common::{Result, autostart};
use menu::build_menu;
use tracing::{error, info};
use tray_icon::{Icon, TrayIconBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pet_tray=debug".parse().unwrap()),
        )
        .init();

    info!("Starting tray process...");

    let paths = AppPaths::new()?;
    let state = AppState::load(&paths).unwrap_or_default();
    let config = AppConfig::load(&paths).unwrap_or_default();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = TrayApp::new(paths, state, config);

    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");

    Ok(())
}

struct TrayApp {
    paths: AppPaths,
    state: AppState,
    config: AppConfig,
    tray_icon: Option<tray_icon::TrayIcon>,
}

impl TrayApp {
    fn new(paths: AppPaths, state: AppState, config: AppConfig) -> Self {
        Self {
            paths,
            state,
            config,
            tray_icon: None,
        }
    }

    fn create_tray_icon(&mut self) {
        let icon = create_default_icon();
        let menu = build_menu(self.state.pet_visible, self.config.auto_start);

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Pet - Desktop Companion")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        self.tray_icon = Some(tray_icon);
        info!("Tray icon created");
    }

    fn rebuild_menu(&self) {
        if let Some(ref tray) = self.tray_icon {
            let menu = build_menu(self.state.pet_visible, self.config.auto_start);
            tray.set_menu(Some(Box::new(menu)));
        }
    }
}

impl ApplicationHandler for TrayApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.tray_icon.is_none() {
            self.create_tray_icon();
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            match event.id.0.as_str() {
                "show_pet" => {
                    info!("Toggle pet visibility");
                    self.state.pet_visible = !self.state.pet_visible;
                    let _ = self.state.save(&self.paths);
                    self.rebuild_menu();
                }
                "auto_start" => {
                    info!("Toggle auto-start");
                    self.config.auto_start = !self.config.auto_start;
                    let _ = self.config.save(&self.paths);

                    if let Err(e) = autostart::sync_autostart(self.config.auto_start) {
                        error!("Failed to sync auto-start: {e}");
                    }
                }
                "open_manager" => {
                    info!("Open manager requested");
                    self.state.manager_open = true;
                    let _ = self.state.save(&self.paths);
                }
                "quit" => {
                    info!("Quit requested");
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

fn create_default_icon() -> Icon {
    let size: i32 = 32;
    let half = size / 2;
    #[allow(clippy::cast_sign_loss)]
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let dx = x - half;
            let dy = y - half;
            let dist = f64::from(dx * dx + dy * dy).sqrt();

            if dist < f64::from(size) / 2.0 - 2.0 {
                rgba.push(100);
                rgba.push(150);
                rgba.push(255);
                rgba.push(255);
            } else {
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }

    #[allow(clippy::cast_sign_loss)]
    let size_u32 = size as u32;
    Icon::from_rgba(rgba, size_u32, size_u32).expect("Failed to create icon")
}
