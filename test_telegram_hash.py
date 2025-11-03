#!/usr/bin/env python3
"""
Скрипт для генерации валидной подписи Telegram WebApp для тестирования
"""
import hmac
import hashlib
from urllib.parse import urlencode

# Ваш BOT_TOKEN из .env
BOT_TOKEN = "8265053392:AAE_D8DD1N9nR-KJ4Nq1mDEV3z5WN-qp6gk"

# Данные пользователя
user_data = {
    "id": 123456789,
    "first_name": "John",
    "username": "player"
}

# Параметры для проверки подписи
auth_data = {
    "auth_date": "1234567890",
    "user": user_data
}

def generate_telegram_hash(data: dict, bot_token: str) -> str:
    """Генерирует подпись для Telegram WebApp"""
    # Сортируем ключи
    sorted_keys = sorted(data.keys())
    
    # Формируем check_string
    check_string_parts = []
    for key in sorted_keys:
        if key != "hash":
            value = data[key]
            if isinstance(value, dict):
                # Для user объекта используем JSON
                import json
                value = json.dumps(value, separators=(',', ':'))
            check_string_parts.append(f"{key}={value}")
    
    check_string = "\n".join(check_string_parts)
    
    # Вычисляем секретный ключ
    secret_key = hashlib.sha256(bot_token.encode()).digest()
    
    # Создаём HMAC
    hmac_obj = hmac.new(secret_key, check_string.encode(), hashlib.sha256)
    hash_value = hmac_obj.hexdigest()
    
    return hash_value

# Создаём данные с user объектом
test_data = {
    "auth_date": "1234567890",
    "user": '{"id":123456789,"first_name":"John","username":"player"}'
}

hash_result = generate_telegram_hash(test_data, BOT_TOKEN)
test_data["hash"] = hash_result

print("JSON для Thunder Client:")
print("{")
print(f'  "auth_date": "{test_data["auth_date"]}",')
print(f'  "user": {test_data["user"]},')
print(f'  "hash": "{hash_result}"')
print("}")

