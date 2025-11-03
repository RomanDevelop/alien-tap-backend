# Alien Tap Backend

Полноценный backend на **Rust (Axum)** с базой данных **PostgreSQL** и интеграцией **Telegram WebApp SDK** для игрового приложения.

## 🚀 Технологии

- **Rust** с **Axum** (веб-фреймворк)
- **PostgreSQL** (база данных)
- **SQLx** (асинхронный драйвер для PostgreSQL)
- **JWT** (авторизация)
- **Telegram WebApp SDK** (авторизация через Telegram)

## 📋 Требования

- Rust 1.81+
- PostgreSQL 12+
- Docker и Docker Compose (опционально)

## 🔧 Установка

### Локальная разработка

1. Клонируйте репозиторий
2. Установите зависимости:

```bash
cargo build
```

3. Создайте `.env` файл (скопируйте из `.env.example`):

```bash
DATABASE_URL=postgres://alien_user:alien_password@localhost:5432/alien_game
TELEGRAM_BOT_TOKEN=your_bot_token_here
JWT_SECRET=your_secret_key
PORT=8000
```

4. Запустите PostgreSQL и выполните миграции:

```bash
# Через docker-compose
docker-compose up -d postgres

# Или вручную
psql -U alien_user -d alien_game -f migrations/20240101000001_create_tables.sql
```

5. Запустите сервер:

```bash
cargo run
```

### Docker

1. Соберите и запустите через Docker Compose:

```bash
docker-compose up --build
```

Сервер будет доступен на `http://localhost:8000`

## 📡 API Эндпоинты

### Health Check

```
GET /health
```

Ответ: `{ "status": "ok" }`

### Авторизация

#### POST `/auth/telegram`

Авторизация через Telegram WebApp initData.

**Запрос:**

```json
{
  "user": {
    "id": 123456789,
    "username": "user",
    "first_name": "John",
    "last_name": "Doe"
  },
  "hash": "abc123...",
  "auth_date": "1234567890",
  ...
}
```

**Ответ:**

```json
{
  "token": "jwt_token_here",
  "user_id": "uuid-here"
}
```

### Игра

#### POST `/game/update_score`

Обновляет счет пользователя. Требует JWT токен в заголовке `Authorization: Bearer <token>`.

**Запрос:**

```json
{
  "score": 1000
}
```

**Ответ:**

```json
{
  "success": true,
  "score": 1000
}
```

#### GET `/game/leaderboard`

Возвращает топ-10 игроков по очкам.

**Ответ:**

```json
[
  {
    "user_id": "uuid",
    "username": "player1",
    "first_name": "Player",
    "score": 5000
  },
  ...
]
```

### Вывод токенов

#### POST `/claim/start`

Инициализирует запрос на вывод токенов. Требует JWT токен.

**Запрос:**

```json
{
  "amount": 100.5
}
```

**Ответ:**

```json
{
  "claim_id": "uuid",
  "status": "pending"
}
```

#### POST `/claim/confirm`

Подтверждает и завершает транзакцию вывода. Требует JWT токен.

**Запрос:**

```json
{
  "claim_id": "uuid"
}
```

**Ответ:**

```json
{
  "success": true,
  "status": "completed"
}
```

## 🔐 Авторизация

Все эндпоинты кроме `/auth/telegram`, `/game/leaderboard` и `/health` требуют JWT токен в заголовке:

```
Authorization: Bearer <jwt_token>
```

Токен получается при авторизации через `/auth/telegram`.

## 🗄️ База данных

### Таблицы

#### users

- `id` (UUID) - первичный ключ
- `telegram_id` (BIGINT) - уникальный ID из Telegram
- `username` (TEXT) - username пользователя
- `first_name` (TEXT) - имя
- `last_name` (TEXT) - фамилия
- `created_at` (TIMESTAMP) - дата создания

#### scores

- `id` (UUID) - первичный ключ
- `user_id` (UUID) - внешний ключ на users
- `score` (INT) - очки игрока
- `updated_at` (TIMESTAMP) - дата обновления

#### claims

- `id` (UUID) - первичный ключ
- `user_id` (UUID) - внешний ключ на users
- `amount` (DECIMAL) - сумма вывода
- `status` (TEXT) - статус ('pending', 'completed')
- `created_at` (TIMESTAMP) - дата создания

## 🐳 Docker

### Docker Compose

`docker-compose.yml` включает:

- PostgreSQL контейнер
- Backend контейнер

Запуск:

```bash
docker-compose up --build
```

### Dockerfile

Для сборки standalone образа:

```bash
docker build -t alien-tap-backend .
docker run -p 8000:8000 alien-tap-backend
```

## 📝 Переменные окружения

| Переменная           | Описание                         | Обязательно |
| -------------------- | -------------------------------- | ----------- |
| `DATABASE_URL`       | PostgreSQL connection string     | Да          |
| `TELEGRAM_BOT_TOKEN` | Токен бота от @BotFather         | Да          |
| `JWT_SECRET`         | Секретный ключ для JWT           | Да          |
| `PORT`               | Порт сервера (по умолчанию 8000) | Нет         |

## 🧪 Тестирование

Пример теста с curl:

```bash
# Health check
curl http://localhost:8000/health

# Авторизация
curl -X POST http://localhost:8000/auth/telegram \
  -H "Content-Type: application/json" \
  -d '{"user": {...}, "hash": "..."}'

# Обновление счета (требует токен)
curl -X POST http://localhost:8000/game/update_score \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"score": 100}'

# Лидерборд
curl http://localhost:8000/game/leaderboard
```

## 🔄 Миграции

Миграции находятся в папке `migrations/`. Для применения миграций:

1. Через SQLx CLI:

```bash
sqlx migrate run
```

2. Или вручную через psql:

```bash
psql -U alien_user -d alien_game -f migrations/20240101000001_create_tables.sql
```

## 🚢 Деплой

Проект готов к деплою на:

- **Render**
- **Railway**
- **Fly.io**
- **Heroku**
- Любой другой PaaS с поддержкой Rust и PostgreSQL

Убедитесь, что установлены все переменные окружения.

## 📦 Структура проекта

```
src/
 ├── main.rs          # Точка входа, настройка сервера
 ├── config.rs        # Конфигурация из env
 ├── db.rs            # Подключение к БД
 ├── routes/          # Эндпоинты API
 │    ├── auth.rs     # Авторизация Telegram
 │    ├── game.rs     # Игровые эндпоинты
 │    └── claim.rs    # Вывод токенов
 ├── models/          # Модели данных
 │    ├── user.rs
 │    ├── score.rs
 │    └── claim.rs
 └── utils/           # Утилиты
      ├── telegram.rs # Верификация Telegram
      ├── jwt.rs      # JWT токены
      └── errors.rs   # Обработка ошибок
```

## 📄 Лицензия

MIT

## 🤝 Вклад

Приветствуются PR и Issues!
# alien-tap-backend
