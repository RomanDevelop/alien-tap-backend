// Отладка: проверяем формат user объекта как его видит сервер

use serde_json::json;

fn main() {
    let user_obj = json!({
        "id": 123456789,
        "username": "player",
        "first_name": "John"
    });
    
    // Сервер делает value.to_string() для объекта
    let as_string = user_obj.to_string();
    
    println!("Формат через to_string(): {}", as_string);
    println!("\nФормат через serde_json::to_string(): {}", serde_json::to_string(&user_obj).unwrap());
    
    // Проверяем, одинаковы ли они
    let serde_format = serde_json::to_string(&user_obj).unwrap();
    println!("\nОдинаковы? {}", as_string == serde_format);
}

