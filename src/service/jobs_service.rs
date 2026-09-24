use crate::models::trade_confirmation_import_job::NewTradeConfirmationImportJob;

pub struct JobsService {

}

#[async_trait::async_trait]
pub trait JobsRepository {
    async fn add_job(&self, job: NewTradeConfirmationImportJob) -> Result<i64, JobsError>;
}

#[derive(Debug, thiserror::Error)]
pub enum JobsError {
    #[error("Db error {0}")]
    DBError(#[from] sqlx::Error)
}