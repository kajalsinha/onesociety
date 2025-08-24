use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: i64,    // expiration
    pub iat: i64,    // issued at
    pub token_type: String,
}

const JWT_SECRET: &str = "your-secret-key-change-in-production";
const ACCESS_TOKEN_EXPIRY: i64 = 3600; // 1 hour
const REFRESH_TOKEN_EXPIRY: i64 = 2592000; // 30 days

pub fn create_access_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + Duration::seconds(ACCESS_TOKEN_EXPIRY)).timestamp(),
        iat: now.timestamp(),
        token_type: "access".to_string(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.as_ref()))
}

pub fn create_refresh_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + Duration::seconds(REFRESH_TOKEN_EXPIRY)).timestamp(),
        iat: now.timestamp(),
        token_type: "refresh".to_string(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.as_ref()))
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

pub async fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub async fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}
