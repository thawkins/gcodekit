//! Settings Management Module
//!
//! Provides comprehensive machine profile and settings management including:
//! - Save/load GRBL machine profiles
//! - Multi-machine support with profile switching
//! - Settings backup/restore functionality
//! - Configuration templates
//! - Persistent storage across sessions
//!
//! Profiles are stored as JSON files in the application config directory.

pub mod profile;
pub mod storage;

pub use profile::{MachineProfile, ProfileManager, ProfileSettings};
pub use storage::SettingsStorage;

use anyhow::Result;
use std::path::PathBuf;

/// Global settings directory for the application
pub fn get_settings_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
    let gcodekit_dir = config_dir.join("gcodekit");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&gcodekit_dir)?;

    Ok(gcodekit_dir)
}

/// Get path for machine profiles directory
pub fn get_profiles_dir() -> Result<PathBuf> {
    let settings_dir = get_settings_dir()?;
    let profiles_dir = settings_dir.join("profiles");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&profiles_dir)?;

    Ok(profiles_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_dir_creation() {
        let result = get_settings_dir();
        assert!(result.is_ok());
    }

    #[test]
    fn test_profiles_dir_creation() {
        let result = get_profiles_dir();
        assert!(result.is_ok());
    }
}
