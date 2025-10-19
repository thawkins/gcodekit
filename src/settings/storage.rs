//! Settings Storage
//!
//! Handles persistence of machine profiles and settings to disk.
//! Supports save, load, backup, and restore operations.

use super::profile::MachineProfile;
use super::{get_profiles_dir, get_settings_dir};
use anyhow::{anyhow, Result};
use chrono::Local;
use std::fs;
use std::path::Path;

/// Manages settings storage and persistence
pub struct SettingsStorage;

impl SettingsStorage {
    /// Save a machine profile to disk
    pub fn save_profile(profile: &MachineProfile) -> Result<()> {
        let profiles_dir = get_profiles_dir()?;
        let filename = format!("{}.json", sanitize_filename(&profile.name));
        let path = profiles_dir.join(filename);

        let json = serde_json::to_string_pretty(profile)?;
        fs::write(path, json)?;

        Ok(())
    }

    /// Load a machine profile from disk
    pub fn load_profile(name: &str) -> Result<MachineProfile> {
        let profiles_dir = get_profiles_dir()?;
        let filename = format!("{}.json", sanitize_filename(name));
        let path = profiles_dir.join(filename);

        if !path.exists() {
            return Err(anyhow!("Profile not found: {}", name));
        }

        let json = fs::read_to_string(path)?;
        let profile = serde_json::from_str(&json)?;

        Ok(profile)
    }

    /// List all available profiles
    pub fn list_profiles() -> Result<Vec<String>> {
        let profiles_dir = get_profiles_dir()?;

        if !profiles_dir.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        for entry in fs::read_dir(profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(filename) = path.file_stem() {
                    if let Some(name) = filename.to_str() {
                        profiles.push(name.to_string());
                    }
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    /// Delete a profile from disk
    pub fn delete_profile(name: &str) -> Result<()> {
        let profiles_dir = get_profiles_dir()?;
        let filename = format!("{}.json", sanitize_filename(name));
        let path = profiles_dir.join(filename);

        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Export a profile to a specific path
    pub fn export_profile(profile: &MachineProfile, export_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(profile)?;
        fs::write(export_path, json)?;
        Ok(())
    }

    /// Import a profile from a specific path
    pub fn import_profile(import_path: &Path) -> Result<MachineProfile> {
        if !import_path.exists() {
            return Err(anyhow!("Import file not found: {:?}", import_path));
        }

        let json = fs::read_to_string(import_path)?;
        let profile = serde_json::from_str(&json)?;

        Ok(profile)
    }

    /// Backup all profiles to a directory
    pub fn backup_all_profiles(backup_dir: &Path) -> Result<u32> {
        fs::create_dir_all(backup_dir)?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_subdir = backup_dir.join(format!("gcodekit_backup_{}", timestamp));
        fs::create_dir_all(&backup_subdir)?;

        let profiles_dir = get_profiles_dir()?;
        let mut count = 0;

        if profiles_dir.exists() {
            for entry in fs::read_dir(profiles_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    if let Some(filename) = path.file_name() {
                        let dest = backup_subdir.join(filename);
                        fs::copy(&path, dest)?;
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Restore profiles from a backup directory
    pub fn restore_profiles(backup_dir: &Path) -> Result<u32> {
        if !backup_dir.exists() {
            return Err(anyhow!("Backup directory not found: {:?}", backup_dir));
        }

        let profiles_dir = get_profiles_dir()?;
        let mut count = 0;

        for entry in fs::read_dir(backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(filename) = path.file_name() {
                    let dest = profiles_dir.join(filename);
                    fs::copy(&path, dest)?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Export all profiles to a zip or directory
    pub fn export_all_profiles(export_dir: &Path) -> Result<u32> {
        fs::create_dir_all(export_dir)?;

        let profiles_dir = get_profiles_dir()?;
        let mut count = 0;

        if profiles_dir.exists() {
            for entry in fs::read_dir(profiles_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    if let Some(filename) = path.file_name() {
                        let dest = export_dir.join(filename);
                        fs::copy(&path, dest)?;
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Import all profiles from a directory
    pub fn import_all_profiles(import_dir: &Path) -> Result<u32> {
        if !import_dir.exists() {
            return Err(anyhow!("Import directory not found: {:?}", import_dir));
        }

        let profiles_dir = get_profiles_dir()?;
        let mut count = 0;

        for entry in fs::read_dir(import_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(filename) = path.file_name() {
                    let dest = profiles_dir.join(filename);
                    fs::copy(&path, dest)?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

/// Sanitize a profile name for use as a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test/Profile"), "Test_Profile");
        assert_eq!(sanitize_filename("Profile:Name"), "Profile_Name");
        assert_eq!(sanitize_filename("Normal"), "Normal");
    }

    #[test]
    fn test_profile_save_and_load() {
        let profile =
            MachineProfile::new("TestProfile".to_string(), "CNC Mill".to_string());

        // Save profile
        let result = SettingsStorage::save_profile(&profile);
        assert!(result.is_ok());

        // Load profile
        let loaded = SettingsStorage::load_profile("TestProfile");
        assert!(loaded.is_ok());

        let loaded_profile = loaded.unwrap();
        assert_eq!(loaded_profile.name, "TestProfile");

        // Cleanup
        let _ = SettingsStorage::delete_profile("TestProfile");
    }

    #[test]
    fn test_list_profiles() {
        let profile1 = MachineProfile::new("Profile1".to_string(), "CNC".to_string());
        let profile2 = MachineProfile::new("Profile2".to_string(), "Laser".to_string());

        let _ = SettingsStorage::save_profile(&profile1);
        let _ = SettingsStorage::save_profile(&profile2);

        let profiles = SettingsStorage::list_profiles();
        assert!(profiles.is_ok());
        let list = profiles.unwrap();
        assert!(list.iter().any(|p| p == "Profile1"));
        assert!(list.iter().any(|p| p == "Profile2"));

        // Cleanup
        let _ = SettingsStorage::delete_profile("Profile1");
        let _ = SettingsStorage::delete_profile("Profile2");
    }

    #[test]
    fn test_delete_profile() {
        let profile = MachineProfile::new("DeleteMe".to_string(), "CNC".to_string());
        let _ = SettingsStorage::save_profile(&profile);

        let result = SettingsStorage::delete_profile("DeleteMe");
        assert!(result.is_ok());

        let loaded = SettingsStorage::load_profile("DeleteMe");
        assert!(loaded.is_err());
    }
}
