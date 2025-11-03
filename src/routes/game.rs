use axum::{
    extract::State,
    http::HeaderMap,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::models::score::{UpdateScoreRequest, LeaderboardEntry};
use crate::utils::errors::AppError;
use crate::utils::jwt;

#[derive(Debug, Serialize)]
pub struct UpdateScoreResponse {
    pub success: bool,
    pub score: i32,
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

async fn update_score(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateScoreRequest>,
) -> Result<Json<UpdateScoreResponse>, AppError> {
    let user_id = extract_user_id(&headers, &state.config.jwt_secret).await?;
    
    // Обновляем счёт пользователя
    let score = sqlx::query!(
        r#"
        INSERT INTO scores (id, user_id, score)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) 
        DO UPDATE SET 
            score = GREATEST(scores.score, $3),
            updated_at = now()
        RETURNING score
        "#,
        Uuid::new_v4(),
        user_id,
        payload.score
    )
    .fetch_one(&state.pool)
    .await?;
    
    Ok(Json(UpdateScoreResponse {
        success: true,
        score: score.score.unwrap_or(0),
    }))
}

async fn leaderboard(
    State(state): State<AppState>,
) -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            u.id as user_id,
            u.username,
            u.first_name,
            s.score
        FROM scores s
        JOIN users u ON s.user_id = u.id
        ORDER BY s.score DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.pool)
    .await?;
    
    let entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .map(|row| LeaderboardEntry {
            user_id: row.user_id,
            username: row.username,
            first_name: row.first_name,
            score: row.score.unwrap_or(0),
        })
        .collect();
    
    Ok(Json(entries))
}

pub fn router() -> Router<crate::app_state::AppState> {
    Router::new()
        .route("/update_score", post(update_score))
        .route("/leaderboard", get(leaderboard))
}
