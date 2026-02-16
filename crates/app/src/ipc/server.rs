//! UDS server for accepting and managing IPC connections

use common::ipc::{IpcEnvelope, IpcMessage, MAX_MESSAGE_SIZE, ProcessId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Handle to write messages to a connected client
#[derive(Clone)]
pub struct ClientWriter {
    tx: mpsc::Sender<Vec<u8>>,
}

impl ClientWriter {
    /// Send an envelope to this client
    pub async fn send(&self, envelope: &IpcEnvelope) -> common::Result<()> {
        let data = envelope.encode()?;
        self.tx
            .send(data)
            .await
            .map_err(|_| common::Error::IpcConnection("client disconnected".to_string()))
    }
}

/// Incoming message from a connected client
pub struct IncomingMessage {
    pub envelope: IpcEnvelope,
}

/// UDS server that manages client connections and message routing
pub struct IpcServer {
    socket_path: PathBuf,
    incoming_tx: mpsc::Sender<IncomingMessage>,
    incoming_rx: Option<mpsc::Receiver<IncomingMessage>>,
    clients: Arc<Mutex<HashMap<ProcessId, ClientWriter>>>,
}

impl IpcServer {
    pub fn new(socket_path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel(256);
        Self {
            socket_path,
            incoming_tx: tx,
            incoming_rx: Some(rx),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Take the incoming message receiver (can only be called once)
    pub fn take_incoming(&mut self) -> mpsc::Receiver<IncomingMessage> {
        self.incoming_rx.take().expect("incoming_rx already taken")
    }

    /// Get a handle to the client registry for routing
    pub fn clients(&self) -> Arc<Mutex<HashMap<ProcessId, ClientWriter>>> {
        self.clients.clone()
    }

    /// Start the UDS listener
    pub fn start(&self) -> common::Result<UnixListener> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!("IPC server listening on {}", self.socket_path.display());
        Ok(listener)
    }

    /// Handle a new incoming connection by spawning read/write tasks
    pub fn handle_connection(&self, stream: UnixStream) {
        let incoming_tx = self.incoming_tx.clone();
        let clients = self.clients.clone();

        tokio::spawn(async move {
            let (read_half, write_half) = stream.into_split();

            // Write task channel
            let (write_tx, mut write_rx) = mpsc::channel::<Vec<u8>>(64);

            // Spawn write task
            let write_handle = tokio::spawn(async move {
                let mut writer = write_half;
                while let Some(data) = write_rx.recv().await {
                    if let Err(e) = writer.write_all(&data).await {
                        error!("IPC write error: {e}");
                        break;
                    }
                }
            });

            // Read loop
            let mut reader = read_half;
            let mut registered_id: Option<ProcessId> = None;

            loop {
                let mut len_buf = [0u8; 4];
                if let Err(e) = reader.read_exact(&mut len_buf).await {
                    if e.kind() != std::io::ErrorKind::UnexpectedEof {
                        warn!("IPC read error: {e}");
                    }
                    break;
                }

                let len = u32::from_le_bytes(len_buf);
                if len > MAX_MESSAGE_SIZE {
                    error!("IPC message too large: {len} bytes");
                    break;
                }

                let mut payload = vec![0u8; len as usize];
                if let Err(e) = reader.read_exact(&mut payload).await {
                    warn!("IPC read payload error: {e}");
                    break;
                }

                let envelope = match IpcEnvelope::decode(&payload) {
                    Ok(env) => env,
                    Err(e) => {
                        error!("IPC decode error: {e}");
                        continue;
                    }
                };

                // Register client on ProcessReady
                if matches!(envelope.payload, IpcMessage::ProcessReady) && registered_id.is_none() {
                    registered_id = Some(envelope.source);
                    let writer = ClientWriter {
                        tx: write_tx.clone(),
                    };
                    clients.lock().await.insert(envelope.source, writer);
                    info!("IPC client registered: {}", envelope.source);
                }

                let _ = incoming_tx.send(IncomingMessage { envelope }).await;
            }

            // Cleanup on disconnect
            if let Some(id) = registered_id {
                clients.lock().await.remove(&id);
                info!("IPC client disconnected: {id}");
            }
            write_handle.abort();
        });
    }

    /// Remove the socket file
    pub fn cleanup(&self) {
        cleanup_socket(&self.socket_path);
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        cleanup_socket(&self.socket_path);
    }
}

fn cleanup_socket(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}
