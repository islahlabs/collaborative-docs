use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;
use tracing::warn;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

fn default_max_connections() -> u32 { 10 }
fn default_min_connections() -> u32 { 2 }

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: None,
                host: Some("localhost".to_string()),
                port: Some(5432),
                username: Some("postgres".to_string()),
                password: Some("password".to_string()),
                database: Some("collaborative_docs".to_string()),
                max_connections: 10,
                min_connections: 2,
            },
            cors: CorsConfig {
                allowed_origins: vec!["http://localhost:5173".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()],
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        tracing::info!("Running in {} mode", run_mode);

        let config = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("database.host", "localhost")?
            .set_default("database.port", 5432)?
            .set_default("database.username", "postgres")?
            .set_default("database.password", "password")?
            .set_default("database.database", "collaborative_docs")?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 2)?
            .set_default("cors.allowed_origins", vec!["http://localhost:5173"])?
            .set_default("cors.allowed_methods", vec!["GET", "POST", "PUT"])?
            // Load config files
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Load from environment variables for overrides
            .add_source(Environment::with_prefix("APP").separator("__").ignore_empty(true))
            .build()?;

        let mut app_config: AppConfig = config.try_deserialize()?;

        // Override with DATABASE_URL if provided (for compatibility with cloud platforms)
        if let Ok(database_url) = env::var("DATABASE_URL") {
            tracing::info!("Using DATABASE_URL from environment");
            app_config.database = Self::parse_database_url(&database_url)?;
        }

        // Validate configuration
        app_config.validate()?;

        tracing::info!("Configuration loaded successfully");
        tracing::debug!("Server: {}:{}", app_config.server.host, app_config.server.port);
        if let Some(url) = &app_config.database.url {
            tracing::debug!("Database URL: {}", url);
        } else {
            tracing::debug!("Database: {}:{}/{}", 
                app_config.database.host.as_deref().unwrap_or("unknown"),
                app_config.database.port.unwrap_or(0),
                app_config.database.database.as_deref().unwrap_or("unknown")
            );
        }

        Ok(app_config)
    }

    fn parse_database_url(url: &str) -> Result<DatabaseConfig, config::ConfigError> {
        // Parse PostgreSQL URL: postgresql://username:password@host:port/database
        if !url.starts_with("postgresql://") {
            return Err(config::ConfigError::NotFound("Invalid database URL format".to_string()));
        }

        let url = url.trim_start_matches("postgresql://");
        
        // Split into credentials and host parts
        let (credentials, rest) = url.split_once('@')
            .ok_or_else(|| config::ConfigError::NotFound("Invalid database URL format".to_string()))?;
        
        // Parse credentials
        let (username, password) = credentials.split_once(':')
            .ok_or_else(|| config::ConfigError::NotFound("Invalid database URL format".to_string()))?;
        
        // Parse host and database
        let (host_port, database) = rest.split_once('/')
            .ok_or_else(|| config::ConfigError::NotFound("Invalid database URL format".to_string()))?;
        
        let (host, port) = if host_port.contains(':') {
            let (host, port) = host_port.split_once(':')
                .ok_or_else(|| config::ConfigError::NotFound("Invalid database URL format".to_string()))?;
            (host, port.parse::<u16>().unwrap_or(5432))
        } else {
            (host_port, 5432)
        };

        Ok(DatabaseConfig {
            url: None,
            host: Some(host.to_string()),
            port: Some(port),
            username: Some(username.to_string()),
            password: Some(password.to_string()),
            database: Some(database.to_string()),
            max_connections: 10,
            min_connections: 2,
        })
    }

    fn validate(&self) -> Result<(), config::ConfigError> {
        // Validate server config
        if self.server.port == 0 {
            return Err(config::ConfigError::NotFound("Server port cannot be 0".to_string()));
        }

        // Validate database config
        if let Some(port) = self.database.port {
            if port == 0 {
                return Err(config::ConfigError::NotFound("Database port cannot be 0".to_string()));
            }
        }

        if let Some(username) = &self.database.username {
            if username.is_empty() {
                return Err(config::ConfigError::NotFound("Database username cannot be empty".to_string()));
            }
        }

        if let Some(database) = &self.database.database {
            if database.is_empty() {
                return Err(config::ConfigError::NotFound("Database name cannot be empty".to_string()));
            }
        }

        // Validate CORS config
        if self.cors.allowed_origins.is_empty() {
            warn!("No CORS origins configured, API will not be accessible from browsers");
        }

        Ok(())
    }

    pub fn database_url(&self) -> String {
        if let Some(url) = &self.database.url {
            url.clone()
        } else {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.database.username.as_deref().unwrap_or("postgres"),
                self.database.password.as_deref().unwrap_or("password"),
                self.database.host.as_deref().unwrap_or("localhost"),
                self.database.port.unwrap_or(5432),
                self.database.database.as_deref().unwrap_or("collaborative_docs")
            )
        }
    }

    pub fn is_production(&self) -> bool {
        env::var("RUN_MODE").unwrap_or_else(|_| "development".into()) == "production"
    }
} 