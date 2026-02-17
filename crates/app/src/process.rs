#[cfg(unix)]
use app::ipc::router::AppCommand;
#[cfg(unix)]
use app::ipc::server::ClientWriter;
#[cfg(unix)]
use app::ipc::{IpcServer, MessageRouter};
use common::config::{AppConfig, AppState, AppearanceSettings};
use common::paths::AppPaths;
use common::{Result, autostart};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{error, info, warn};
#[cfg(unix)]
use {
    common::ipc::{IpcEnvelope, IpcMessage, ProcessId},
    std::collections::HashMap,
    std::sync::Arc,
    tokio::sync::Mutex,
};

pub struct ProcessManager {
    paths: AppPaths,
    tray: Option<Child>,
    theater: Option<Child>,
    manager: Option<Child>,
    settings: Option<Child>,
    pet_visible: bool,
    last_active_pet: Option<uuid::Uuid>,
    last_appearance: AppearanceSettings,
    #[cfg(unix)]
    ipc_clients: Option<Arc<Mutex<HashMap<ProcessId, ClientWriter>>>>,
}

impl ProcessManager {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            tray: None,
            theater: None,
            manager: None,
            settings: None,
            pet_visible: true,
            last_active_pet: None,
            last_appearance: AppearanceSettings::default(),
            #[cfg(unix)]
            ipc_clients: None,
        }
    }

    fn get_exe_path(name: &str) -> std::path::PathBuf {
        let exe = std::env::current_exe().expect("Failed to get current executable path");
        let dir = exe.parent().expect("Failed to get executable directory");
        dir.join(name)
    }

    pub fn start_tray(&mut self) -> Result<()> {
        if self.tray.is_some() {
            info!("Tray process already running");
            return Ok(());
        }

        let exe_path = Self::get_exe_path("pet-tray");
        info!("Starting tray process: {exe_path:?}");

        let child = Command::new(&exe_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        self.tray = Some(child);
        info!("Tray process started");
        Ok(())
    }

    pub async fn stop_tray(&mut self) -> Result<()> {
        if let Some(mut child) = self.tray.take() {
            info!("Stopping tray process...");
            #[cfg(unix)]
            self.send_shutdown(ProcessId::Tray).await;
            Self::wait_or_kill(&mut child, 1).await;
            info!("Tray process stopped");
        }
        Ok(())
    }

    pub fn start_theater(&mut self) -> Result<()> {
        if self.theater.is_some() {
            info!("Theater process already running");
            return Ok(());
        }

        let exe_path = Self::get_exe_path("pet-theater");
        info!("Starting theater process: {exe_path:?}");

        let child = Command::new(&exe_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        self.theater = Some(child);
        self.update_state(|state| state.theater_running = true)?;
        info!("Theater process started");
        Ok(())
    }

    pub async fn stop_theater(&mut self) -> Result<()> {
        if let Some(mut child) = self.theater.take() {
            info!("Stopping theater process...");
            #[cfg(unix)]
            self.send_shutdown(ProcessId::Theater).await;
            Self::wait_or_kill(&mut child, 2).await;
            self.update_state(|state| state.theater_running = false)?;
            info!("Theater process stopped");
        }
        Ok(())
    }

    pub fn start_manager(&mut self) -> Result<()> {
        if self.manager.is_some() {
            info!("Manager process already running");
            return Ok(());
        }

        let exe_path = Self::get_exe_path("pet-manager");
        info!("Starting manager process: {exe_path:?}");

        let child = Command::new(&exe_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        self.manager = Some(child);
        info!("Manager process started");
        Ok(())
    }

    pub async fn stop_manager(&mut self) -> Result<()> {
        if let Some(mut child) = self.manager.take() {
            info!("Stopping manager process...");
            #[cfg(unix)]
            self.send_shutdown(ProcessId::Manager).await;
            Self::wait_or_kill(&mut child, 1).await;
            info!("Manager process stopped");
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn start_settings(&mut self) -> Result<()> {
        if self.settings.is_some() {
            info!("Settings process already running");
            return Ok(());
        }

        let exe_path = Self::get_exe_path("pet-settings");
        info!("Starting settings process: {exe_path:?}");

        let child = Command::new(&exe_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        self.settings = Some(child);
        info!("Settings process started");
        Ok(())
    }

    pub async fn stop_settings(&mut self) -> Result<()> {
        if let Some(mut child) = self.settings.take() {
            info!("Stopping settings process...");
            #[cfg(unix)]
            self.send_shutdown(ProcessId::Settings).await;
            Self::wait_or_kill(&mut child, 1).await;
            info!("Settings process stopped");
        }
        Ok(())
    }

    /// Wait for a child to exit within timeout, then kill if still alive
    async fn wait_or_kill(child: &mut Child, timeout_secs: u64) {
        match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), child.wait()).await
        {
            Ok(Ok(status)) => {
                info!("Process exited gracefully: {status:?}");
            }
            Ok(Err(e)) => {
                warn!("Error waiting for process: {e}");
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
            Err(_) => {
                warn!("Process did not exit in time, killing");
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }
    }

    /// Send IPC Shutdown message to a process
    #[cfg(unix)]
    async fn send_shutdown(&self, target: ProcessId) {
        if let Some(ref clients) = self.ipc_clients {
            let clients = clients.lock().await;
            if let Some(writer) = clients.get(&target) {
                let msg = IpcEnvelope::new(ProcessId::App, target, IpcMessage::Shutdown);
                let _ = writer.send(&msg).await;
            }
        }
    }

    /// Send an IPC message to a specific process
    #[cfg(unix)]
    async fn send_to(&self, target: ProcessId, payload: IpcMessage) {
        if let Some(ref clients) = self.ipc_clients {
            let clients = clients.lock().await;
            if let Some(writer) = clients.get(&target) {
                let msg = IpcEnvelope::new(ProcessId::App, target, payload);
                if let Err(e) = writer.send(&msg).await {
                    warn!("Failed to send IPC message to {target}: {e}");
                }
            }
        }
    }

    fn update_state<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppState),
    {
        let mut state = AppState::load(&self.paths).unwrap_or_default();
        f(&mut state);
        state.save(&self.paths)?;
        Ok(())
    }

    pub async fn run(&mut self, config: &AppConfig) -> Result<()> {
        // Initialize tracking state from config
        self.last_active_pet = config.active_pet;
        self.last_appearance = config.appearance.clone();
        self.pet_visible = AppState::load(&self.paths)
            .map(|s| s.pet_visible)
            .unwrap_or(true);

        let config_file = self.paths.config_file();
        let config_dir = self.paths.config_dir().clone();

        // File watcher for config.toml changes
        let (tx, mut rx) = tokio_mpsc::channel::<notify::Event>(16);
        let _watcher = {
            let tx = tx.clone();
            let config_file = config_file.clone();
            let mut watcher = RecommendedWatcher::new(
                move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res
                        && event.paths.iter().any(|p| p == &config_file)
                    {
                        let _ = tx.blocking_send(event);
                    }
                },
                Config::default(),
            )
            .expect("Failed to create file watcher");

            watcher
                .watch(&config_dir, RecursiveMode::NonRecursive)
                .expect("Failed to watch config directory");

            watcher
        };

        info!("Process manager running, waiting for events...");

        #[cfg(unix)]
        {
            // Start IPC server BEFORE spawning child processes
            let mut ipc_server = IpcServer::new(self.paths.socket_path());
            let mut ipc_incoming = ipc_server.take_incoming();
            let listener = ipc_server.start()?;
            self.ipc_clients = Some(ipc_server.clients());

            let (cmd_tx, mut cmd_rx) = tokio_mpsc::channel::<AppCommand>(16);
            let router = MessageRouter::new(ipc_server.clients(), cmd_tx);

            // Now spawn child processes (IPC server is ready)
            self.start_child_processes(config);

            loop {
                tokio::select! {
                    _ = signal::ctrl_c() => {
                        info!("Received shutdown signal");
                        break;
                    }
                    should_exit = self.check_processes() => {
                        if should_exit {
                            info!("Tray process exited, shutting down...");
                            break;
                        }
                    }
                    Some(_event) = rx.recv() => {
                        self.handle_config_change().await;
                    }
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _addr)) => {
                                info!("New IPC connection accepted");
                                ipc_server.handle_connection(stream);
                            }
                            Err(e) => {
                                error!("Failed to accept IPC connection: {e}");
                            }
                        }
                    }
                    Some(msg) = ipc_incoming.recv() => {
                        router.route(msg.envelope).await;
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        if self.handle_app_command(cmd).await {
                            break;
                        }
                    }
                }
            }

            ipc_server.cleanup();
        }

        #[cfg(not(unix))]
        {
            self.start_child_processes(config);

            loop {
                tokio::select! {
                    _ = signal::ctrl_c() => {
                        info!("Received shutdown signal");
                        break;
                    }
                    should_exit = self.check_processes() => {
                        if should_exit {
                            info!("Tray process exited, shutting down...");
                            break;
                        }
                    }
                    Some(_event) = rx.recv() => {
                        self.handle_config_change().await;
                    }
                }
            }
        }

        self.shutdown().await?;
        Ok(())
    }

    /// Start child processes after IPC server is ready
    fn start_child_processes(&mut self, config: &AppConfig) {
        if let Err(e) = self.start_tray() {
            error!("Failed to start tray process: {e}");
        }

        if let Err(e) = self.start_theater() {
            error!("Failed to start theater process: {e}");
        }

        let is_first_run = config.pets.is_empty();
        if is_first_run {
            info!("First run detected, opening manager...");
            if let Err(e) = self.start_manager() {
                error!("Failed to start manager process: {e}");
            }
        }
    }

    /// Handle config.toml changes: auto_start, active_pet, appearance
    async fn handle_config_change(&mut self) {
        info!("Config file changed, reloading...");
        let Ok(config) = AppConfig::load(&self.paths) else {
            return;
        };

        // Sync auto-start setting
        if let Err(e) = autostart::sync_autostart(config.general.auto_start) {
            error!("Failed to sync auto-start on config change: {e}");
        }

        // Detect active_pet change -> restart theater
        if config.active_pet != self.last_active_pet {
            info!(
                "Active pet changed: {:?} -> {:?}",
                self.last_active_pet, config.active_pet
            );
            self.last_active_pet = config.active_pet;
            let _ = self.stop_theater().await;
            if config.active_pet.is_some()
                && self.pet_visible
                && let Err(e) = self.start_theater()
            {
                error!("Failed to restart theater: {e}");
            }
        }

        // Detect appearance change -> push to Theater via IPC
        if config.appearance != self.last_appearance {
            info!("Appearance settings changed, pushing to Theater");
            self.last_appearance = config.appearance.clone();
            #[cfg(unix)]
            self.send_to(
                ProcessId::Theater,
                IpcMessage::UpdateAppearance {
                    pet_scale: config.appearance.pet_scale,
                    opacity: config.appearance.opacity,
                    always_on_top: config.appearance.always_on_top,
                },
            )
            .await;
        }
    }

    /// Handle commands from the IPC router. Returns true if the app should quit.
    #[cfg(unix)]
    async fn handle_app_command(&mut self, cmd: AppCommand) -> bool {
        match cmd {
            AppCommand::TogglePetVisibility => {
                self.pet_visible = !self.pet_visible;
                info!("Pet visibility toggled: {}", self.pet_visible);

                if self.pet_visible {
                    if let Err(e) = self.start_theater() {
                        error!("Failed to start theater: {e}");
                    }
                } else {
                    let _ = self.stop_theater().await;
                }

                let _ = self.update_state(|s| s.pet_visible = self.pet_visible);
                self.send_to(
                    ProcessId::Tray,
                    IpcMessage::PetVisibilityChanged {
                        visible: self.pet_visible,
                    },
                )
                .await;
                false
            }
            AppCommand::OpenManager => {
                if let Err(e) = self.start_manager() {
                    error!("Failed to start manager: {e}");
                }
                false
            }
            AppCommand::OpenSettings => {
                if let Err(e) = self.start_settings() {
                    error!("Failed to start settings: {e}");
                }
                false
            }
            AppCommand::QuitApp => {
                info!("Quit requested via IPC");
                true
            }
        }
    }

    async fn check_processes(&mut self) -> bool {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let mut tray_exited = false;

        if let Some(ref mut child) = self.tray
            && let Ok(Some(status)) = child.try_wait()
        {
            warn!("Tray process exited with status: {status:?}");
            self.tray = None;
            tray_exited = true;
        }

        if let Some(ref mut child) = self.theater
            && let Ok(Some(status)) = child.try_wait()
        {
            warn!("Theater process exited with status: {status:?}");
            self.theater = None;
            let _ = self.update_state(|state| state.theater_running = false);
        }

        if let Some(ref mut child) = self.manager
            && let Ok(Some(status)) = child.try_wait()
        {
            info!("Manager process exited with status: {status:?}");
            self.manager = None;
        }

        if let Some(ref mut child) = self.settings
            && let Ok(Some(status)) = child.try_wait()
        {
            info!("Settings process exited with status: {status:?}");
            self.settings = None;
        }

        tray_exited
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down all processes...");

        if let Err(e) = self.stop_settings().await {
            error!("Error stopping settings: {}", e);
        }

        if let Err(e) = self.stop_manager().await {
            error!("Error stopping manager: {}", e);
        }

        if let Err(e) = self.stop_theater().await {
            error!("Error stopping theater: {}", e);
        }

        if let Err(e) = self.stop_tray().await {
            error!("Error stopping tray: {}", e);
        }

        Ok(())
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.tray {
            let _ = child.start_kill();
        }
        if let Some(ref mut child) = self.theater {
            let _ = child.start_kill();
        }
        if let Some(ref mut child) = self.manager {
            let _ = child.start_kill();
        }
        if let Some(ref mut child) = self.settings {
            let _ = child.start_kill();
        }
    }
}
