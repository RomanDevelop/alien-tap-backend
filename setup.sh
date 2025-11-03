#!/bin/bash

# Скрипт для быстрой настройки проекта

echo "🚀 Настройка Alien Tap Backend..."

# Проверка наличия Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust не установлен. Установите Rust с https://rustup.rs/"
    exit 1
fi

echo "✅ Rust найден"

# Создание .env файла, если его нет
if [ ! -f .env ]; then
    echo "📝 Создание .env файла из примера..."
    cp env.example .env
    echo "⚠️  Отредактируйте .env файл и добавьте свои значения!"
fi

# Проверка наличия PostgreSQL
if ! command -v psql &> /dev/null; then
    echo "⚠️  PostgreSQL не найден. Используйте Docker Compose для запуска БД."
else
    echo "✅ PostgreSQL найден"
fi

# Запуск через Docker Compose (рекомендуется)
if command -v docker-compose &> /dev/null || command -v docker compose &> /dev/null; then
    echo "🐳 Запуск через Docker Compose..."
    
    if command -v docker compose &> /dev/null; then
        docker compose up -d postgres
    else
        docker-compose up -d postgres
    fi
    
    echo "✅ PostgreSQL запущен в Docker"
    echo "⏳ Ожидание готовности PostgreSQL..."
    sleep 5
fi

# Сборка проекта
echo "🔨 Сборка проекта..."
cargo build --release

echo "✅ Проект готов!"
echo ""
echo "Для запуска используйте:"
echo "  cargo run"
echo ""
echo "Или через Docker Compose:"
echo "  docker-compose up"
