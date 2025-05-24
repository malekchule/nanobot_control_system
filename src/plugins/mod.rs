// Branch: plugin-system
// File: src/plugins/mod.rs

use std::collections::HashMap;
use anyhow::Result;

pub trait Plugin {
    fn name(&self) -> &'static str;
    fn execute(&self, input: &str) -> Result<String>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin<P: Plugin + Send + Sync + 'static>(&mut self, plugin: P) {
        self.plugins.insert(plugin.name().to_string(), Box::new(plugin));
    }

    pub fn execute_plugin(&self, name: &str, input: &str) -> Result<String> {
        match self.plugins.get(name) {
            Some(plugin) => plugin.execute(input),
            None => Err(anyhow::anyhow!("Plugin '{}' not found", name)),
        }
    }

    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

// Example Plugin
pub struct EchoPlugin;

impl Plugin for EchoPlugin {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn execute(&self, input: &str) -> Result<String> {
        Ok(format!("Echo: {}", input))
    }
}

