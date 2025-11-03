use axum::{
    extract::State,
    http::HeaderMap,
    response::Json,
    routing::post,
    Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::models::claim::{CreateClaimRequest, ConfirmClaimRequest};
use crate::utils::errors::AppError;
use crate::utils::jwt;

#[derive(Debug, Serialize)]
pub struct CreateClaimResponse {
    pub claim_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ConfirmClaimResponse {
    pub success: bool,
    pub status: String,
}

// Извлекаем JWT токен из заголовка Authorization
async fn extract_user_id(
    headers: &HeaderMap,
    jwt_secret: &str,
) -> Result<Uuid, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized)?;
    
    let claims = jwt::verify_jwt(token, jwt_secret)
        .map_err(|_| AppError::Unauthorized)?;
    
    Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Validation("Invalid user ID in token".to_string()))
}

async fn create_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateClaimRequest>,
) -> Result<Json<CreateClaimResponse>, AppError> {
    let user_id = extract_user_id(&headers, &state.config.jwt_secret).await?;
    
    let claim_id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO claims (id, user_id, amount, status)
        VALUES ($1, $2, $3::DECIMAL, 'pending')
        "#,
    )
    .bind(claim_id)
    .bind(user_id)
    .bind(payload.amount.to_string())
    .execute(&state.pool)
    .await?;
    
    Ok(Json(CreateClaimResponse {
        claim_id: claim_id.to_string(),
        status: "pending".to_string(),
    }))
}

async fn confirm_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ConfirmClaimRequest>,
) -> Result<Json<ConfirmClaimResponse>, AppError> {
    let user_id = extract_user_id(&headers, &state.config.jwt_secret).await?;
    
    // Проверяем, что claim принадлежит пользователю
    let claim = sqlx::query!(
        r#"
        SELECT id, status FROM claims
        WHERE id = $1 AND user_id = $2
        "#,
        payload.claim_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;
    
    let claim = claim.ok_or_else(|| AppError::NotFound("Claim not found".to_string()))?;
    
    if claim.status.as_deref() != Some("pending") {
        return Err(AppError::Validation("Claim is not in pending status".to_string()));
    }
    
    // Обновляем статус на completed (имитация on-chain транзакции)
    sqlx::query!(
        r#"
        UPDATE claims
        SET status = 'completed'
        WHERE id = $1
        "#,
        payload.claim_id
    )
    .execute(&state.pool)
    .await?;
    
    Ok(Json(ConfirmClaimResponse {
        success: true,
        status: "completed".to_string(),
    }))
}

pub fn router() -> Router<crate::app_state::AppState> {
    Router::new()
        .route("/start", post(create_claim))
        .route("/confirm", post(confirm_claim))
}
