# 🚀 Быстрый старт

## Вариант 1: Docker Compose (рекомендуется)

1. Скопируйте `env.example` в `.env` и заполните значения:

```bash
cp env.example .env
# Отредактируйте .env и добавьте TELEGRAM_BOT_TOKEN
```

2. Запустите проект:

```bash
docker-compose up --build
```

Сервер будет доступен на `http://localhost:8000`

## Вариант 2: Локальная разработка

1. Установите Rust (если еще не установлен):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Установите PostgreSQL или запустите через Docker:

```bash
docker-compose up -d postgres
```

3. Создайте `.env` файл:

```bash
cp env.example .env
# Отредактируйте .env
```

4. Примените миграции:

```bash
# Через psql
psql -U alien_user -d alien_game -f migrations/20240101000001_create_tables.sql

# Или миграции применятся автоматически через docker-compose
```

5. Соберите и запустите:

```bash
cargo build --release
cargo run
```

## Тестирование API

### Health Check

```bash
curl http://localhost:8000/health
```

### Авторизация через Telegram

```bash
curl -X POST http://localhost:8000/auth/telegram \
  -H "Content-Type: application/json" \
  -d '{
    "user": {
      "id": 123456789,
      "username": "testuser",
      "first_name": "Test"
    },
    "hash": "your_telegram_hash_here",
    "auth_date": "1234567890"
  }'
```

### Обновление счёта

```bash
curl -X POST http://localhost:8000/game/update_score \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"score": 1000}'
```

### Лидерборд

```bash
curl http://localhost:8000/game/leaderboard
```

## Полезные команды

```bash
# Сборка
make build

# Запуск
make run

# Docker
make docker-up
make docker-down
make docker-logs

# Проверка кода
make check
make lint
make format
```

## Переменные окружения

| Переменная           | Описание                     | Пример                                   |
| -------------------- | ---------------------------- | ---------------------------------------- |
| `DATABASE_URL`       | PostgreSQL connection string | `postgres://user:pass@localhost:5432/db` |
| `TELEGRAM_BOT_TOKEN` | Токен бота от @BotFather     | `1234567890:ABC...`                      |
| `JWT_SECRET`         | Секретный ключ для JWT       | `your-secret-key`                        |
| `PORT`               | Порт сервера                 | `8000`                                   |

## Структура проекта

```
alien-tap-backend/
├── src/                    # Исходный код
│   ├── main.rs            # Точка входа
│   ├── config.rs          # Конфигурация
│   ├── db.rs              # База данных
│   ├── models/            # Модели данных
│   ├── routes/            # API эндпоинты
│   └── utils/             # Утилиты
├── migrations/            # SQL миграции
├── Dockerfile             # Docker образ
├── docker-compose.yml     # Docker Compose
├── Cargo.toml             # Зависимости Rust
├── README.md              # Документация
└── .env                   # Переменные окружения (создать из env.example)
```

## Следующие шаги

1. Получите токен бота у [@BotFather](https://t.me/BotFather)
2. Настройте переменные окружения в `.env`
3. Запустите проект
4. Интегрируйте с Flutter WebApp (см. `INTEGRATION_EXAMPLE.md`)

## Деплой

Проект готов к деплою на:

- **Render**: Добавьте `Dockerfile` и настройте env переменные
- **Railway**: Используйте Dockerfile или Rust buildpack
- **Fly.io**: `fly launch` и настройте PostgreSQL
- **Heroku**: Добавьте `Procfile` с командой `./target/release/alien-tap-backend`
