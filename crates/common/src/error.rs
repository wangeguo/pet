use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config directory not found")]
    ConfigDirNotFound,

    #[error("Data directory not found")]
    DataDirNotFound,

    #[error("Pet not found: {0}")]
    PetNotFound(uuid::Uuid),

    #[error("Script not found: {0}")]
    ScriptNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Assets directory not found")]
    AssetsNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
