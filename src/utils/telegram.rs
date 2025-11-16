use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use base64::{Engine as _, engine::general_purpose};
use std::collections::BTreeMap;
use percent_encoding::percent_decode_str;

pub fn verify_telegram_auth(data: &BTreeMap<String, String>, hash: &str, bot_token: &str, dev_mode: bool) -> bool {
    // В dev-режиме пропускаем мок-хэши для локальной разработки
    if dev_mode && hash.starts_with("mock_hash_for_development_") {
        tracing::info!("🔧 Dev mode: Allowing mock hash for development");
        return true;
    }
    
    tracing::debug!("🔍 Проверка подписи Telegram:");
    tracing::debug!("   Полученный hash: {}", hash);
    tracing::debug!("   Данные для проверки: {:?}", data);
    
    let mut check_string = String::new();
    
    // Сортируем ключи в алфавитном порядке
    for (key, value) in data {
        if key != "hash" {
            check_string.push_str(&format!("{}={}\n", key, value));
        }
    }
    
    // Удаляем последний символ новой строки
    if check_string.ends_with('\n') {
        check_string.pop();
    }
    
    tracing::debug!("   Сформированный check_string: {:?}", check_string);
    tracing::debug!("   check_string (bytes): {:?}", check_string.as_bytes());
    
    // Вычисляем секретный ключ согласно документации Telegram:
    // secret_key = HMAC-SHA256(key="WebAppData", message=bot_token)
    let mut mac_key = Hmac::<Sha256>::new_from_slice(b"WebAppData")
        .expect("HMAC can take key of any size");
    mac_key.update(bot_token.as_bytes());
    let secret_key = mac_key.finalize().into_bytes();
    
    tracing::debug!("   Секретный ключ (первые 16 байт): {:?}", &secret_key[..16.min(secret_key.len())]);
    
    // Создаем HMAC для проверки подписи
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key)
        .expect("HMAC can take key of any size");
    mac.update(check_string.as_bytes());
    
    let result = mac.finalize().into_bytes();
    let expected_hash = hex::encode(result);
    
    tracing::debug!("   Вычисленный hash: {}", expected_hash);
    tracing::debug!("   Hash совпадает: {}", expected_hash == hash);
    
    if expected_hash != hash {
        tracing::warn!("❌ Неверная подпись Telegram!");
        tracing::warn!("   Ожидалось: {}", expected_hash);
        tracing::warn!("   Получено:  {}", hash);
    }
    
    expected_hash == hash
}

