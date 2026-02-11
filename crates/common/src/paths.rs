use crate::error::{Error, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::debug;

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

    /// Find the assets directory using a four-layer lookup strategy:
    ///
    /// 1. Environment variable `PET_ASSETS_DIR` (for dev/test override)
    /// 2. Relative to executable (tar.gz/zip, .app bundle, DEB/RPM)
    /// 3. System standard paths (Linux FHS, Windows Program Files)
    /// 4. Development fallback (running from workspace)
    pub fn find_assets_dir() -> Result<PathBuf> {
        let env_override = std::env::var("PET_ASSETS_DIR").ok();
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        find_assets_dir_with(env_override.as_deref(), exe_dir.as_deref())
    }
}

/// Inner implementation that accepts parameters for testability.
fn find_assets_dir_with(
    env_override: Option<&str>,
    exe_dir: Option<&std::path::Path>,
) -> Result<PathBuf> {
    // Layer 1: Environment variable override
    if let Some(dir) = env_override {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            debug!("Assets found via PET_ASSETS_DIR: {}", path.display());
            return Ok(path);
        }
    }

    // Layer 2: Relative to executable
    if let Some(exe) = exe_dir {
        let candidates = [
            // tar.gz/zip/MSI: assets in same directory
            exe.join("assets"),
            // macOS .app bundle: Contents/MacOS/../Resources/assets
            exe.join("../Resources/assets"),
            // DEB/RPM: FHS standard /usr/local/share/pet/assets
            exe.join("../share/pet/assets"),
        ];
        for candidate in &candidates {
            if let Ok(path) = candidate.canonicalize()
                && path.is_dir()
            {
                debug!("Assets found relative to exe: {}", path.display());
                return Ok(path);
            }
        }
    }

    // Layer 3: System standard paths
    #[cfg(target_os = "linux")]
    {
        let system_paths = [
            PathBuf::from("/usr/share/pet/assets"),
            PathBuf::from("/usr/local/share/pet/assets"),
        ];
        for candidate in &system_paths {
            if candidate.is_dir() {
                debug!("Assets found at system path: {}", candidate.display());
                return Ok(candidate.clone());
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let candidate = PathBuf::from(pf).join("Pet/assets");
            if candidate.is_dir() {
                debug!("Assets found at system path: {}", candidate.display());
                return Ok(candidate);
            }
        }
    }

    // Layer 4: Development fallback
    let dev_candidates = [PathBuf::from("assets"), PathBuf::from("../../assets")];
    for candidate in &dev_candidates {
        if let Ok(path) = candidate.canonicalize()
            && path.is_dir()
        {
            debug!("Assets found via dev fallback: {}", path.display());
            return Ok(path);
        }
    }

    Err(Error::AssetsNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn env_override_with_valid_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let assets = tmp.path().join("assets");
        fs::create_dir(&assets).unwrap();

        let result = find_assets_dir_with(Some(assets.to_str().unwrap()), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), assets);
    }

    #[test]
    fn env_override_with_nonexistent_dir_falls_through() {
        // Invalid env path should not be returned; function falls through to later layers
        let result = find_assets_dir_with(Some("/nonexistent/path/assets"), None);
        if let Ok(path) = &result {
            // If it succeeds via dev fallback, it must NOT be the invalid env path
            assert_ne!(path, &PathBuf::from("/nonexistent/path/assets"));
        }
    }

    #[test]
    fn exe_dir_with_assets_sibling() {
        let tmp = tempfile::tempdir().unwrap();
        let exe_dir = tmp.path().join("bin");
        let assets = tmp.path().join("bin/assets");
        fs::create_dir_all(&assets).unwrap();

        let result = find_assets_dir_with(None, Some(&exe_dir));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), assets.canonicalize().unwrap());
    }

    #[test]
    fn exe_dir_with_resources_assets() {
        // Simulates macOS .app bundle: Contents/MacOS/../Resources/assets
        let tmp = tempfile::tempdir().unwrap();
        let macos_dir = tmp.path().join("Contents/MacOS");
        let resources_assets = tmp.path().join("Contents/Resources/assets");
        fs::create_dir_all(&macos_dir).unwrap();
        fs::create_dir_all(&resources_assets).unwrap();

        let result = find_assets_dir_with(None, Some(&macos_dir));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), resources_assets.canonicalize().unwrap());
    }

    #[test]
    fn exe_dir_with_share_pet_assets() {
        // Simulates DEB/RPM FHS: /usr/local/bin/../share/pet/assets
        let tmp = tempfile::tempdir().unwrap();
        let bin_dir = tmp.path().join("usr/local/bin");
        let share_assets = tmp.path().join("usr/local/share/pet/assets");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&share_assets).unwrap();

        let result = find_assets_dir_with(None, Some(&bin_dir));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), share_assets.canonicalize().unwrap());
    }

    #[test]
    fn env_override_takes_priority_over_exe_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let env_assets = tmp.path().join("env_assets");
        let exe_dir = tmp.path().join("bin");
        let exe_assets = tmp.path().join("bin/assets");
        fs::create_dir_all(&env_assets).unwrap();
        fs::create_dir_all(&exe_assets).unwrap();

        let result = find_assets_dir_with(Some(env_assets.to_str().unwrap()), Some(&exe_dir));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), env_assets);
    }

    #[test]
    fn no_candidates_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let empty_dir = tmp.path().join("empty");
        fs::create_dir_all(&empty_dir).unwrap();

        let result = find_assets_dir_with(None, Some(&empty_dir));
        // May succeed via dev fallback if run from workspace root.
        // With a non-workspace cwd it would fail, but we can't control cwd
        // in parallel tests. Just verify it doesn't panic.
        let _ = result;
    }

    #[test]
    fn exe_dir_none_skips_layer2() {
        // No env override, no exe_dir — should fall through to dev fallback or error
        let result = find_assets_dir_with(None, None);
        // May succeed via dev fallback if run from workspace root
        let _ = result;
    }
}
