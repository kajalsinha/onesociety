use monolith_server::jwt::{create_access_token, create_refresh_token, verify_token, Claims};
use uuid::Uuid;

#[tokio::test]
async fn test_create_access_token() {
    let user_id = Uuid::new_v4();
    let token = create_access_token(user_id).unwrap();
    
    // Verify the token can be decoded
    let claims = verify_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.token_type, "access");
}

#[tokio::test]
async fn test_create_refresh_token() {
    let user_id = Uuid::new_v4();
    let token = create_refresh_token(user_id).unwrap();
    
    // Verify the token can be decoded
    let claims = verify_token(&token).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.token_type, "refresh");
}

#[tokio::test]
async fn test_verify_invalid_token() {
    let result = verify_token("invalid.token.here");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_expiration() {
    let user_id = Uuid::new_v4();
    let token = create_access_token(user_id).unwrap();
    
    // Token should be valid immediately after creation
    let claims = verify_token(&token).unwrap();
    assert!(claims.exp > chrono::Utc::now().timestamp());
}

#[tokio::test]
async fn test_different_user_ids() {
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    
    let token1 = create_access_token(user1).unwrap();
    let token2 = create_access_token(user2).unwrap();
    
    let claims1 = verify_token(&token1).unwrap();
    let claims2 = verify_token(&token2).unwrap();
    
    assert_eq!(claims1.sub, user1.to_string());
    assert_eq!(claims2.sub, user2.to_string());
    assert_ne!(claims1.sub, claims2.sub);
}

#[tokio::test]
async fn test_password_hashing() {
    use monolith_server::jwt::{hash_password, verify_password};
    
    let password = "test_password_123";
    let hash = hash_password(password).await.unwrap();
    
    // Verify the password
    let is_valid = verify_password(password, &hash).await.unwrap();
    assert!(is_valid);
    
    // Verify wrong password fails
    let is_invalid = verify_password("wrong_password", &hash).await.unwrap();
    assert!(!is_invalid);
}