/// Проверяет подпись Telegram используя оригинальную строку initData
/// Это правильный способ проверки, так как сохраняет точный порядок полей
pub fn verify_telegram_auth_from_init_data(
    init_data: &str,
    bot_token: &str,
    dev_mode: bool,
) -> Result<bool, String> {
    tracing::debug!("🔍 Проверка подписи Telegram из initData:");
    tracing::debug!("   initData длина: {}", init_data.len());
    
    // Парсим initData строку
    // Telegram может использовать либо "hash", либо "signature" в зависимости от версии API
    // ВАЖНО: Приоритет у "hash", если есть оба параметра
    let mut all_params: Vec<(&str, &str)> = Vec::new();
    let mut hash_value: Option<&str> = None;
    let mut signature_value: Option<&str> = None;
    
    // Сначала проходим по всем параметрам, чтобы найти hash и signature
    for pair in init_data.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "hash" {
                hash_value = Some(value);
            } else if key == "signature" {
                signature_value = Some(value);
            }
            // Сохраняем все параметры (включая hash и signature) для дальнейшей обработки
            all_params.push((key, value));
        }
    }
    
    // Telegram может использовать либо hash, либо signature
    // ВАЖНО: Используем hash для проверки (HMAC-SHA256), так как signature требует Ed25519
    // hash - это HMAC-SHA256, который мы уже реализовали правильно
    // signature - это Ed25519, который требует публичный ключ Telegram (более сложная реализация)
    // Если есть hash, используем его. Если нет hash, но есть signature, используем signature (но это не реализовано)
    let received_hash = hash_value
        .ok_or_else(|| {
            if signature_value.is_some() {
                "Signature verification (Ed25519) not yet implemented. Please use hash parameter.".to_string()
            } else {
                "Hash or signature not found in initData".to_string()
            }
        })?;
    
    // Логируем, какой hash используется
    tracing::debug!("   Используем hash (HMAC-SHA256, официальный формат)");
    if signature_value.is_some() {
        tracing::debug!("   ВАЖНО: signature присутствует и будет включен в check_string (Bot API 8.0+)");
    }
    
    // ВАЖНО: После обновления Bot API 8.0 (ноябрь 2024), signature ДОЛЖЕН быть включен в check_string
    // Формат: auth_date=<auth_date>\nquery_id=<query_id>\nsignature=<signature>\nuser=<user>
    // Исключаем только hash из check_string
    // query_id и signature ДОЛЖНЫ быть включены в check_string согласно Bot API 8.0+
    let mut params: Vec<(&str, &str)> = Vec::new();
    for (key, value) in &all_params {
        if *key != "hash" {
            // Включаем все параметры кроме hash (включая query_id и signature)
            params.push((*key, *value));
        }
    }
    
    // В dev-режиме пропускаем мок-хэши
    if dev_mode && received_hash.starts_with("mock_hash_for_development_") {
        tracing::info!("🔧 Dev mode: Allowing mock hash for development");
        return Ok(true);
    }
    
    tracing::debug!("   Полученный hash: {}", received_hash);
    tracing::debug!("   Параметров (кроме hash, но включая query_id и signature): {}", params.len());
    tracing::debug!("   Параметры до сортировки: {:?}", params.iter().map(|(k, _)| *k).collect::<Vec<_>>());
    
    // Сортируем параметры по ключам (алфавитный порядок) - это требование Telegram
    // ВАЖНО: значения остаются оригинальными из initData, сохраняя порядок внутри (например, в объекте user)
    params.sort_by(|a, b| a.0.cmp(b.0));
    
    tracing::debug!("   Параметры после сортировки: {:?}", params.iter().map(|(k, _)| *k).collect::<Vec<_>>());
    
    // Формируем check_string из оригинальной строки (без параметра hash)
    // ВАЖНО: Согласно официальной документации Telegram, значения нужно декодировать из URL-encoding
    // перед включением в check_string. В примере документации user показан как JSON объект, а не URL-encoded
    // Пример: user=%7B%22id%22%3A279058397%7D -> user={"id":279058397}
    let mut check_string_parts = Vec::new();
    for (key, value) in &params {
        // Декодируем значение из URL-encoding перед включением в check_string
        let decoded_value = percent_decode_str(value)
            .decode_utf8()
            .map_err(|e| format!("Failed to decode value for key {}: {}", key, e))?;
        tracing::debug!("   Параметр {}: оригинальное значение: {:?}, декодированное: {:?}", key, value, decoded_value);
        check_string_parts.push(format!("{}={}", key, decoded_value));
    }
    let check_string = check_string_parts.join("\n");
    
    tracing::debug!("   Сформированный check_string: {:?}", check_string);
    tracing::debug!("   check_string (bytes): {:?}", check_string.as_bytes());
    
    // Вычисляем секретный ключ согласно документации Telegram:
    // secret_key = HMAC-SHA256(key="WebAppData", message=bot_token)
    let mut mac_key = Hmac::<Sha256>::new_from_slice(b"WebAppData")
        .map_err(|e| format!("Failed to create HMAC key: {}", e))?;
    mac_key.update(bot_token.as_bytes());
    let secret_key = mac_key.finalize().into_bytes();
    
    tracing::debug!("   Секретный ключ (первые 16 байт): {:?}", &secret_key[..16.min(secret_key.len())]);
    
    // Создаем HMAC для проверки подписи
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(check_string.as_bytes());
    
    let result = mac.finalize().into_bytes();
    let computed_hash_hex = hex::encode(result);
    
    // Telegram может отправлять hash в формате hex, base64 или base64url
    // Пробуем все варианты
    let computed_hash_base64 = general_purpose::STANDARD.encode(result);
    let computed_hash_base64url = general_purpose::URL_SAFE_NO_PAD.encode(result);
    
    tracing::debug!("   Вычисленный hash (hex): {}", computed_hash_hex);
    tracing::debug!("   Вычисленный hash (base64): {}", computed_hash_base64);
    tracing::debug!("   Вычисленный hash (base64url): {}", computed_hash_base64url);
    tracing::debug!("   Полученный hash: {}", received_hash);
    
    // Пробуем декодировать полученный hash
    // ВАЖНО: hash обычно в hex формате, signature - в base64url
    // Проверяем в правильном порядке: сначала hex, потом base64url, потом base64
    let received_hash_bytes = if let Ok(decoded) = hex::decode(received_hash) {
        tracing::debug!("   Полученный hash (декодирован из hex, длина {} байт)", decoded.len());
        Some(decoded)
    } else if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(received_hash) {
        let decoded_hex = hex::encode(&decoded);
        tracing::debug!("   Полученный hash (декодирован из base64url, длина {} байт)", decoded.len());
        tracing::debug!("   Полученный hash (декодирован в hex): {}", decoded_hex);
        Some(decoded)
    } else if let Ok(decoded) = general_purpose::STANDARD.decode(received_hash) {
        let decoded_hex = hex::encode(&decoded);
        tracing::debug!("   Полученный hash (декодирован из base64, длина {} байт)", decoded.len());
        tracing::debug!("   Полученный hash (декодирован в hex): {}", decoded_hex);
        Some(decoded)
    } else {
        None
    };
    
    // Преобразуем result в Vec<u8> для сравнения
    let result_bytes: Vec<u8> = result.to_vec();
    
    // Проверяем все форматы (прямое сравнение строк и байтов)
    let hash_matches = computed_hash_hex == received_hash 
        || computed_hash_base64 == received_hash 
        || computed_hash_base64url == received_hash
        || received_hash_bytes.map(|h| h == result_bytes).unwrap_or(false);
    
    tracing::debug!("   Hash совпадает: {}", hash_matches);
    
    if !hash_matches {
        tracing::warn!("❌ Неверная подпись Telegram!");
        tracing::warn!("   Ожидалось (hex): {}", computed_hash_hex);
        tracing::warn!("   Ожидалось (base64): {}", computed_hash_base64);
        tracing::warn!("   Ожидалось (base64url): {}", computed_hash_base64url);
        tracing::warn!("   Получено:  {}", received_hash);
        return Err(format!(
            "Invalid telegram signature. Expected (hex): {}, Expected (base64): {}, Expected (base64url): {}, Got: {}",
            computed_hash_hex, computed_hash_base64, computed_hash_base64url, received_hash
        ));
    }
    
    Ok(true)
}

/// Парсит объект user из initData строки
pub fn parse_user_from_init_data(init_data: &str) -> Result<serde_json::Value, String> {
    for pair in init_data.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "user" {
                // Декодируем URL-encoded значение
                let decoded = percent_decode_str(value)
                    .decode_utf8()
                    .map_err(|e| format!("Failed to decode user param: {}", e))?;
                
                // Парсим JSON
                let user: serde_json::Value = serde_json::from_str(&decoded)
                    .map_err(|e| format!("Failed to parse user JSON: {}", e))?;
                
                tracing::debug!("📋 Извлечен user из initData: {:?}", user);
                return Ok(user);
            }
        }
    }
    
    Err("User parameter not found in initData".to_string())
}
