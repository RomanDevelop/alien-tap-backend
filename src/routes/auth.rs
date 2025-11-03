use axum::{
    extract::State,
    response::Json,
    routing::post,
    Router,
};
use serde::Serialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::models::user::TelegramUser;
use crate::utils::telegram;
use crate::utils::jwt;
use crate::utils::errors::AppError;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
}

async fn authenticate_telegram(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<AuthResponse>, AppError> {
    // Парсим данные из JSON
    let mut data: BTreeMap<String, String> = BTreeMap::new();
    let hash = payload["hash"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing hash field".to_string()))?
        .to_string();
    
    // Извлекаем все поля кроме hash для проверки подписи
    for (key, value) in payload.as_object().unwrap() {
        if key != "hash" {
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => value.to_string(),
            };
            data.insert(key.clone(), val_str);
        }
    }
    
    // Проверяем подпись
    if !telegram::verify_telegram_auth(&data, &hash, &state.config.telegram_bot_token) {
        return Err(AppError::Authentication("Invalid telegram signature".to_string()));
    }
    
    // Извлекаем пользователя
    let user_data = payload["user"].as_object()
        .ok_or_else(|| AppError::Validation("Missing user field".to_string()))?;
    
    let telegram_user = TelegramUser {
        id: user_data.get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AppError::Validation("Invalid user id".to_string()))?,
        username: user_data.get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        first_name: user_data.get("first_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        last_name: user_data.get("last_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };
    
    // Ищем или создаём пользователя
    let user_id = Uuid::new_v4();
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, telegram_id, username, first_name, last_name)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (telegram_id) 
        DO UPDATE SET 
            username = EXCLUDED.username,
            first_name = EXCLUDED.first_name,
            last_name = EXCLUDED.last_name
        RETURNING id, telegram_id, username, first_name, last_name, created_at
        "#,
        user_id,
        telegram_user.id,
        telegram_user.username.as_deref(),
        telegram_user.first_name.as_deref(),
        telegram_user.last_name.as_deref()
    )
    .fetch_one(&state.pool)
    .await?;
    
    let user = crate::models::user::User {
        id: row.id,
        telegram_id: row.telegram_id,
        username: row.username,
        first_name: row.first_name,
        last_name: row.last_name,
        created_at: row.created_at
            .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc))
            .unwrap_or_else(|| chrono::Utc::now()),
    };
    
    // Создаём или обновляем счёт
    sqlx::query!(
        r#"
        INSERT INTO scores (id, user_id, score)
        VALUES ($1, $2, 0)
        ON CONFLICT (user_id) DO NOTHING
        "#,
        Uuid::new_v4(),
        user.id
    )
    .execute(&state.pool)
    .await?;
    
    // Создаём JWT токен
    let token = jwt::create_jwt(&user.id.to_string(), &state.config.jwt_secret)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT error: {}", e)))?;
    
    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
    }))
}

pub fn router() -> Router<crate::app_state::AppState> {
    Router::new().route("/telegram", post(authenticate_telegram))
}
