use crate::error::{Error, Result};
use crate::models::Pet;
use crate::paths::AppPaths;
use crate::script::BehaviorScript;
use std::fs;
use uuid::Uuid;

pub struct StorageService {
    paths: AppPaths,
}

impl StorageService {
    #[must_use]
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    #[must_use]
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn save_model(&self, pet_id: &Uuid, data: &[u8]) -> Result<std::path::PathBuf> {
        self.paths.ensure_dirs()?;
        let path = self.paths.model_path(pet_id);
        fs::write(&path, data)?;
        Ok(path)
    }

    pub fn save_thumbnail(&self, pet_id: &Uuid, data: &[u8]) -> Result<std::path::PathBuf> {
        self.paths.ensure_dirs()?;
        let path = self.paths.models_dir().join(format!("{pet_id}.png"));
        fs::write(&path, data)?;
        Ok(path)
    }

    pub fn delete_model(&self, pet_id: &Uuid) -> Result<()> {
        let path = self.paths.model_path(pet_id);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn model_exists(&self, pet_id: &Uuid) -> bool {
        self.paths.model_path(pet_id).exists()
    }

    pub fn save_script(&self, script: &BehaviorScript) -> Result<()> {
        self.paths.ensure_dirs()?;
        let path = self.paths.scripts_dir().join(format!("{}.toml", script.id));
        let content = toml::to_string_pretty(script)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_script(&self, script_id: &str) -> Result<BehaviorScript> {
        let path = self.paths.scripts_dir().join(format!("{script_id}.toml"));
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let script: BehaviorScript = toml::from_str(&content)?;
            Ok(script)
        } else {
            Err(Error::ScriptNotFound(script_id.to_string()))
        }
    }

    pub fn load_all_scripts(&self) -> Result<Vec<BehaviorScript>> {
        let scripts_dir = self.paths.scripts_dir();
        let mut scripts = Vec::new();

        if scripts_dir.exists() {
            for entry in fs::read_dir(&scripts_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    let content = fs::read_to_string(&path)?;
                    if let Ok(script) = toml::from_str::<BehaviorScript>(&content) {
                        scripts.push(script);
                    }
                }
            }
        }

        Ok(scripts)
    }

    pub fn init_builtin_scripts(&self) -> Result<()> {
        self.paths.ensure_dirs()?;
        for script in BehaviorScript::builtin_scripts() {
            let path = self.paths.scripts_dir().join(format!("{}.toml", script.id));
            if !path.exists() {
                self.save_script(&script)?;
            }
        }
        Ok(())
    }

    pub fn delete_pet_data(&self, pet: &Pet) -> Result<()> {
        self.delete_model(&pet.id)?;
        let thumbnail = self.paths.models_dir().join(format!("{}.png", pet.id));
        if thumbnail.exists() {
            fs::remove_file(thumbnail)?;
        }
        Ok(())
    }
}
