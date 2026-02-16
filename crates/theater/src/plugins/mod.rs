//! Bevy plugins for the theater process

mod interaction;
#[cfg(unix)]
mod ipc;
mod pet;
mod replay;

pub use interaction::InteractionPlugin;
#[cfg(unix)]
pub use ipc::IpcPlugin;
pub use pet::PetPlugin;
pub use replay::ReplayPlugin;
