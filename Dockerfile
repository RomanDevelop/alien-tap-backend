FROM rust:1.81 as builder

WORKDIR /app

# Устанавливаем зависимости системы для PostgreSQL
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Копируем файлы манифеста
COPY Cargo.toml Cargo.lock* ./

# Создаём фиктивный файл для кеширования зависимостей
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Копируем исходный код
COPY . .

# Собираем проект
RUN cargo build --release

# Финальный образ
FROM debian:bullseye-slim

WORKDIR /app

# Устанавливаем зависимости для runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Копируем бинарный файл
COPY --from=builder /app/target/release/alien-tap-backend /app/backend

# Копируем .env файл (опционально, может быть переопределён через docker-compose)
COPY .env* /app/

EXPOSE 8000

CMD ["./backend"]
