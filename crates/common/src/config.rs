use crate::error::{Error, Result};
use crate::models::{Pet, WindowPosition};
use crate::paths::AppPaths;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub pets: Vec<Pet>,
    pub active_pet: Option<Uuid>,
    #[serde(default)]
    pub pet_position: WindowPosition,
    #[serde(default = "default_scale")]
    pub pet_scale: f32,
    #[serde(default)]
    pub auto_start: bool,
    pub meshy_api_key: Option<String>,
}

fn default_scale() -> f32 {
    1.0
}

impl AppConfig {
    pub fn load(paths: &AppPaths) -> Result<Self> {
        let config_file = paths.config_file();
        if config_file.exists() {
            let settings = Config::builder()
                .add_source(File::from(config_file))
                .build()
                .map_err(|e| Error::InvalidConfig(e.to_string()))?;

            settings
                .try_deserialize()
                .map_err(|e| Error::InvalidConfig(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, paths: &AppPaths) -> Result<()> {
        paths.ensure_dirs()?;
        let config_file = paths.config_file();
        let content = toml::to_string_pretty(self)?;
        let mut file = fs::File::create(config_file)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn add_pet(&mut self, pet: Pet) {
        self.pets.push(pet);
    }

    pub fn remove_pet(&mut self, pet_id: Uuid) {
        self.pets.retain(|p| p.id != pet_id);
        if self.active_pet == Some(pet_id) {
            self.active_pet = self.pets.first().map(|p| p.id);
        }
    }

    #[must_use]
    pub fn get_pet(&self, pet_id: Uuid) -> Option<&Pet> {
        self.pets.iter().find(|p| p.id == pet_id)
    }

    #[must_use]
    pub fn get_active_pet(&self) -> Option<&Pet> {
        self.active_pet.and_then(|id| self.get_pet(id))
    }

    pub fn set_active_pet(&mut self, pet_id: Uuid) {
        if self.pets.iter().any(|p| p.id == pet_id) {
            self.active_pet = Some(pet_id);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub pet_visible: bool,
    pub theater_running: bool,
    pub manager_open: bool,
}

impl AppState {
    pub fn load(paths: &AppPaths) -> Result<Self> {
        let state_file = paths.state_file();
        if state_file.exists() {
            let settings = Config::builder()
                .add_source(File::from(state_file))
                .build()
                .map_err(|e| Error::InvalidConfig(e.to_string()))?;

            settings
                .try_deserialize()
                .map_err(|e| Error::InvalidConfig(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, paths: &AppPaths) -> Result<()> {
        paths.ensure_dirs()?;
        let state_file = paths.state_file();
        let content = toml::to_string_pretty(self)?;
        let mut file = fs::File::create(state_file)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
