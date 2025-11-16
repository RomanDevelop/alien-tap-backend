#!/usr/bin/env python3
"""
Тестовый скрипт для проверки логики верификации Telegram initData
Используем данные из реальных логов для отладки
"""

import hmac
import hashlib
from urllib.parse import unquote

# Данные из лога (строка 34)
init_data = "query_id=AAEesK1TAAAAAB6wrVNvlqv9&user=%7B%22id%22%3A1403891742%2C%22first_name%22%3A%22rom%22%2C%22last_name%22%3A%22%22%2C%22username%22%3A%22romanklia%22%2C%22language_code%22%3A%22ru%22%2C%22allows_write_to_pm%22%3Atrue%2C%22photo_url%22%3A%22https%3A%5C%2F%5C%2Ft.me%5C%2Fi%5C%2Fuserpic%5C%2F320%5C%2Fk2zz98D_4NYVFEtpv-BXLucFat-IyvKctvl6lGT8IPc.svg%22%7D&auth_date=1763294634&signature=NQ8YFyLXTAEv5PUIBseGojIHTVztUZeLaSAOsX8d5Lg12HX71bFdRD5CeD9_3N3Fw99xuJ0INjkg0u3WmdnrAQ&hash=99a5b788a784fba5358604ced7af5f73397bd0b2eff1e67107f120ce66bd08e6"

# Бот токен (нужно получить из конфига)
# ВАЖНО: Замените на реальный токен!
BOT_TOKEN = "YOUR_BOT_TOKEN_HERE"

def verify_telegram_init_data(init_data: str, bot_token: str):
    """Проверка initData согласно документации Telegram"""
    
    # Парсим параметры
    params = {}
    hash_value = None
    signature_value = None
    
    for pair in init_data.split('&'):
        if '=' in pair:
            key, value = pair.split('=', 1)
            if key == 'hash':
                hash_value = value
            elif key == 'signature':
                signature_value = value
            else:
                params[key] = value
    
    print(f"Найден hash: {hash_value}")
    print(f"Найден signature: {signature_value}")
    print(f"Параметры (кроме hash и signature): {list(params.keys())}")
    
    # Формируем check_string
    # Вариант 1: С декодированием значений (как в документации)
    check_string_parts = []
    for key in sorted(params.keys()):
        value = params[key]
        # Декодируем значение
        decoded_value = unquote(value)
        check_string_parts.append(f"{key}={decoded_value}")
    
    check_string = "\n".join(check_string_parts)
    print(f"\ncheck_string (с декодированием):\n{check_string}")
    
    # Вариант 2: БЕЗ декодирования значений
    check_string_parts_no_decode = []
    for key in sorted(params.keys()):
        value = params[key]
        # НЕ декодируем значение
        check_string_parts_no_decode.append(f"{key}={value}")
    
    check_string_no_decode = "\n".join(check_string_parts_no_decode)
    print(f"\ncheck_string (БЕЗ декодирования):\n{check_string_no_decode}")
    
    # Вычисляем secret_key
    secret_key = hmac.new(
        b"WebAppData",
        bot_token.encode('utf-8'),
        hashlib.sha256
    ).digest()
    
    print(f"\nsecret_key (первые 16 байт): {secret_key[:16].hex()}")
    
    # Вычисляем hash для варианта 1 (с декодированием)
    computed_hash_1 = hmac.new(
        secret_key,
        check_string.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()
    
    print(f"\nВычисленный hash (с декодированием): {computed_hash_1}")
    print(f"Полученный hash: {hash_value}")
    print(f"Совпадает (с декодированием): {computed_hash_1 == hash_value}")
    
    # Вычисляем hash для варианта 2 (БЕЗ декодирования)
    computed_hash_2 = hmac.new(
        secret_key,
        check_string_no_decode.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()
    
    print(f"\nВычисленный hash (БЕЗ декодирования): {computed_hash_2}")
    print(f"Совпадает (БЕЗ декодирования): {computed_hash_2 == hash_value}")
    
    # Вариант 3: Без query_id
    params_no_query_id = {k: v for k, v in params.items() if k != 'query_id'}
    check_string_parts_no_query = []
    for key in sorted(params_no_query_id.keys()):
        value = params_no_query_id[key]
        decoded_value = unquote(value)
        check_string_parts_no_query.append(f"{key}={decoded_value}")
    
    check_string_no_query = "\n".join(check_string_parts_no_query)
    computed_hash_3 = hmac.new(
        secret_key,
        check_string_no_query.encode('utf-8'),
        hashlib.sha256
    ).hexdigest()
    
    print(f"\nВычисленный hash (без query_id, с декодированием): {computed_hash_3}")
    print(f"Совпадает (без query_id): {computed_hash_3 == hash_value}")

if __name__ == "__main__":
    print("=" * 80)
    print("Тест верификации Telegram initData")
    print("=" * 80)
    print(f"\ninitData: {init_data[:100]}...")
    print("\nВАЖНО: Замените BOT_TOKEN на реальный токен бота!")
    print("\n" + "=" * 80)
    
    # Запускаем тест (но без реального токена он не сработает)
    # verify_telegram_init_data(init_data, BOT_TOKEN)

