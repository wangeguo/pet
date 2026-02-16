//! IPC infrastructure for the main process

pub mod router;
pub mod server;

pub use router::MessageRouter;
pub use server::IpcServer;
