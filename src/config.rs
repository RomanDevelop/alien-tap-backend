use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub telegram_bot_token: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok(); // Загружаем .env, но не падаем если его нет
        
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN")?,
            jwt_secret: env::var("JWT_SECRET")?,
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
        })
    }
}
