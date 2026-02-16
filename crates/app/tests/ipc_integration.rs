//! Integration tests for the IPC infrastructure

use common::ipc::{IpcEnvelope, IpcMessage, MAX_MESSAGE_SIZE, ProcessId};
use common::script::{Action, BehaviorScript, Keyframe};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[test]
fn encode_decode_roundtrip() {
    let envelope = IpcEnvelope::new(ProcessId::Theater, ProcessId::App, IpcMessage::PetClicked);
    let encoded = envelope.encode().unwrap();

    // Verify length prefix
    let len = u32::from_le_bytes(encoded[..4].try_into().unwrap());
    assert_eq!(len as usize, encoded.len() - 4);

    // Decode payload (without length prefix)
    let decoded = IpcEnvelope::decode(&encoded[4..]).unwrap();
    assert_eq!(decoded.source, ProcessId::Theater);
    assert_eq!(decoded.target, ProcessId::App);
    assert!(matches!(decoded.payload, IpcMessage::PetClicked));
}

#[test]
fn process_id_all_variants_roundtrip() {
    let ids = [
        ProcessId::App,
        ProcessId::Tray,
        ProcessId::Theater,
        ProcessId::Brain,
        ProcessId::Manager,
        ProcessId::Settings,
    ];

    for id in ids {
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ProcessId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}

#[test]
fn execute_script_message_in_envelope() {
    let script = BehaviorScript {
        id: "wave".to_string(),
        duration: Some(3.0),
        keyframes: vec![
            Keyframe {
                time: 0.0,
                action: Action::PlayAnimation {
                    name: "wave_start".to_string(),
                },
            },
            Keyframe {
                time: 1.5,
                action: Action::PlayAnimation {
                    name: "wave_end".to_string(),
                },
            },
        ],
        next: Some("idle".to_string()),
        interruptible: true,
    };

    let envelope = IpcEnvelope::new(
        ProcessId::Brain,
        ProcessId::Theater,
        IpcMessage::ExecuteScript {
            script: script.clone(),
        },
    );

    let encoded = envelope.encode().unwrap();
    let decoded = IpcEnvelope::decode(&encoded[4..]).unwrap();

    assert_eq!(decoded.source, ProcessId::Brain);
    assert_eq!(decoded.target, ProcessId::Theater);

    if let IpcMessage::ExecuteScript {
        script: decoded_script,
    } = decoded.payload
    {
        assert_eq!(decoded_script.id, "wave");
        assert_eq!(decoded_script.duration, Some(3.0));
        assert_eq!(decoded_script.keyframes.len(), 2);
        assert_eq!(decoded_script.next, Some("idle".to_string()));
    } else {
        panic!("Expected ExecuteScript message");
    }
}

/// Helper: connect to UDS server, send ProcessReady, return stream
async fn connect_and_register(socket_path: &std::path::Path, process_id: ProcessId) -> UnixStream {
    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    let ready = IpcEnvelope::new(process_id, ProcessId::App, IpcMessage::ProcessReady);
    let data = ready.encode().unwrap();
    stream.write_all(&data).await.unwrap();

    // Give server time to process registration
    tokio::time::sleep(Duration::from_millis(50)).await;
    stream
}

/// Helper: read one envelope from a stream
async fn read_envelope(stream: &mut UnixStream) -> IpcEnvelope {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.unwrap();
    let len = u32::from_le_bytes(len_buf);
    assert!(len <= MAX_MESSAGE_SIZE);

    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload).await.unwrap();
    IpcEnvelope::decode(&payload).unwrap()
}

/// Helper: write one envelope to a stream
async fn write_envelope(stream: &mut UnixStream, envelope: &IpcEnvelope) {
    let data = envelope.encode().unwrap();
    stream.write_all(&data).await.unwrap();
}

