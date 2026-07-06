use chrono::Utc;

use crate::{
    models::user::{User, UserAuth},
    service::user_service::UserRepository,
};

pub struct PostgresUserRepository {
    db_pool: sqlx::PgPool,
}

impl PostgresUserRepository {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn get_by_user_name(&self, email: String) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserWithAuthRow>(
            "SELECT 
            u.id, u.email, u.display_name, u.created_at,
            ua.id as auth_id,
            ua.provider,
            ua.provider_id,
            ua.credential,
            ua.created_at as auth_created_at
        FROM users u
        LEFT JOIN user_auth ua ON ua.user_id = u.id AND ua.provider = 'local'
        WHERE u.email = $1",
        )
        .bind(email)
        .fetch_optional(&self.db_pool)
        .await?
        .map(|r| User {
            id: r.id,
            email: r.email,
            display_name: r.display_name,
            created_at: r.created_at,
            auth: r.auth_id.map(|id| UserAuth {
                id,
                user_id: r.id,
                provider: r.provider.unwrap_or_default(),
                provider_id: r.provider_id.unwrap_or_default(),
                credential: r.credential,
                created_at: r.auth_created_at.unwrap_or_else(Utc::now),
            }),
        });

        Ok(user)
    }
}

#[derive(sqlx::FromRow)]
struct UserWithAuthRow {
    id: i64,
    email: String,
    display_name: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    auth_id: Option<i64>,
    provider: Option<String>,
    provider_id: Option<String>,
    credential: Option<String>,
    auth_created_at: Option<chrono::DateTime<chrono::Utc>>,
}
