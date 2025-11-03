use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation, errors::Error};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
    pub iat: usize,
}

pub fn create_jwt(user_id: &str, secret: &str) -> Result<String, Error> {
    let now = Utc::now().timestamp() as usize;
    let expiration = now + (60 * 60 * 24); // 24 часа
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: now,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref())
    )
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, Error> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;
    
    Ok(token_data.claims)
}
