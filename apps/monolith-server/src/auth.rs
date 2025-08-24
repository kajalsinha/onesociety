use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{ok, err, AppState};
use crate::jwt::{create_access_token, create_refresh_token, verify_token, hash_password, verify_password};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileUpdateRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    // Check if user already exists
    let existing_user = sqlx::query!(
        "SELECT user_id FROM user_schema.users WHERE email = $1",
        req.email
    )
    .fetch_optional(&state.db)
    .await;

    if let Ok(Some(_)) = existing_user {
        return err(StatusCode::CONFLICT, "User already exists");
    }

    // Hash password
    let password_hash = match hash_password(&req.password).await {
        Ok(hash) => hash,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"),
    };

    // Create user
    let user_id = Uuid::new_v4();
    let result = sqlx::query!(
        "INSERT INTO user_schema.users (user_id, email, password_hash) VALUES ($1, $2, $3)",
        user_id,
        req.email,
        password_hash
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user");
    }

    // Create user profile
    let profile_result = sqlx::query!(
        "INSERT INTO user_schema.user_profiles (user_id, first_name, last_name) VALUES ($1, $2, $3)",
        user_id,
        req.first_name,
        req.last_name
    )
    .execute(&state.db)
    .await;

    if profile_result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user profile");
    }

    // Generate tokens
    let access_token = match create_access_token(user_id) {
        Ok(token) => token,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create access token"),
    };

    let refresh_token = match create_refresh_token(user_id) {
        Ok(token) => token,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create refresh token"),
    };

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserResponse {
            user_id,
            email: req.email,
            first_name: req.first_name,
            last_name: req.last_name,
        },
    };

    ok(response)
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    // Get user with password hash
    let user = sqlx::query!(
        "SELECT user_id, password_hash FROM user_schema.users WHERE email = $1",
        req.email
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => return err(StatusCode::UNAUTHORIZED, "Invalid credentials"),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
    };

    // Verify password
    let is_valid = match verify_password(&req.password, &user.password_hash).await {
        Ok(valid) => valid,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Password verification failed"),
    };

    if !is_valid {
        return err(StatusCode::UNAUTHORIZED, "Invalid credentials");
    }

    // Get user profile
    let profile = sqlx::query!(
        "SELECT first_name, last_name FROM user_schema.user_profiles WHERE user_id = $1",
        user.user_id
    )
    .fetch_one(&state.db)
    .await;

    let profile = match profile {
        Ok(profile) => profile,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user profile"),
    };

    // Update last login
    let _ = sqlx::query!(
        "UPDATE user_schema.users SET last_login = NOW() WHERE user_id = $1",
        user.user_id
    )
    .execute(&state.db)
    .await;

    // Generate tokens
    let access_token = match create_access_token(user.user_id) {
        Ok(token) => token,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create access token"),
    };

    let refresh_token = match create_refresh_token(user.user_id) {
        Ok(token) => token,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create refresh token"),
    };

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserResponse {
            user_id: user.user_id,
            email: req.email,
            first_name: profile.first_name,
            last_name: profile.last_name,
        },
    };

    ok(response)
}

pub async fn refresh(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let refresh_token = match req.get("refresh_token").and_then(|t| t.as_str()) {
        Some(token) => token,
        None => return err(StatusCode::BAD_REQUEST, "Refresh token required"),
    };

    // Verify refresh token
    let claims = match verify_token(refresh_token) {
        Ok(claims) => claims,
        Err(_) => return err(StatusCode::UNAUTHORIZED, "Invalid refresh token"),
    };

    if claims.token_type != "refresh" {
        return err(StatusCode::UNAUTHORIZED, "Invalid token type");
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return err(StatusCode::UNAUTHORIZED, "Invalid user ID in token"),
    };

    // Generate new access token
    let access_token = match create_access_token(user_id) {
        Ok(token) => token,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create access token"),
    };

    let response = serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer"
    });

    ok(response)
}

pub async fn get_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return err(StatusCode::UNAUTHORIZED, "Missing authorization header"),
    };

    let claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return err(StatusCode::UNAUTHORIZED, "Invalid token"),
    };

    if claims.token_type != "access" {
        return err(StatusCode::UNAUTHORIZED, "Invalid token type");
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return err(StatusCode::UNAUTHORIZED, "Invalid user ID in token"),
    };

    // Get user and profile
    let user_data = sqlx::query!(
        r#"
        SELECT u.email, up.first_name, up.last_name, up.avg_rating, up.total_reviews
        FROM user_schema.users u
        JOIN user_schema.user_profiles up ON u.user_id = up.user_id
        WHERE u.user_id = $1
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await;

    let user_data = match user_data {
        Ok(data) => data,
        Err(_) => return err(StatusCode::NOT_FOUND, "User not found"),
    };

    let response = serde_json::json!({
        "user_id": user_id,
        "email": user_data.email,
        "first_name": user_data.first_name,
        "last_name": user_data.last_name,
        "avg_rating": user_data.avg_rating,
        "total_reviews": user_data.total_reviews
    });

    ok(response)
}

pub async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ProfileUpdateRequest>,
) -> impl IntoResponse {
    let auth_header = headers.get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return err(StatusCode::UNAUTHORIZED, "Missing authorization header"),
    };

    let claims = match verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return err(StatusCode::UNAUTHORIZED, "Invalid token"),
    };

    if claims.token_type != "access" {
        return err(StatusCode::UNAUTHORIZED, "Invalid token type");
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return err(StatusCode::UNAUTHORIZED, "Invalid user ID in token"),
    };

    // Update profile
    let result = sqlx::query!(
        r#"
        UPDATE user_schema.user_profiles 
        SET 
            first_name = COALESCE($2, first_name),
            last_name = COALESCE($3, last_name),
            updated_at = NOW()
        WHERE user_id = $1
        "#,
        user_id,
        req.first_name,
        req.last_name
    )
    .execute(&state.db)
    .await;

    if result.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update profile");
    }

    ok(serde_json::json!({ "message": "Profile updated successfully" }))
}
