.PHONY: build run test docker-build docker-up docker-down migrate clean help

# Переменные
RUST_VERSION := 1.81
DOCKER_IMAGE := alien-tap-backend
DOCKER_COMPOSE := docker-compose

help: ## Показать эту справку
	@echo "Доступные команды:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Собрать проект
	cargo build --release

run: ## Запустить проект локально
	cargo run

test: ## Запустить тесты
	cargo test

docker-build: ## Собрать Docker образ
	docker build -t $(DOCKER_IMAGE) .

docker-up: ## Запустить через Docker Compose
	$(DOCKER_COMPOSE) up --build

docker-down: ## Остановить Docker Compose
	$(DOCKER_COMPOSE) down

docker-logs: ## Показать логи Docker
	$(DOCKER_COMPOSE) logs -f

migrate: ## Применить миграции (локально)
	psql -U alien_user -d alien_game -f migrations/20240101000001_create_tables.sql || \
	echo "Запустите PostgreSQL или используйте docker-compose up postgres"

clean: ## Очистить проект
	cargo clean
	rm -rf target/

lint: ## Проверить код линтером
	cargo clippy

format: ## Форматировать код
	cargo fmt

check: ## Проверить код без сборки
	cargo check
