use std::fmt;
pub struct AppConfig {
    pub server_port: u16
}
const DEFAULT_PORT: u16 = 3000;

impl AppConfig {
    pub fn load() -> Result<AppConfig, ConfigError> {
        dotenvy::dotenv().ok();
        let port = if let Ok(server_port) = std::env::var("PORT") {
            if let Ok(v_port) = server_port.parse() {
                v_port
            } else {
                return Err(ConfigError::InvalidPort(server_port));
            }
        } else {
            DEFAULT_PORT
        };
        Ok(AppConfig { server_port: port })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidPort(String)
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidPort(val) => write!(f, "Invalid port value: {}", val),
        }
    }
}

impl std::error::Error for ConfigError {}