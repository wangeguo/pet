pub mod autostart;
pub mod config;
pub mod error;
pub mod ipc;
pub mod models;
pub mod paths;
pub mod script;
pub mod storage;

pub use config::AppConfig;
pub use error::{Error, Result};
pub use ipc::{IpcEnvelope, IpcMessage, ProcessId};
pub use models::Pet;
pub use paths::AppPaths;
pub use script::BehaviorScript;
