// Chapter 12, Exercise 2: Configuration Module


#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub max_connections: usize,
}

mod development {
    use super::Config;
    
    pub fn config() -> Config {
        Config {
            database_url: "postgres://localhost:5432/dev_db".to_string(),
            port: 3000,
            debug_mode: true,
            max_connections: 10,
        }
    }
}

mod production {
    use super::Config;
    
    pub fn config() -> Config {
        Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://prod-server:5432/prod_db".to_string()),
            port: 8080,
            debug_mode: false,
            max_connections: 100,
        }
    }
}

mod test {
    use super::Config;
    
    pub fn config() -> Config {
        Config {
            database_url: "postgres://localhost:5432/test_db".to_string(),
            port: 3001,
            debug_mode: true,
            max_connections: 5,
        }
    }
}

pub enum Environment {
    Development,
    Production,
    Test,
}

impl Config {
    pub fn load() -> Config {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        
        match env.as_str() {
            "production" | "prod" => production::config(),
            "test" => test::config(),
            _ => development::config(),
        }
    }
    
    pub fn for_environment(env: Environment) -> Config {
        match env {
            Environment::Development => development::config(),
            Environment::Production => production::config(),
            Environment::Test => test::config(),
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.database_url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }
        
        if self.port == 0 {
            errors.push("Port cannot be 0".to_string());
        }
        
        if self.max_connections == 0 {
            errors.push("Max connections must be at least 1".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Additional configuration utilities
pub mod utils {
    use super::Config;
    
    pub fn connection_string(config: &Config) -> String {
        format!("{}?max_connections={}", config.database_url, config.max_connections)
    }
    
    pub fn server_address(config: &Config) -> String {
        format!("0.0.0.0:{}", config.port)
    }
}