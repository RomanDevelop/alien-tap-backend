use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Claim {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub status: String,
    #[sqlx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateClaimRequest {
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmClaimRequest {
    pub claim_id: Uuid,
}
