use crate::error::{Error, Result};
use crate::models::{Pet, WindowPosition};
use crate::paths::AppPaths;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use tracing::info;
use uuid::Uuid;

// --- Settings group structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_scale")]
    pub pet_scale: f32,
    #[serde(default)]
    pub pet_position: WindowPosition,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_scale() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_opacity() -> f32 {
    1.0
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            pet_scale: default_scale(),
            pet_position: WindowPosition::default(),
            always_on_top: default_true(),
            opacity: default_opacity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub provider: AiProvider,
    pub api_key: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    pub endpoint: Option<String>,
    #[serde(default)]
    pub personality: PersonalityConfig,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AiProvider::default(),
            api_key: None,
            model: default_model(),
            endpoint: None,
            personality: PersonalityConfig::default(),
            temperature: default_temperature(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum AiProvider {
    #[default]
    OpenAi,
    Anthropic,
    Ollama,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    #[serde(default = "default_pet_name")]
    pub name: String,
    #[serde(default)]
    pub traits: Vec<String>,
    pub custom_prompt: Option<String>,
}

fn default_pet_name() -> String {
    "Pet".to_string()
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            name: default_pet_name(),
            traits: vec![],
            custom_prompt: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeshySettings {
    pub api_key: Option<String>,
}

// --- Main config struct ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub ai: AiSettings,
    #[serde(default)]
    pub meshy: MeshySettings,
    #[serde(default)]
    pub pets: Vec<Pet>,
    pub active_pet: Option<Uuid>,
}

// --- Legacy config for migration ---

#[derive(Debug, Deserialize)]
struct LegacyAppConfig {
    #[serde(default)]
    pets: Vec<Pet>,
    active_pet: Option<Uuid>,
    #[serde(default)]
    pet_position: WindowPosition,
    #[serde(default = "default_scale")]
    pet_scale: f32,
    #[serde(default)]
    auto_start: bool,
    meshy_api_key: Option<String>,
}

fn migrate_from_legacy(legacy: LegacyAppConfig) -> AppConfig {
    AppConfig {
        general: GeneralSettings {
            auto_start: legacy.auto_start,
            ..Default::default()
        },
        appearance: AppearanceSettings {
            pet_scale: legacy.pet_scale,
            pet_position: legacy.pet_position,
            ..Default::default()
        },
        ai: AiSettings::default(),
        meshy: MeshySettings {
            api_key: legacy.meshy_api_key,
        },
        pets: legacy.pets,
        active_pet: legacy.active_pet,
    }
}

impl AppConfig {
    pub fn load(paths: &AppPaths) -> Result<Self> {
        let config_file = paths.config_file();
        if !config_file.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_file)?;
        let raw: toml::Value =
            toml::from_str(&content).map_err(|e| Error::InvalidConfig(e.to_string()))?;

        // Detect legacy flat format by top-level keys
        let is_legacy = raw.get("pet_scale").is_some()
            || raw.get("auto_start").is_some()
            || raw.get("meshy_api_key").is_some();

        if is_legacy {
            let legacy: LegacyAppConfig =
                toml::from_str(&content).map_err(|e| Error::InvalidConfig(e.to_string()))?;
            let migrated = migrate_from_legacy(legacy);
            migrated.save(paths)?;
            info!("Migrated config from legacy flat format");
            Ok(migrated)
        } else {
            let settings = Config::builder()
                .add_source(File::from(config_file))
                .build()
                .map_err(|e| Error::InvalidConfig(e.to_string()))?;

            settings
                .try_deserialize()
                .map_err(|e| Error::InvalidConfig(e.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::AppPaths;
    use tempfile::TempDir;

    fn test_paths() -> (TempDir, AppPaths) {
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::with_dirs(dir.path().join("config"), dir.path().join("data"));
        (dir, paths)
    }

    #[test]
    fn migrate_legacy_config() {
        let legacy_toml = r#"
auto_start = true
pet_scale = 1.5
meshy_api_key = "test-key"

[pet_position]
x = 100
y = 200
"#;
        let legacy: LegacyAppConfig = toml::from_str(legacy_toml).unwrap();
        let migrated = migrate_from_legacy(legacy);
        assert!(migrated.general.auto_start);
        assert_eq!(migrated.appearance.pet_scale, 1.5);
        assert_eq!(migrated.appearance.pet_position.x, 100);
        assert_eq!(migrated.appearance.pet_position.y, 200);
        assert_eq!(migrated.meshy.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn new_format_round_trips() {
        let config = AppConfig {
            general: GeneralSettings {
                auto_start: true,
                language: "zh".to_string(),
            },
            appearance: AppearanceSettings {
                pet_scale: 2.0,
                always_on_top: false,
                opacity: 0.8,
                ..Default::default()
            },
            ..Default::default()
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert!(deserialized.general.auto_start);
        assert_eq!(deserialized.general.language, "zh");
        assert_eq!(deserialized.appearance.pet_scale, 2.0);
        assert!(!deserialized.appearance.always_on_top);
    }

    #[test]
    fn default_config_round_trips() {
        let config = AppConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.appearance.pet_scale, 1.0);
        assert_eq!(deserialized.general.language, "en");
        assert!(!deserialized.general.auto_start);
    }

    #[test]
    fn load_legacy_file_migrates() {
        let (_dir, paths) = test_paths();
        paths.ensure_dirs().unwrap();

        let legacy_content = r#"
auto_start = true
pet_scale = 1.5
meshy_api_key = "key123"

[pet_position]
x = 50
y = 75
"#;
        fs::write(paths.config_file(), legacy_content).unwrap();

        let config = AppConfig::load(&paths).unwrap();
        assert!(config.general.auto_start);
        assert_eq!(config.appearance.pet_scale, 1.5);
        assert_eq!(config.meshy.api_key, Some("key123".to_string()));

        // Verify file was rewritten in new format
        let reloaded = AppConfig::load(&paths).unwrap();
        assert!(reloaded.general.auto_start);
        assert_eq!(reloaded.appearance.pet_scale, 1.5);
    }

    #[test]
    fn load_new_format_file() {
        let (_dir, paths) = test_paths();
        paths.ensure_dirs().unwrap();

        let new_content = r#"
[general]
auto_start = false
language = "zh"

[appearance]
pet_scale = 2.0
always_on_top = true
opacity = 0.9

[ai]
enabled = false

[meshy]
"#;
        fs::write(paths.config_file(), new_content).unwrap();

        let config = AppConfig::load(&paths).unwrap();
        assert!(!config.general.auto_start);
        assert_eq!(config.general.language, "zh");
        assert_eq!(config.appearance.pet_scale, 2.0);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let (_dir, paths) = test_paths();
        let config = AppConfig::load(&paths).unwrap();
        assert_eq!(config.appearance.pet_scale, 1.0);
        assert_eq!(config.general.language, "en");
    }
}
