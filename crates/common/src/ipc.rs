//! IPC message types and frame encoding for inter-process communication

use crate::script::BehaviorScript;
use serde::{Deserialize, Serialize};

/// Maximum IPC message payload size (1 MB)
pub const MAX_MESSAGE_SIZE: u32 = 1_048_576;

/// Process identifier for IPC routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessId {
    App,
    Tray,
    Theater,
    Brain,
    Manager,
    Settings,
}

/// IPC message envelope with routing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEnvelope {
    pub source: ProcessId,
    pub target: ProcessId,
    pub payload: IpcMessage,
    pub timestamp: i64,
}

/// IPC message payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    // Theater -> Brain
    PetClicked,
    PetDragCompleted { x: i32, y: i32 },
    AnimationsAvailable { animations: Vec<String> },

    // Brain -> Theater
    ExecuteScript { script: BehaviorScript },
    AiThinking { is_thinking: bool },

    // Tray/Manager -> Brain
    UserTextInput { text: String },

    // General
    ProcessReady,
    Shutdown,
    Ping,
    Pong,
}

impl IpcEnvelope {
    /// Create a new envelope with the current timestamp
    pub fn new(source: ProcessId, target: ProcessId, payload: IpcMessage) -> Self {
        Self {
            source,
            target,
            payload,
            timestamp: jiff::Timestamp::now().as_millisecond(),
        }
    }

    /// Serialize to length-prefixed JSON bytes
    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        let json = serde_json::to_vec(self)?;
        let len = json.len() as u32;
        if len > MAX_MESSAGE_SIZE {
            return Err(crate::Error::IpcMessageTooLarge(len));
        }
        let mut buf = Vec::with_capacity(4 + json.len());
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&json);
        Ok(buf)
    }

    /// Decode from a JSON byte slice (without the length prefix)
    pub fn decode(data: &[u8]) -> crate::Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::App => write!(f, "App"),
            Self::Tray => write!(f, "Tray"),
            Self::Theater => write!(f, "Theater"),
            Self::Brain => write!(f, "Brain"),
            Self::Manager => write!(f, "Manager"),
            Self::Settings => write!(f, "Settings"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::{Action, BehaviorScript, Keyframe};

    #[test]
    fn encode_decode_roundtrip() {
        let envelope = IpcEnvelope::new(ProcessId::Theater, ProcessId::App, IpcMessage::PetClicked);
        let encoded = envelope.encode().unwrap();
        // Skip 4-byte length prefix
        let decoded = IpcEnvelope::decode(&encoded[4..]).unwrap();
        assert_eq!(decoded.source, ProcessId::Theater);
        assert_eq!(decoded.target, ProcessId::App);
        assert!(matches!(decoded.payload, IpcMessage::PetClicked));
    }

    #[test]
    fn process_id_serialization() {
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
    fn execute_script_message_roundtrip() {
        let script = BehaviorScript {
            id: "test".to_string(),
            duration: Some(2.0),
            keyframes: vec![Keyframe {
                time: 0.0,
                action: Action::PlayAnimation {
                    name: "idle".to_string(),
                },
            }],
            next: None,
            interruptible: true,
        };

        let envelope = IpcEnvelope::new(
            ProcessId::Brain,
            ProcessId::Theater,
            IpcMessage::ExecuteScript { script },
        );

        let encoded = envelope.encode().unwrap();
        let decoded = IpcEnvelope::decode(&encoded[4..]).unwrap();

        if let IpcMessage::ExecuteScript { script } = decoded.payload {
            assert_eq!(script.id, "test");
            assert_eq!(script.keyframes.len(), 1);
        } else {
            panic!("Expected ExecuteScript message");
        }
    }

    #[test]
    fn length_prefix_is_correct() {
        let envelope = IpcEnvelope::new(ProcessId::App, ProcessId::Theater, IpcMessage::Ping);
        let encoded = envelope.encode().unwrap();
        let len = u32::from_le_bytes(encoded[..4].try_into().unwrap());
        assert_eq!(len as usize, encoded.len() - 4);
    }

    #[test]
    fn animations_available_message() {
        let envelope = IpcEnvelope::new(
            ProcessId::Theater,
            ProcessId::Brain,
            IpcMessage::AnimationsAvailable {
                animations: vec!["idle".to_string(), "walk".to_string()],
            },
        );

        let encoded = envelope.encode().unwrap();
        let decoded = IpcEnvelope::decode(&encoded[4..]).unwrap();

        if let IpcMessage::AnimationsAvailable { animations } = decoded.payload {
            assert_eq!(animations, vec!["idle", "walk"]);
        } else {
            panic!("Expected AnimationsAvailable message");
        }
    }

    #[test]
    fn process_id_display() {
        assert_eq!(ProcessId::Theater.to_string(), "Theater");
        assert_eq!(ProcessId::Brain.to_string(), "Brain");
    }
}