#[tokio::test]
async fn server_client_ping_pong() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");

    let mut server = app::ipc::IpcServer::new(socket_path.clone());
    let mut incoming = server.take_incoming();
    let listener = server.start().unwrap();
    let router = app::ipc::MessageRouter::new(server.clients());

    // Accept connections in background
    let accept_handle = tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            // Accept one connection
            let (stream, _) = listener.accept().await.unwrap();
            (stream, socket_path)
        }
    });

    // Client connects and registers
    let mut client = connect_and_register(&socket_path, ProcessId::Theater).await;

    // Accept the connection on server side
    let (stream, _) = accept_handle.await.unwrap();
    server.handle_connection(stream);

    // Drain ProcessReady message from incoming
    tokio::time::sleep(Duration::from_millis(100)).await;
    while let Ok(msg) = incoming.try_recv() {
        router.route(msg.envelope).await;
    }

    // Client sends Ping to App
    let ping = IpcEnvelope::new(ProcessId::Theater, ProcessId::App, IpcMessage::Ping);
    write_envelope(&mut client, &ping).await;

    // Wait for server to process and route
    tokio::time::sleep(Duration::from_millis(100)).await;
    while let Ok(msg) = incoming.try_recv() {
        router.route(msg.envelope).await;
    }

    // Client should receive Pong
    let response = tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut client))
        .await
        .expect("Timed out waiting for Pong");

    assert_eq!(response.source, ProcessId::App);
    assert_eq!(response.target, ProcessId::Theater);
    assert!(matches!(response.payload, IpcMessage::Pong));

    server.cleanup();
}

#[tokio::test]
async fn message_routing_between_clients() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");

    let mut server = app::ipc::IpcServer::new(socket_path.clone());
    let mut incoming = server.take_incoming();
    let listener = server.start().unwrap();
    let router = app::ipc::MessageRouter::new(server.clients());

    // Spawn accept loop
    let server_handle = tokio::spawn(async move {
        let mut streams = Vec::new();
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            streams.push(stream);
        }
        streams
    });

    // Connect two clients
    let mut theater_client = connect_and_register(&socket_path, ProcessId::Theater).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut brain_client = connect_and_register(&socket_path, ProcessId::Brain).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Accept both connections
    let streams = server_handle.await.unwrap();
    for stream in streams {
        server.handle_connection(stream);
    }

    // Drain registration messages
    tokio::time::sleep(Duration::from_millis(100)).await;
    while let Ok(msg) = incoming.try_recv() {
        router.route(msg.envelope).await;
    }

    // Theater sends PetClicked to Brain
    let click = IpcEnvelope::new(ProcessId::Theater, ProcessId::Brain, IpcMessage::PetClicked);
    write_envelope(&mut theater_client, &click).await;

    // Route the message
    tokio::time::sleep(Duration::from_millis(100)).await;
    while let Ok(msg) = incoming.try_recv() {
        router.route(msg.envelope).await;
    }

    // Brain should receive PetClicked
    let received = tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut brain_client))
        .await
        .expect("Timed out waiting for PetClicked");

    assert_eq!(received.source, ProcessId::Theater);
    assert_eq!(received.target, ProcessId::Brain);
    assert!(matches!(received.payload, IpcMessage::PetClicked));

    // Brain sends ExecuteScript back to Theater
    let script = BehaviorScript {
        id: "react".to_string(),
        duration: Some(1.0),
        keyframes: vec![Keyframe {
            time: 0.0,
            action: Action::PlayAnimation {
                name: "happy".to_string(),
            },
        }],
        next: None,
        interruptible: true,
    };
    let exec = IpcEnvelope::new(
        ProcessId::Brain,
        ProcessId::Theater,
        IpcMessage::ExecuteScript { script },
    );
    write_envelope(&mut brain_client, &exec).await;

    // Route
    tokio::time::sleep(Duration::from_millis(100)).await;
    while let Ok(msg) = incoming.try_recv() {
        router.route(msg.envelope).await;
    }

    // Theater should receive ExecuteScript
    let received = tokio::time::timeout(Duration::from_secs(2), read_envelope(&mut theater_client))
        .await
        .expect("Timed out waiting for ExecuteScript");

    assert_eq!(received.source, ProcessId::Brain);
    assert_eq!(received.target, ProcessId::Theater);
    if let IpcMessage::ExecuteScript { script } = received.payload {
        assert_eq!(script.id, "react");
    } else {
        panic!("Expected ExecuteScript message");
    }

    server.cleanup();
}
