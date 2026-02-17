#[cfg(unix)]
mod ipc;
mod menu;

use common::Result;
use common::config::AppState;
use common::paths::AppPaths;
use menu::build_menu;
use tracing::info;
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
    let pet_visible = state.pet_visible;

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = TrayApp::new(paths, pet_visible);

    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");

    Ok(())
}

struct TrayApp {
    pet_visible: bool,
    tray_icon: Option<tray_icon::TrayIcon>,
    #[cfg(unix)]
    ipc_outgoing: std::sync::mpsc::Sender<common::ipc::IpcEnvelope>,
    #[cfg(unix)]
    ipc_incoming: std::sync::mpsc::Receiver<common::ipc::IpcEnvelope>,
    #[cfg(unix)]
    ipc_connected: bool,
    // Non-unix fallback fields
    #[cfg(not(unix))]
    paths: AppPaths,
    #[cfg(not(unix))]
    state: AppState,
}

impl TrayApp {
    fn new(paths: AppPaths, pet_visible: bool) -> Self {
        #[cfg(unix)]
        {
            let (incoming_tx, incoming_rx) = std::sync::mpsc::channel();
            let (outgoing_tx, outgoing_rx) = std::sync::mpsc::channel();

            let socket_path = paths.socket_path();
            let connected = ipc::spawn_ipc_client(socket_path, incoming_tx, outgoing_rx);

            if connected {
                info!("Tray IPC connected");
            } else {
                info!("Tray IPC not connected, menu commands may not work");
            }

            Self {
                pet_visible,
                tray_icon: None,
                ipc_outgoing: outgoing_tx,
                ipc_incoming: incoming_rx,
                ipc_connected: connected,
            }
        }

        #[cfg(not(unix))]
        {
            let state = AppState {
                pet_visible,
                ..Default::default()
            };
            Self {
                pet_visible,
                tray_icon: None,
                paths,
                state,
            }
        }
    }

    fn create_tray_icon(&mut self) {
        let icon = create_default_icon();
        let menu = build_menu(self.pet_visible);

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
            let menu = build_menu(self.pet_visible);
            tray.set_menu(Some(Box::new(menu)));
        }
    }

    #[cfg(unix)]
    fn send_ipc(&self, payload: common::ipc::IpcMessage) {
        if !self.ipc_connected {
            return;
        }
        let envelope = common::ipc::IpcEnvelope::new(
            common::ipc::ProcessId::Tray,
            common::ipc::ProcessId::App,
            payload,
        );
        let _ = self.ipc_outgoing.send(envelope);
    }

    #[cfg(unix)]
    fn poll_ipc(&mut self) {
        use common::ipc::IpcMessage;

        while let Ok(envelope) = self.ipc_incoming.try_recv() {
            match envelope.payload {
                IpcMessage::PetVisibilityChanged { visible } => {
                    self.pet_visible = visible;
                    self.rebuild_menu();
                }
                IpcMessage::Shutdown => {
                    info!("Received shutdown via IPC");
                    std::process::exit(0);
                }
                _ => {
                    info!("Tray: unhandled IPC message: {:?}", envelope.payload);
                }
            }
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
                    #[cfg(unix)]
                    self.send_ipc(common::ipc::IpcMessage::TogglePetVisibility);
                    #[cfg(not(unix))]
                    {
                        self.state.pet_visible = !self.state.pet_visible;
                        self.pet_visible = self.state.pet_visible;
                        let _ = self.state.save(&self.paths);
                        self.rebuild_menu();
                    }
                }
                "open_settings" => {
                    info!("Open settings requested");
                    #[cfg(unix)]
                    self.send_ipc(common::ipc::IpcMessage::OpenSettings);
                }
                "open_manager" => {
                    info!("Open manager requested");
                    #[cfg(unix)]
                    self.send_ipc(common::ipc::IpcMessage::OpenManager);
                }
                "quit" => {
                    info!("Quit requested");
                    #[cfg(unix)]
                    self.send_ipc(common::ipc::IpcMessage::QuitApp);
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        #[cfg(unix)]
        self.poll_ipc();
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
