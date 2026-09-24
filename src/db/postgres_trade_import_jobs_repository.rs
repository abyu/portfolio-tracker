use crate::{
    models::trade_confirmation_import_job::NewTradeConfirmationImportJob,
    service::jobs_service::{JobsError, JobsRepository},
};

pub struct PostgresTradeImportJobsRepository {
    db_pool: sqlx::PgPool,
}

impl PostgresTradeImportJobsRepository {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl JobsRepository for PostgresTradeImportJobsRepository {
    async fn add_job(&self, job: NewTradeConfirmationImportJob) -> Result<i64, JobsError> {
        let result = sqlx::query_scalar("INSERT INTO trade_confirmation_import_jobs (user_id, file_name, file_content) VALUES ($1, $2, $3) RETURNING id")
        .bind(job.user_id)
        .bind(job.file_name)
        .bind(job.file_content)
        .fetch_one(&self.db_pool)
        .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_user(pool: &sqlx::PgPool) -> i64 {
        sqlx::query_scalar("INSERT INTO users (email, display_name) VALUES ($1, $2) RETURNING id")
            .bind("test@example.com")
            .bind("Test User")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_job_added_to_db_successfully(pool: sqlx::PgPool) {
        let repo = PostgresTradeImportJobsRepository::new(pool.clone());
        let test_user = create_test_user(&pool).await;
        let res = repo
            .add_job(NewTradeConfirmationImportJob {
                user_id: test_user,
                file_name: "same_file".to_string(),
                file_content: "hello".as_bytes().to_vec(),
            })
            .await;

        assert!(res.is_ok())
    }
}
