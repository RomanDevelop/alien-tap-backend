use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    
    Ok(pool)
}
