use error_stack::{IntoReport, Report, ResultExt};
use sqlx::{query, query_as, types::Json, PgPool};

use crate::models::{ModelCategory, ModelDefinition, ModelParams};

use super::DbError;

pub async fn list_models(pool: &PgPool) -> Result<Vec<ModelDefinition>, Report<DbError>> {
    query_as!(
        ModelDefinition,
        r##"SELECT id,
            name,
            category as "category: ModelCategory",
            params as "params: ModelParams"
            FROM models"##
    )
    .fetch_all(pool)
    .await
    .into_report()
    .change_context(DbError {})
}

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn list() {
        dotenvy::dotenv().ok();
        let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        let models = super::list_models(&pool).await.unwrap();

        println!("{:?}", models);
    }
}
