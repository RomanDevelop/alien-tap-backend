use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Score {
    pub id: Uuid,
    pub user_id: Uuid,
    pub score: i32,
    #[sqlx(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScoreRequest {
    pub score: i32,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub score: i32,
}
