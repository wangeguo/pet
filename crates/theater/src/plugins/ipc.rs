//! IPC plugin - bridges UDS client with Bevy ECS

use bevy::prelude::*;
use common::ipc::{IpcEnvelope, IpcMessage, MAX_MESSAGE_SIZE, ProcessId};
use std::sync::Mutex;
use std::sync::mpsc as std_mpsc;
use tracing::{error, info, warn};

use crate::events::{PetClickedEvent, SwitchScriptEvent};
use crate::resources::{ScriptLibrary, TheaterConfig};

/// Resource holding the IPC bridge channels.
/// Mutex wrappers are needed because mpsc channels are !Sync.
#[derive(Resource)]
struct IpcBridge {
    incoming_rx: Mutex<std_mpsc::Receiver<IpcEnvelope>>,
    outgoing_tx: Mutex<std_mpsc::Sender<IpcEnvelope>>,
    connected: bool,
}

/// Bevy plugin for IPC communication
pub struct IpcPlugin;

impl Plugin for IpcPlugin {
    fn build(&self, app: &mut App) {
        let (incoming_tx, incoming_rx) = std_mpsc::channel();
        let (outgoing_tx, outgoing_rx) = std_mpsc::channel();

        let connected = app
            .world()
            .get_resource::<TheaterConfig>()
            .map(|config| {
                let socket_path = config.paths.socket_path();
                spawn_ipc_thread(socket_path, incoming_tx, outgoing_rx)
            })
            .unwrap_or_else(|| {
                warn!("No TheaterConfig, IPC disabled");
                false
            });

        app.insert_resource(IpcBridge {
            incoming_rx: Mutex::new(incoming_rx),
            outgoing_tx: Mutex::new(outgoing_tx),
            connected,
        })
        .add_systems(Update, (receive_ipc_messages, forward_clicks_to_ipc));
    }
}

/// Spawn a background thread with tokio runtime for UDS communication.
/// Returns true if the connection was established successfully.
fn spawn_ipc_thread(
    socket_path: std::path::PathBuf,
    incoming_tx: std_mpsc::Sender<IpcEnvelope>,
    outgoing_rx: std_mpsc::Receiver<IpcEnvelope>,
) -> bool {
    let (connected_tx, connected_rx) = std_mpsc::channel();

    std::thread::Builder::new()
        .name("ipc-client".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime for IPC");

            rt.block_on(async move {
                // Connect with timeout
                let stream = match tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    tokio::net::UnixStream::connect(&socket_path),
                )
                .await
                {
                    Ok(Ok(s)) => {
                        info!("IPC client connected to {}", socket_path.display());
                        let _ = connected_tx.send(true);
                        s
                    }
                    Ok(Err(e)) => {
                        warn!("IPC connection failed: {e}");
                        let _ = connected_tx.send(false);
                        return;
                    }
                    Err(_) => {
                        warn!("IPC connection timed out");
                        let _ = connected_tx.send(false);
                        return;
                    }
                };

                let (mut reader, mut writer) = stream.into_split();

                // Send ProcessReady registration
                let ready =
                    IpcEnvelope::new(ProcessId::Theater, ProcessId::App, IpcMessage::ProcessReady);
                if let Ok(data) = ready.encode()
                    && let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut writer, &data).await
                {
                    error!("Failed to send ProcessReady: {e}");
                    return;
                }

                // Spawn read task
                let read_task = tokio::spawn(async move {
                    loop {
                        let mut len_buf = [0u8; 4];
                        if tokio::io::AsyncReadExt::read_exact(&mut reader, &mut len_buf)
                            .await
                            .is_err()
                        {
                            break;
                        }
                        let len = u32::from_le_bytes(len_buf);
                        if len > MAX_MESSAGE_SIZE {
                            error!("IPC message too large: {len}");
                            break;
                        }
                        let mut payload = vec![0u8; len as usize];
                        if tokio::io::AsyncReadExt::read_exact(&mut reader, &mut payload)
                            .await
                            .is_err()
                        {
                            break;
                        }
                        if let Ok(env) = IpcEnvelope::decode(&payload)
                            && incoming_tx.send(env).is_err()
                        {
                            break;
                        }
                    }
                    info!("IPC read task ended");
                });

                // Write loop: poll outgoing channel
                loop {
                    match outgoing_rx.try_recv() {
                        Ok(envelope) => {
                            if let Ok(data) = envelope.encode()
                                && tokio::io::AsyncWriteExt::write_all(&mut writer, &data)
                                    .await
                                    .is_err()
                            {
                                break;
                            }
                        }
                        Err(std_mpsc::TryRecvError::Empty) => {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                        Err(std_mpsc::TryRecvError::Disconnected) => {
                            break;
                        }
                    }
                }

                read_task.abort();
                info!("IPC client thread exiting");
            });
        })
        .expect("Failed to spawn IPC thread");

    // Wait for connection result
    connected_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap_or(false)
}

/// Bevy system: poll incoming IPC messages and dispatch to ECS events
fn receive_ipc_messages(
    bridge: Res<IpcBridge>,
    mut switch_events: MessageWriter<SwitchScriptEvent>,
    mut script_library: ResMut<ScriptLibrary>,
) {
    let rx = bridge.incoming_rx.lock().unwrap();
    while let Ok(envelope) = rx.try_recv() {
        match envelope.payload {
            IpcMessage::ExecuteScript { script } => {
                info!("IPC: received script '{}'", script.id);
                let script_id = script.id.clone();
                script_library.add(script);
                switch_events.write(SwitchScriptEvent {
                    script_id,
                    force: true,
                });
            }
            IpcMessage::Shutdown => {
                info!("IPC: received shutdown");
                std::process::exit(0);
            }
            IpcMessage::Pong => {
                info!("IPC: received pong");
            }
            _ => {
                info!("IPC: unhandled message: {:?}", envelope.payload);
            }
        }
    }
}

/// Bevy system: forward PetClicked events to IPC
fn forward_clicks_to_ipc(bridge: Res<IpcBridge>, mut click_events: MessageReader<PetClickedEvent>) {
    if !bridge.connected {
        return;
    }

    let tx = bridge.outgoing_tx.lock().unwrap();
    for _event in click_events.read() {
        let envelope =
            IpcEnvelope::new(ProcessId::Theater, ProcessId::Brain, IpcMessage::PetClicked);
        let _ = tx.send(envelope);
    }
}
