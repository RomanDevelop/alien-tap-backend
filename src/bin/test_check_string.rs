// Тест: что именно попадает в check_string

use serde_json::json;
use std::collections::BTreeMap;

fn main() {
    // Имитируем что делает сервер
    let payload = json!({
        "auth_date": "1234567890",
        "user": {
            "id": 123456789,
            "username": "player",
            "first_name": "John"
        },
        "hash": "test_hash"
    });
    
    let mut data: BTreeMap<String, String> = BTreeMap::new();
    
    // Точно как в сервере
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
    
    // Формируем check_string как в telegram.rs
    let mut check_string = String::new();
    for (key, value) in &data {
        check_string.push_str(&format!("{}={}\n", key, value));
    }
    if check_string.ends_with('\n') {
        check_string.pop();
    }
    
    println!("Данные в BTreeMap:");
    for (k, v) in &data {
        println!("  {} = {}", k, v);
    }
    
    println!("\ncheck_string:");
    println!("{}", check_string);
    println!("\ncheck_string (escaped):");
    println!("{:?}", check_string);
}

