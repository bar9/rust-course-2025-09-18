// Chapter 12, Exercise 3: Plugin Architecture


pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self);
    fn version(&self) -> &str {
        "1.0.0"
    }
}

pub mod plugins {
    use super::Plugin;
    
    pub mod logger {
        use super::Plugin;
        
        pub struct LoggerPlugin {
            name: String,
            level: LogLevel,
        }
        
        #[derive(Debug)]
        pub enum LogLevel {
            Debug,
            Info,
            Warn,
            Error,
        }
        
        impl LoggerPlugin {
            pub fn new(name: String, level: LogLevel) -> Self {
                LoggerPlugin { name, level }
            }
        }
        
        impl Plugin for LoggerPlugin {
            fn name(&self) -> &str {
                &self.name
            }
            
            fn execute(&self) {
                println!("[{}] Logger plugin executing at level {:?}", self.name, self.level);
            }
            
            fn version(&self) -> &str {
                "2.0.0"
            }
        }
    }
    
    pub mod metrics {
        use super::Plugin;
        use std::time::SystemTime;
        
        pub struct MetricsPlugin {
            name: String,
            start_time: SystemTime,
            metrics_collected: u64,
        }
        
        impl MetricsPlugin {
            pub fn new(name: String) -> Self {
                MetricsPlugin {
                    name,
                    start_time: SystemTime::now(),
                    metrics_collected: 0,
                }
            }
            
            pub fn collect(&mut self) {
                self.metrics_collected += 1;
            }
        }
        
        impl Plugin for MetricsPlugin {
            fn name(&self) -> &str {
                &self.name
            }
            
            fn execute(&self) {
                let uptime = self.start_time.elapsed()
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                
                println!("[{}] Metrics plugin - Uptime: {}s, Metrics collected: {}", 
                        self.name, uptime, self.metrics_collected);
            }
        }
    }
    
    pub mod auth {
        use super::Plugin;
        
        pub struct AuthPlugin {
            name: String,
            auth_type: AuthType,
        }
        
        pub enum AuthType {
            Basic,
            OAuth,
            JWT,
        }
        
        impl AuthPlugin {
            pub fn new(name: String, auth_type: AuthType) -> Self {
                AuthPlugin { name, auth_type }
            }
        }
        
        impl Plugin for AuthPlugin {
            fn name(&self) -> &str {
                &self.name
            }
            
            fn execute(&self) {
                let auth_str = match self.auth_type {
                    AuthType::Basic => "Basic Authentication",
                    AuthType::OAuth => "OAuth 2.0",
                    AuthType::JWT => "JWT Token",
                };
                
                println!("[{}] Auth plugin using {}", self.name, auth_str);
            }
        }
    }
}

pub mod registry {
    use super::Plugin;
    use std::collections::HashMap;
    
    pub struct PluginRegistry {
        plugins: HashMap<String, Box<dyn Plugin>>,
        execution_order: Vec<String>,
    }
    
    impl Default for PluginRegistry {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PluginRegistry {
        pub fn new() -> Self {
            PluginRegistry {
                plugins: HashMap::new(),
                execution_order: Vec::new(),
            }
        }
        
        pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), String> {
            let name = plugin.name().to_string();
            
            if self.plugins.contains_key(&name) {
                return Err(format!("Plugin '{}' already registered", name));
            }
            
            println!("Registering plugin: {} (version {})", name, plugin.version());
            self.plugins.insert(name.clone(), plugin);
            self.execution_order.push(name);
            
            Ok(())
        }
        
        pub fn unregister(&mut self, name: &str) -> Result<(), String> {
            if self.plugins.remove(name).is_some() {
                self.execution_order.retain(|n| n != name);
                Ok(())
            } else {
                Err(format!("Plugin '{}' not found", name))
            }
        }
        
        pub fn execute_all(&self) {
            println!("\nExecuting {} plugins:", self.plugins.len());
            println!("{}", "=".repeat(40));
            
            for name in &self.execution_order {
                if let Some(plugin) = self.plugins.get(name) {
                    plugin.execute();
                }
            }
            
            println!("{}", "=".repeat(40));
        }
        
        pub fn execute(&self, name: &str) -> Result<(), String> {
            self.plugins
                .get(name)
                .map(|plugin| plugin.execute())
                .ok_or_else(|| format!("Plugin '{}' not found", name))
        }
        
        pub fn list_plugins(&self) -> Vec<String> {
            self.execution_order.clone()
        }
        
        pub fn get_plugin_info(&self, name: &str) -> Option<(String, String)> {
            self.plugins.get(name).map(|p| {
                (p.name().to_string(), p.version().to_string())
            })
        }
    }
}