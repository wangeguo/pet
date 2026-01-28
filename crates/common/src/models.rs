use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub model_path: PathBuf,
    pub created_at: Timestamp,
}

impl Pet {
    #[must_use]
    pub fn new(name: String, description: String, model_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            model_path,
            created_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl WindowPosition {
    #[must_use]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
