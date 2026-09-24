#[derive(Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub db_url: String,
    pub jwt_secret: String,
    pub ollama_url: String,
    pub ollama_model: String,
}
const DEFAULT_PORT: u16 = 3000;
static DEFAULT_MODEL: &str = "qwen3-vl:2b";

impl AppConfig {
    pub fn load() -> Result<AppConfig, ConfigError> {
        dotenvy::dotenv().ok();
        let port = Self::get_env_u16_or_default("SERVER_PORT", DEFAULT_PORT)?;
        let db_url = Self::get_env_string("DATABASE_URL")?;
        let jwt_secret = Self::get_env_string("JWT_SECRET")?;
        let ollama_url = Self::get_env_string("OLLAMA_URL")?;
        let ollama_model =
            Self::get_env_string_or_default("OLLAMA_MODEL", DEFAULT_MODEL.to_string());
        Ok(AppConfig {
            server_port: port,
            db_url,
            jwt_secret,
            ollama_url,
            ollama_model,
        })
    }

    fn get_env_string(key: &str) -> Result<String, ConfigError> {
        if let Ok(val) = std::env::var(key) {
            Ok(val)
        } else {
            Err(ConfigError::MissingValue(key.to_string()))
        }
    }

    fn get_env_string_or_default(key: &str, default: String) -> String {
        if let Ok(val) = std::env::var(key) {
            val
        } else {
            default
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
    MissingValue(String),
}
