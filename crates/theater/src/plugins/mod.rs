//! Bevy plugins for the theater process

mod interaction;
mod ipc;
mod pet;
mod replay;

pub use interaction::InteractionPlugin;
pub use ipc::IpcPlugin;
pub use pet::PetPlugin;
pub use replay::ReplayPlugin;
