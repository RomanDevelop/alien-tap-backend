mod app_state;
mod config;
mod db;
mod models;
mod routes;
mod utils;

use axum::{
    http::Method,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::classify::ServerErrorsFailureClass;
use tracing_subscriber;

use crate::app_state::AppState;
use config::Config;
use db::create_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "alien_tap_backend=debug,tower_http=debug,axum=debug".into()),
        )
        .init();
    
    // Загрузка конфигурации
    let config = Config::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    
    tracing::info!("Starting server on port {}", config.port);
    
    // Подключение к базе данных
    let pool = create_pool(&config).await?;
    tracing::info!("Connected to database");
    
    // Настройка CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);
    
    // Создание состояния приложения
    let app_state = AppState {
        pool,
        config: config.clone(),
    };
    
    // Создание роутера
    let app = Router::new()
        .route("/health", get(health))
        .nest("/auth", routes::auth::router())
        .nest("/game", routes::game::router())
        .nest("/claim", routes::claim::router())
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                                version = ?request.version(),
                            )
                        })
                        .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                            tracing::info!(
                                "→ {} {}",
                                request.method(),
                                request.uri().path()
                            );
                        })
                        .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                            tracing::info!(
                                "← {} ({}ms)",
                                response.status(),
                                latency.as_millis()
                            );
                        })
                        .on_failure(|error: ServerErrorsFailureClass, _latency: std::time::Duration, _span: &tracing::Span| {
                            tracing::error!("✗ Request failed: {:?}", error);
                        }),
                )
                .layer(cors)
        )
        .with_state(app_state);
    
    // Запуск сервера
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    
    tracing::info!("Server listening on http://0.0.0.0:{}", config.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
