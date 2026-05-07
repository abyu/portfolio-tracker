#[derive(Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub db_url: String
}
const DEFAULT_PORT: u16 = 3000;

impl AppConfig {
    pub fn load() -> Result<AppConfig, ConfigError> {
        dotenvy::dotenv().ok();
        let port = Self::get_env_u16_or_default("SERVER_PORT", DEFAULT_PORT)?;
        let db_url = Self::get_env_string("DATABASE_URL")?;
        Ok(AppConfig { server_port: port, db_url })
    }

    fn get_env_string(key: &str) -> Result<String, ConfigError> {
        if let Ok(val) = std::env::var(key) {
            Ok(val)
        } else {
            Err(ConfigError::MissingValue(key.to_string()))
        }
    }

    fn get_env_u16_or_default(key: &str, default: u16) -> Result<u16, ConfigError> {
        if let Ok(val) = std::env::var(key) {
            if let Ok(v_port) = val.parse() {
                Ok(v_port)
            } else {
                Err(ConfigError::InvalidValue(val))
            }
        } else {
            Ok(default)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
     #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Missing config value for: {0}")]
    MissingValue(String)
}