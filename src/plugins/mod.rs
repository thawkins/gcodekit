//! Plugin system for extending gcodekit functionality.
//!
//! This module provides a plugin architecture that allows third-party extensions
//! to hook into various application events and extend functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Error(String),
}

pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn initialize(&mut self, context: &PluginContext) -> Result<(), String>;
    fn shutdown(&mut self) -> Result<(), String>;

    // Optional hooks
    fn on_gcode_loaded(&mut self, _gcode: &str) {}
    fn on_machine_connected(&mut self, _port: &str) {}
    fn on_machine_disconnected(&mut self) {}
    fn on_job_started(&mut self, _job_id: &str) {}
    fn on_job_completed(&mut self, _job_id: &str) {}
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginContext {
    pub fn new() -> Self {
        let base_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gcodekit");

        Self {
            plugin_dir: base_dir.join("plugins"),
            config_dir: base_dir.join("config"),
            data_dir: base_dir.join("data"),
        }
    }
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_states: HashMap<String, PluginState>,
    context: PluginContext,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_states: HashMap::new(),
            context: PluginContext::new(),
        }
    }

    pub fn load_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<(), String> {
        let plugin_id = plugin.info().id.clone();

        if self.plugins.contains_key(&plugin_id) {
            return Err(format!("Plugin '{}' is already loaded", plugin_id));
        }

        self.plugin_states
            .insert(plugin_id.clone(), PluginState::Loading);

        match plugin.initialize(&self.context) {
            Ok(()) => {
                self.plugins.insert(plugin_id.clone(), plugin);
                self.plugin_states.insert(plugin_id, PluginState::Loaded);
                Ok(())
            }
            Err(e) => {
                self.plugin_states
                    .insert(plugin_id, PluginState::Error(e.clone()));
                Err(format!("Failed to initialize plugin: {}", e))
            }
        }
    }

    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        if let Some(mut plugin) = self.plugins.remove(plugin_id) {
            match plugin.shutdown() {
                Ok(()) => {
                    self.plugin_states
                        .insert(plugin_id.to_string(), PluginState::Unloaded);
                    Ok(())
                }
                Err(e) => {
                    self.plugin_states
                        .insert(plugin_id.to_string(), PluginState::Error(e.clone()));
                    Err(format!("Failed to shutdown plugin: {}", e))
                }
            }
        } else {
            Err(format!("Plugin '{}' not found", plugin_id))
        }
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&dyn Plugin> {
        self.plugins.get(plugin_id).map(|b| b.as_ref())
    }

    pub fn get_plugin_state(&self, plugin_id: &str) -> PluginState {
        self.plugin_states
            .get(plugin_id)
            .cloned()
            .unwrap_or(PluginState::Unloaded)
    }

    pub fn list_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|b| b.as_ref()).collect()
    }

    pub fn list_plugin_states(&self) -> &HashMap<String, PluginState> {
        &self.plugin_states
    }

    // Event hooks
    pub fn notify_gcode_loaded(&mut self, gcode: &str) {
        for plugin in self.plugins.values_mut() {
            plugin.on_gcode_loaded(gcode);
        }
    }

    pub fn notify_machine_connected(&mut self, port: &str) {
        for plugin in self.plugins.values_mut() {
            plugin.on_machine_connected(port);
        }
    }

    pub fn notify_machine_disconnected(&mut self) {
        for plugin in self.plugins.values_mut() {
            plugin.on_machine_disconnected();
        }
    }

    pub fn notify_job_started(&mut self, job_id: &str) {
        for plugin in self.plugins.values_mut() {
            plugin.on_job_started(job_id);
        }
    }

    pub fn notify_job_completed(&mut self, job_id: &str) {
        for plugin in self.plugins.values_mut() {
            plugin.on_job_completed(job_id);
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Example plugin implementation
pub struct ExamplePlugin {
    info: PluginInfo,
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "example_plugin".to_string(),
                name: "Example Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "An example plugin demonstrating the plugin system".to_string(),
                author: "gcodekit".to_string(),
                homepage: None,
                repository: None,
                license: Some("MIT".to_string()),
                dependencies: vec![],
            },
        }
    }
}

impl Plugin for ExamplePlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, _context: &PluginContext) -> Result<(), String> {
        info!("Example plugin initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        info!("Example plugin shutdown");
        Ok(())
    }

    fn on_gcode_loaded(&mut self, gcode: &str) {
        info!(
            "Example plugin: G-code loaded, {} lines",
            gcode.lines().count()
        );
    }

    fn on_machine_connected(&mut self, port: &str) {
        info!("Example plugin: Machine connected on port {}", port);
    }
}
