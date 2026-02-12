#[cfg(target_os = "macos")]
use std::path::PathBuf;

use auto_launch::AutoLaunchBuilder;
use tracing::info;

use crate::error::{Error, Result};

const APP_NAME: &str = "Pet";

/// Resolves the path to use for auto-launch registration.
///
/// On macOS, if running inside an `.app` bundle, returns the bundle path
/// (e.g. `/Applications/Pet.app`). Otherwise returns the current executable.
fn resolve_app_path() -> Result<String> {
    let exe = std::env::current_exe().map_err(|e| Error::AutoStart(e.to_string()))?;
    let exe = exe
        .canonicalize()
        .map_err(|e| Error::AutoStart(e.to_string()))?;

    #[cfg(target_os = "macos")]
    {
        // If the exe lives inside a .app bundle, register the bundle itself.
        // Typical layout: Foo.app/Contents/MacOS/foo
        if let Some(path) = find_macos_bundle(&exe) {
            return Ok(path.to_string_lossy().into_owned());
        }
    }

    Ok(exe.to_string_lossy().into_owned())
}

#[cfg(target_os = "macos")]
fn find_macos_bundle(exe: &std::path::Path) -> Option<PathBuf> {
    // Walk up looking for a component ending in `.app`
    let mut current = exe.parent();
    while let Some(dir) = current {
        if dir
            .file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".app"))
        {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

/// Idempotently enable or disable auto-start at login.
///
/// Safe to call repeatedly with the same value — it only touches the OS
/// registration when the current state differs from `enabled`.
pub fn sync_autostart(enabled: bool) -> Result<()> {
    let app_path = resolve_app_path()?;
    info!("Auto-start path resolved to: {app_path}");

    let launcher = AutoLaunchBuilder::new()
        .set_app_name(APP_NAME)
        .set_app_path(&app_path)
        .build()
        .map_err(|e| Error::AutoStart(e.to_string()))?;

    let currently_enabled = launcher
        .is_enabled()
        .map_err(|e| Error::AutoStart(e.to_string()))?;

    if enabled == currently_enabled {
        info!(
            "Auto-start already {}",
            if enabled { "enabled" } else { "disabled" }
        );
        return Ok(());
    }

    if enabled {
        launcher
            .enable()
            .map_err(|e| Error::AutoStart(e.to_string()))?;
        info!("Auto-start enabled");
    } else {
        launcher
            .disable()
            .map_err(|e| Error::AutoStart(e.to_string()))?;
        info!("Auto-start disabled");
    }

    Ok(())
}
