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
use serde::Deserialize;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TelegramAuthRequest {
    #[serde(rename = "initData")]
    pub init_data: Option<String>,  // Оригинальная строка initData (приоритет)
    pub hash: Option<String>,       // Для обратной совместимости
    #[serde(rename = "auth_date")]
    pub auth_date: Option<String>,   // Для обратной совместимости
    pub user: Option<serde_json::Value>, // Для обратной совместимости
}

async fn authenticate_telegram(
    State(state): State<AppState>,
    Json(payload): Json<TelegramAuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::info!("📥 Получен запрос авторизации:");
    tracing::info!("   initData присутствует: {}", payload.init_data.is_some());
    tracing::info!("   hash присутствует: {}", payload.hash.is_some());
    tracing::info!("   auth_date присутствует: {}", payload.auth_date.is_some());
    tracing::info!("   user присутствует: {}", payload.user.is_some());
    
    if let Some(ref init_data) = payload.init_data {
        tracing::info!("   initData (первые 100 символов): {}", 
            if init_data.len() > 100 { 
                format!("{}...", &init_data[..100]) 
            } else { 
                init_data.clone() 
            });
        // Логируем полный initData для отладки (только в DEBUG режиме)
        tracing::debug!("   initData (полный): {}", init_data);
    }
    
    if let Some(ref hash) = payload.hash {
        tracing::info!("   hash: {}", hash);
    }
    
    if let Some(ref auth_date) = payload.auth_date {
        tracing::info!("   auth_date: {}", auth_date);
    }
    
    if let Some(ref user) = payload.user {
        tracing::info!("   user: {}", serde_json::to_string(user).unwrap_or_else(|_| "не удалось сериализовать".to_string()));
    }
    
    // Приоритет: используем initData строку, если она есть
    let user_data_obj = if let Some(ref init_data) = payload.init_data {
        tracing::info!("✅ Используем оригинальную строку initData для проверки подписи (правильный формат)");
        tracing::info!("   Длина initData: {} символов", init_data.len());
        
        // Проверяем подпись используя оригинальную строку initData
        telegram::verify_telegram_auth_from_init_data(
            init_data,
            &state.config.telegram_bot_token,
            state.config.dev_mode,
        )
        .map_err(|e| {
            tracing::error!("❌ Неверная подпись Telegram: {}", e);
            AppError::Authentication(format!("Invalid telegram signature: {}", e))
        })?;
        
        tracing::info!("✅ Подпись Telegram проверена успешно (через оригинальную строку initData)");
        
        // Извлекаем user из initData
        let user_value = telegram::parse_user_from_init_data(init_data)
            .map_err(|e| AppError::Validation(format!("Failed to parse user from initData: {}", e)))?;
        
        // Клонируем объект, чтобы он жил достаточно долго
        user_value.as_object()
            .ok_or_else(|| AppError::Validation("User data is not an object".to_string()))?
            .clone()
    } else if let (Some(hash), Some(auth_date), Some(user)) = 
        (&payload.hash, &payload.auth_date, &payload.user) 
    {
        // Обратная совместимость: используем старый метод
        tracing::warn!("⚠️ Используется старый формат запроса (без initData)");
        
        // Парсим данные из JSON для старого формата
        let mut data: BTreeMap<String, String> = BTreeMap::new();
        data.insert("auth_date".to_string(), auth_date.clone());
        
        // КРИТИЧНО: Пробуем разные варианты порядка ключей в объекте user
        // Telegram может использовать разный порядок в зависимости от версии
        // Пробуем несколько вариантов, пока не найдем правильный
        let user_obj = user.as_object()
            .ok_or_else(|| AppError::Validation("User data is not an object".to_string()))?;
        
        // Функция для формирования JSON с заданным порядком ключей
        // Используем serde_json для правильного форматирования, но контролируем порядок ключей
        let build_user_json = |ordered_keys: &[&str]| -> String {
            use serde_json::Map;
            let mut ordered_map = Map::new();
            
            // Добавляем ключи в заданном порядке
            for key in ordered_keys {
                if let Some(val) = user_obj.get(*key) {
                    ordered_map.insert(key.to_string(), val.clone());
                }
            }
            
            // Добавляем остальные ключи, если есть
            for (key, val) in user_obj {
                if !ordered_keys.contains(&key.as_str()) {
                    ordered_map.insert(key.clone(), val.clone());
                }
            }
            
            // Используем serde_json::to_string для правильного форматирования
            // Это гарантирует правильное экранирование и форматирование JSON
            serde_json::to_string(&serde_json::Value::Object(ordered_map))
                .unwrap_or_else(|_| "{}".to_string())
        };
        
        // Варианты порядка ключей для проверки
        // Пробуем все возможные комбинации, так как Telegram может использовать любой порядок
        let key_orders = vec![
            vec!["id", "first_name", "last_name", "username"], // Вариант 1
            vec!["id", "first_name", "username", "last_name"], // Вариант 2
            vec!["id", "username", "first_name", "last_name"], // Вариант 3
            vec!["id", "username", "last_name", "first_name"], // Вариант 4
            vec!["id", "last_name", "first_name", "username"], // Вариант 5
            vec!["id", "last_name", "username", "first_name"], // Вариант 6
            vec!["first_name", "id", "last_name", "username"], // Вариант 7 (как приходит от Flutter)
            vec!["first_name", "id", "username", "last_name"], // Вариант 8
            vec!["first_name", "last_name", "id", "username"], // Вариант 9
            vec!["first_name", "last_name", "username", "id"], // Вариант 10
            vec!["first_name", "username", "id", "last_name"], // Вариант 11
            vec!["first_name", "username", "last_name", "id"], // Вариант 12
            vec!["username", "id", "first_name", "last_name"], // Вариант 13
            vec!["username", "id", "last_name", "first_name"], // Вариант 14
            vec!["username", "first_name", "id", "last_name"], // Вариант 15
            vec!["username", "first_name", "last_name", "id"], // Вариант 16
            vec!["username", "last_name", "id", "first_name"], // Вариант 17
            vec!["username", "last_name", "first_name", "id"], // Вариант 18
            vec!["last_name", "id", "first_name", "username"], // Вариант 19
            vec!["last_name", "id", "username", "first_name"], // Вариант 20
            vec!["last_name", "first_name", "id", "username"], // Вариант 21
            vec!["last_name", "first_name", "username", "id"], // Вариант 22
            vec!["last_name", "username", "id", "first_name"], // Вариант 23
            vec!["last_name", "username", "first_name", "id"], // Вариант 24
        ];
        
        let mut signature_valid = false;
        let mut last_user_json = String::new();
        
        // Пробуем каждый вариант порядка ключей
        // Ограничиваем количество попыток для производительности - пробуем только первые 10 вариантов
        // Если не сработает, значит проблема не в порядке ключей
        for (idx, order) in key_orders.iter().take(10).enumerate() {
            let user_json = build_user_json(order);
            last_user_json = user_json.clone();
            
            let mut test_data: BTreeMap<String, String> = BTreeMap::new();
            test_data.insert("auth_date".to_string(), auth_date.clone());
            test_data.insert("user".to_string(), user_json.clone());
            
            if idx < 5 {
                // Логируем только первые 5 попыток, чтобы не засорять логи
                tracing::debug!("📋 Попытка {}: User JSON с порядком {:?}: {}", idx + 1, order, user_json);
            }
            
            if telegram::verify_telegram_auth(&test_data, hash, &state.config.telegram_bot_token, state.config.dev_mode) {
                tracing::info!("✅ Подпись проверена успешно с порядком ключей: {:?}", order);
                signature_valid = true;
                data.insert("user".to_string(), user_json);
                break;
            }
        }
        
        // Если первые 10 вариантов не сработали, пробуем остальные
        if !signature_valid {
            tracing::warn!("⚠️ Первые 10 вариантов не сработали, пробуем остальные...");
            for (idx, order) in key_orders.iter().skip(10).enumerate() {
                let user_json = build_user_json(order);
                last_user_json = user_json.clone();
                
                let mut test_data: BTreeMap<String, String> = BTreeMap::new();
                test_data.insert("auth_date".to_string(), auth_date.clone());
                test_data.insert("user".to_string(), user_json.clone());
                
                if telegram::verify_telegram_auth(&test_data, hash, &state.config.telegram_bot_token, state.config.dev_mode) {
                    tracing::info!("✅ Подпись проверена успешно с порядком ключей: {:?} (попытка {})", order, idx + 11);
                    signature_valid = true;
                    data.insert("user".to_string(), user_json);
                    break;
                }
            }
        }
        
        if !signature_valid {
            // В dev-режиме разрешаем пропустить проверку подписи для разработки
            // ВАЖНО: Это только для разработки! В продакшене это недопустимо!
            if state.config.dev_mode {
                tracing::warn!("⚠️ DEV MODE: Пропускаем проверку подписи Telegram для разработки");
                tracing::warn!("   Попробовано {} вариантов порядка ключей, ни один не сработал", key_orders.len());
                tracing::warn!("   ВАЖНО: Исправьте Flutter, чтобы он отправлял оригинальную строку initData!");
                tracing::warn!("   Без initData невозможно правильно проверить подпись Telegram");
                tracing::warn!("   Авторизация разрешена только в dev-режиме!");
                // Используем последний попробованный вариант для dev-режима
                data.insert("user".to_string(), last_user_json);
            } else {
                tracing::error!("❌ Авторизация не прошла: неверная подпись Telegram");
                tracing::error!("   Попробовано {} вариантов порядка ключей", key_orders.len());
                tracing::error!("   Последний попробованный User JSON: {}", last_user_json);
                tracing::error!("   ВАЖНО: Flutter должен отправлять оригинальную строку initData для правильной проверки подписи!");
                return Err(AppError::Authentication("Invalid telegram signature".to_string()));
            }
        } else {
            tracing::info!("✅ Подпись Telegram проверена успешно (старый формат)");
        }
        
        // Клонируем объект
        user.as_object()
            .ok_or_else(|| AppError::Validation("User data is not an object".to_string()))?
            .clone()
    } else {
        return Err(AppError::Validation(
            "Either initData or (hash, auth_date, user) must be provided".to_string()
        ));
    };
    
    let user_data = &user_data_obj;
    
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
    
    tracing::info!("✅ JWT токен выдан пользователю: user_id={}, telegram_id={}, username={:?}", 
        user.id, user.telegram_id, user.username);
    
    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
    }))
}


pub fn router() -> Router<crate::app_state::AppState> {
    Router::new().route("/telegram", post(authenticate_telegram))
}
