// Генератор валидной подписи Telegram для тестирования
// Запустите в Node.js: node generate_hash.js

const crypto = require('crypto');

// Ваш BOT_TOKEN из .env
const BOT_TOKEN = "8265053392:AAE_D8DD1N9nR-KJ4Nq1mDEV3z5WN-qp6gk";

// Данные для теста
const testData = {
    "auth_date": "1234567890",
    "user": JSON.stringify({
        "id": 123456789,
        "username": "player",
        "first_name": "John"
    })
};

// Функция генерации подписи (как в Rust коде)
function generateTelegramHash(data, botToken) {
    // Сортируем ключи по алфавиту
    const sortedKeys = Object.keys(data).sort();
    
    // Формируем check_string (ключ=значение\n...)
    const checkString = sortedKeys
        .filter(key => key !== "hash")
        .map(key => `${key}=${data[key]}`)
        .join('\n');
    
    // Вычисляем секретный ключ (SHA256 от bot_token)
    const secretKey = crypto.createHash('sha256')
        .update(botToken)
        .digest();
    
    // Создаём HMAC-SHA256
    const hash = crypto.createHmac('sha256', secretKey)
        .update(checkString)
        .digest('hex');
    
    return hash;
}

const hash = generateTelegramHash(testData, BOT_TOKEN);

console.log('\n✅ Готовый JSON для Thunder Client:\n');
console.log(JSON.stringify({
    "auth_date": testData.auth_date,
    "user": JSON.parse(testData.user),
    "hash": hash
}, null, 2));

console.log('\n📝 Или в одну строку:\n');
console.log(JSON.stringify({
    "auth_date": testData.auth_date,
    "user": JSON.parse(testData.user),
    "hash": hash
}));

