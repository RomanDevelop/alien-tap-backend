// Утилита для генерации валидной подписи Telegram для тестирования
// Запуск: cargo run --bin generate_hash

use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use hex;
use std::collections::BTreeMap;
use serde_json::json;

fn generate_telegram_hash(data: &BTreeMap<String, String>, bot_token: &str) -> String {
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
    
    // Вычисляем секретный ключ
    let secret_key = Sha256::digest(bot_token.as_bytes());
    
    // Создаем HMAC
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key)
        .expect("HMAC can take key of any size");
    mac.update(check_string.as_bytes());
    
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

fn main() {
    // Получаем BOT_TOKEN из env или используем из .env
    dotenvy::dotenv().ok();
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN должен быть в .env");
    
    // Тестовые данные
    // ВАЖНО: user должен быть сериализован в том же формате, как его обрабатывает сервер
    // Сервер использует value.to_string() для объектов, что даёт JSON без пробелов
    let user_json = serde_json::to_string(&json!({
        "id": 123456789,
        "username": "player",
        "first_name": "John"
    })).unwrap();
    
    let mut data = BTreeMap::new();
    data.insert("auth_date".to_string(), "1234567890".to_string());
    data.insert("user".to_string(), user_json);
    
    // Генерируем подпись
    let hash = generate_telegram_hash(&data, &bot_token);
    
    // Выводим готовый JSON для тестирования
    println!("\n✅ Готовый JSON для Thunder Client:\n");
    let json = json!({
        "auth_date": "1234567890",
        "user": {
            "id": 123456789,
            "username": "player",
            "first_name": "John"
        },
        "hash": hash
    });
    
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    
    println!("\n📝 Или в одну строку (для копирования):\n");
    println!("{}", serde_json::to_string(&json).unwrap());
}

