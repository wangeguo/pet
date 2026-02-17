//! Message routing for IPC

use common::ipc::{IpcEnvelope, IpcMessage, ProcessId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{info, warn};

use super::server::ClientWriter;

/// Commands that the router sends to the ProcessManager main loop
#[derive(Debug)]
pub enum AppCommand {
    TogglePetVisibility,
    OpenManager,
    OpenSettings,
    QuitApp,
}

/// Routes IPC messages between connected processes
pub struct MessageRouter {
    clients: Arc<Mutex<HashMap<ProcessId, ClientWriter>>>,
    cmd_tx: tokio_mpsc::Sender<AppCommand>,
}

impl MessageRouter {
    pub fn new(
        clients: Arc<Mutex<HashMap<ProcessId, ClientWriter>>>,
        cmd_tx: tokio_mpsc::Sender<AppCommand>,
    ) -> Self {
        Self { clients, cmd_tx }
    }

    /// Route a message to its target
    pub async fn route(&self, envelope: IpcEnvelope) {
        if envelope.target == ProcessId::App {
            self.handle_app_message(&envelope).await;
            return;
        }

        let clients = self.clients.lock().await;
        if let Some(writer) = clients.get(&envelope.target) {
            if let Err(e) = writer.send(&envelope).await {
                warn!("Failed to route message to {}: {e}", envelope.target);
            }
        } else {
            warn!(
                "No client registered for {}, dropping message",
                envelope.target
            );
        }
    }

    async fn handle_app_message(&self, envelope: &IpcEnvelope) {
        match &envelope.payload {
            IpcMessage::Ping => {
                let pong = IpcEnvelope::new(ProcessId::App, envelope.source, IpcMessage::Pong);
                let clients = self.clients.lock().await;
                if let Some(writer) = clients.get(&envelope.source) {
                    let _ = writer.send(&pong).await;
                }
            }
            IpcMessage::ProcessReady => {
                info!("Process {} is ready", envelope.source);
            }
            IpcMessage::TogglePetVisibility => {
                let _ = self.cmd_tx.send(AppCommand::TogglePetVisibility).await;
            }
            IpcMessage::OpenManager => {
                let _ = self.cmd_tx.send(AppCommand::OpenManager).await;
            }
            IpcMessage::OpenSettings => {
                let _ = self.cmd_tx.send(AppCommand::OpenSettings).await;
            }
            IpcMessage::QuitApp => {
                let _ = self.cmd_tx.send(AppCommand::QuitApp).await;
            }
            _ => {
                info!(
                    "App received {:?} from {}",
                    envelope.payload, envelope.source
                );
            }
        }
    }
}
