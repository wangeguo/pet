use crate::error::{Error, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppPaths {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl AppPaths {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("", "", "pet").ok_or(Error::ConfigDirNotFound)?;

        let config_dir = proj_dirs.config_dir().to_path_buf();
        let data_dir = proj_dirs.data_dir().to_path_buf();

        Ok(Self {
            config_dir,
            data_dir,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(self.models_dir())?;
        std::fs::create_dir_all(self.scripts_dir())?;
        std::fs::create_dir_all(self.logs_dir())?;
        Ok(())
    }

    #[must_use]
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    #[must_use]
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    #[must_use]
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    #[must_use]
    pub fn state_file(&self) -> PathBuf {
        self.config_dir.join("state.toml")
    }

    #[must_use]
    pub fn models_dir(&self) -> PathBuf {
        self.data_dir.join("models")
    }

    #[must_use]
    pub fn scripts_dir(&self) -> PathBuf {
        self.data_dir.join("scripts")
    }

    #[must_use]
    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    #[must_use]
    pub fn model_path(&self, pet_id: &uuid::Uuid) -> PathBuf {
        self.models_dir().join(format!("{pet_id}.glb"))
    }
}
