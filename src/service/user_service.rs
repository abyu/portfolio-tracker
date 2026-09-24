use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::models::user::User;

pub struct UserService<U: UserRepository> {
    repo: U,
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_user_name(&self, email: String) -> Result<Option<User>, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait UserServiceTrait: Send + Sync {
    async fn validate_credentials(
        &self,
        email: String,
        password: String,
    ) -> Result<User, AuthenticationError>;
}

impl<U: UserRepository> UserService<U> {
    pub fn new(repo: U) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl<U: UserRepository> UserServiceTrait for UserService<U> {
    async fn validate_credentials(
        &self,
        email: String,
        password: String,
    ) -> Result<User, AuthenticationError> {
        let user = self
            .repo
            .get_by_user_name(email)
            .await?
            .ok_or(AuthenticationError::InvalidCredentials)?;

        let user_creds = user
            .auth
            .as_ref()
            .ok_or(AuthenticationError::InvalidCredentials)?
            .credential
            .as_ref()
            .ok_or(AuthenticationError::InvalidCredentials)?;

        let parsed_hash =
            PasswordHash::new(user_creds).map_err(|_| AuthenticationError::InvalidCredentials)?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthenticationError::InvalidCredentials)?;

        Ok(user)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("Invalid crendentials")]
    InvalidCredentials,
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error),
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;
    use crate::models::user::{User, UserAuth};

    struct MockUserRepository {
        users: HashMap<String, User>,
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn get_by_user_name(&self, email: String) -> Result<Option<User>, sqlx::Error> {
            Ok(self.users.get(&email).cloned())
        }
    }

    #[tokio::test]
    async fn test_validate_credentials_with_valid_creds() {
        let repo = MockUserRepository {
            users: [("a@b.com".to_string(), new_user("a@b.com"))]
                .into_iter()
                .collect(),
        };
        let svc = UserService::new(repo);

        let result_user = svc
            .validate_credentials("a@b.com".to_string(), "password".to_string())
            .await
            .unwrap();

        assert_eq!(result_user.id, 1)
    }

    #[tokio::test]
    async fn test_fails_to_validate_credentials_invalid_creds() {
        let repo = MockUserRepository {
            users: [("a@b.com".to_string(), new_user("a@b.com"))]
                .into_iter()
                .collect(),
        };
        let svc = UserService::new(repo);

        let result = svc
            .validate_credentials("a@b.com".to_string(), "wrong password".to_string())
            .await;

        assert!(result.is_err_and(|f| matches!(f, AuthenticationError::InvalidCredentials)))
    }

    #[tokio::test]
    async fn test_fails_to_validate_credentials_unknown_user() {
        let repo = MockUserRepository {
            users: [("a@b.com".to_string(), new_user("a@b.com"))]
                .into_iter()
                .collect(),
        };
        let svc = UserService::new(repo);

        let result = svc
            .validate_credentials("xyx@yue.com".to_string(), "wrong password".to_string())
            .await;

        assert!(result.is_err_and(|f| matches!(f, AuthenticationError::InvalidCredentials)))
    }

    fn generate_password_hash(password: &str) -> String {
        use argon2::{
            Argon2,
            password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
        };
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    fn new_user(email: &str) -> User {
        User {
            id: 1,
            email: email.to_string(),
            display_name: None,
            created_at: chrono::Utc::now(),
            auth: Some(UserAuth {
                id: 1,
                user_id: 1,
                provider: "local".to_string(),
                provider_id: "1".to_string(),
                credential: Some(generate_password_hash("password")),
                created_at: chrono::Utc::now(),
            }),
        }
    }

    #[test]
    fn print_password_hash() {
        let hash = generate_password_hash("oYoqaZ4D6FqjfMqUQjyD");
        println!("{}", hash);
    }
}
