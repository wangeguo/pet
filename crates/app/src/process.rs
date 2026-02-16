use app::ipc::{IpcServer, MessageRouter};
use common::config::{AppConfig, AppState};
use common::paths::AppPaths;
use common::{Result, autostart};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{error, info, warn};

pub struct ProcessManager {
    paths: AppPaths,
    tray: Option<Child>,
    theater: Option<Child>,
    manager: Option<Child>,
}

impl ProcessManager {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            tray: None,
            theater: None,
            manager: None,
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
            child.kill().await?;
            child.wait().await?;
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
            child.kill().await?;
            child.wait().await?;
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
        self.update_state(|state| state.manager_open = true)?;
        info!("Manager process started");
        Ok(())
    }

    pub async fn stop_manager(&mut self) -> Result<()> {
        if let Some(mut child) = self.manager.take() {
            info!("Stopping manager process...");
            child.kill().await?;
            child.wait().await?;
            self.update_state(|state| state.manager_open = false)?;
            info!("Manager process stopped");
        }
        Ok(())
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

    pub async fn run(&mut self) -> Result<()> {
        let (tx, mut rx) = tokio_mpsc::channel::<notify::Event>(16);

        let config_file = self.paths.config_file();
        let state_file = self.paths.state_file();
        let config_dir = self.paths.config_dir().clone();

        let _watcher = {
            let tx = tx.clone();
            let mut watcher = RecommendedWatcher::new(
                move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res
                        && event
                            .paths
                            .iter()
                            .any(|p| p == &config_file || p == &state_file)
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

        // Setup IPC server
        let mut ipc_server = IpcServer::new(self.paths.socket_path());
        let mut ipc_incoming = ipc_server.take_incoming();
        let listener = ipc_server.start()?;
        let router = MessageRouter::new(ipc_server.clients());

        info!("Process manager running, waiting for events...");

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
                    info!("Config file changed, reloading...");
                    if let Ok(config) = AppConfig::load(&self.paths)
                        && let Err(e) = autostart::sync_autostart(config.auto_start)
                    {
                        error!("Failed to sync auto-start on config change: {e}");
                    }
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
            }
        }

        ipc_server.cleanup();
        self.shutdown().await?;
        Ok(())
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
            let _ = self.update_state(|state| state.manager_open = false);
        }

        tray_exited
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down all processes...");

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
    }
}
