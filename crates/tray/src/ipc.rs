//! IPC client for tray process - connects to the app's UDS server

use common::ipc::{IpcEnvelope, IpcMessage, MAX_MESSAGE_SIZE, ProcessId};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use tracing::{error, info, warn};

/// Spawn a background thread with tokio runtime for UDS communication.
/// Returns true if the connection was established successfully.
pub fn spawn_ipc_client(
    socket_path: PathBuf,
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
                let stream = match tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    tokio::net::UnixStream::connect(&socket_path),
                )
                .await
                {
                    Ok(Ok(s)) => {
                        info!("Tray IPC connected to {}", socket_path.display());
                        let _ = connected_tx.send(true);
                        s
                    }
                    Ok(Err(e)) => {
                        warn!("Tray IPC connection failed: {e}");
                        let _ = connected_tx.send(false);
                        return;
                    }
                    Err(_) => {
                        warn!("Tray IPC connection timed out");
                        let _ = connected_tx.send(false);
                        return;
                    }
                };

                let (mut reader, mut writer) = stream.into_split();

                // Register with the app process
                let ready =
                    IpcEnvelope::new(ProcessId::Tray, ProcessId::App, IpcMessage::ProcessReady);
                if let Ok(data) = ready.encode()
                    && let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut writer, &data).await
                {
                    error!("Failed to send ProcessReady: {e}");
                    return;
                }

                // Read task: forward incoming messages to the main thread
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
                    info!("Tray IPC read task ended");
                });

                // Write loop: poll outgoing channel and send to UDS
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
                info!("Tray IPC client thread exiting");
            });
        })
        .expect("Failed to spawn IPC thread");

    connected_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap_or(false)
}
