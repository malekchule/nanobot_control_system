use anyhow::Result;
use std::collections::HashMap;

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

    pub fn register_plugin<P: Plugin + Send + Sync + 'static>(&mut self, plugin: P) -> Result<()> {
        let name = plugin.name();

        if self.plugins.contains_key(name) {
            anyhow::bail!("Plugin '{}' is already registered", name);
        }

        self.plugins.insert(name.to_string(), Box::new(plugin));
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    struct ReplacementPlugin;

    impl Plugin for ReplacementPlugin {
        fn name(&self) -> &'static str {
            "echo"
        }

        fn execute(&self, _: &str) -> Result<String> {
            Ok("replacement".to_string())
        }
    }

    #[test]
    fn duplicate_registration_preserves_original_plugin() {
        let mut manager = PluginManager::new();

        manager.register_plugin(EchoPlugin).unwrap();

        let error = manager.register_plugin(ReplacementPlugin).unwrap_err();

        assert!(error.to_string().contains("already registered"));
        assert_eq!(
            manager.execute_plugin("echo", "hello").unwrap(),
            "Echo: hello"
        );
        assert_eq!(manager.list_plugins().len(), 1);
    }
}
