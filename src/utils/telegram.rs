use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use hex;
use std::collections::BTreeMap;

pub fn verify_telegram_auth(data: &BTreeMap<String, String>, hash: &str, bot_token: &str) -> bool {
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
    let expected_hash = hex::encode(result);
    
    expected_hash == hash
}
