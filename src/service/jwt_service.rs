use chrono::Utc;

use crate::models::claims::Claims;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: i64) -> Result<String, JwtError> {
        let exp = (Utc::now() + chrono::Duration::days(7)).timestamp() as usize;
        let claims = Claims {
            sub: user_id,
            exp: exp,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::TokenGenerationError(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode(
            &token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|d| d.claims)
        .map_err(|e| JwtError::InvalidToken(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Failed to validate token {0}")]
    InvalidToken(String),
    #[error("Error genarating token {0}")]
    TokenGenerationError(String),
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_and_validate_token_with_user_id() {
        let svc = JwtService::new("a small secret".to_string());

        let token = svc.generate_token(2).unwrap();

        let claims = svc.validate_token(&token).unwrap();

        assert_eq!(claims.sub, 2);
    }

    #[test]
    fn test_fails_to_validate_invalid_token() {
        let svc = JwtService::new("a small secret".to_string());

        let result = svc.validate_token("some garbage");

        assert!(result.is_err_and(|f| matches!(f, JwtError::InvalidToken(_))));
    }

}