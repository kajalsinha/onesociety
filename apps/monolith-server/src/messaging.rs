use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::{err, ok, AppState};

// Request/Response Models
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub rental_id: Uuid,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub message_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationFilters {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub rental_id: Uuid,
    pub owner_id: Uuid,
    pub renter_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message: Option<MessageResponse>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ConversationListResponse {
    pub conversations: Vec<ConversationResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// Helper function to extract user ID from JWT token
async fn extract_user_id_from_token(headers: &HeaderMap) -> Result<Uuid, (axum::http::StatusCode, String)> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid authorization header format".to_string()));
    }

    let token = &auth_header[7..];
    let claims = crate::jwt::verify_token(token)
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    Uuid::parse_str(&claims.sub)
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid user ID in token".to_string()))
}

// Handlers
pub async fn create_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateConversationRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Verify user is the renter of this rental
    let rental = sqlx::query!(
        "SELECT renter_id FROM rental_schema.rentals WHERE id = $1",
        request.rental_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let rental = rental.ok_or((axum::http::StatusCode::NOT_FOUND, "Rental not found"))?;

    if rental.renter_id != user_id {
        return err(axum::http::StatusCode::FORBIDDEN, "You can only create conversations for your own rentals");
    }

    // Get product owner
    let product = sqlx::query!(
        "SELECT owner_id FROM product_schema.products WHERE id = (SELECT product_id FROM rental_schema.rentals WHERE id = $1)",
        request.rental_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Check if conversation already exists
    let existing_conversation = sqlx::query!(
        "SELECT id FROM messaging_schema.conversations WHERE rental_id = $1 AND owner_id = $2 AND renter_id = $3",
        request.rental_id,
        product.owner_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if let Some(conv) = existing_conversation {
        let msg = format!("Conversation already exists: {}", conv.id);
        return err(axum::http::StatusCode::CONFLICT, &msg);
    }

    // Create conversation and first message in a transaction
    let result = sqlx::query!(
        r#"
        WITH new_conversation AS (
            INSERT INTO messaging_schema.conversations (rental_id, owner_id, renter_id)
            VALUES ($1, $2, $3)
            RETURNING id, created_at, updated_at
        )
        INSERT INTO messaging_schema.messages (conversation_id, sender_id, content, message_type)
        SELECT id, $4, $5, 'text'
        FROM new_conversation
        RETURNING conversation_id, id as message_id, created_at as message_created_at
        "#,
        request.rental_id,
        product.owner_id,
        user_id,
        user_id,
        request.message
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = ConversationResponse {
        id: result.conversation_id,
        rental_id: request.rental_id,
        owner_id: product.owner_id,
        renter_id: user_id,
        status: "active".to_string(),
        created_at: result.message_created_at,
        updated_at: result.message_created_at,
        last_message: Some(MessageResponse {
            id: result.message_id,
            conversation_id: result.conversation_id,
            sender_id: user_id,
            content: request.message,
            message_type: "text".to_string(),
            is_read: false,
            created_at: result.message_created_at,
        }),
        unread_count: 0,
    };

    ok(response)
}

pub async fn list_conversations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(filters): Query<ConversationFilters>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let limit = filters.limit.unwrap_or(20).min(100);
    let offset = filters.offset.unwrap_or(0);
    let status = filters.status.unwrap_or_else(|| "active".to_string().to_string());

    let conversations = sqlx::query!(
        r#"
        SELECT 
            c.id, c.rental_id, c.owner_id, c.renter_id, c.status, c.created_at, c.updated_at,
            m.id as last_message_id, m.content as last_message_content, m.message_type as last_message_type,
            m.created_at as last_message_created_at, m.sender_id as last_message_sender_id,
            COUNT(CASE WHEN msg.is_read = false AND msg.sender_id != $1 THEN 1 END) as unread_count
        FROM messaging_schema.conversations c
        LEFT JOIN LATERAL (
            SELECT id, content, message_type, created_at, sender_id
            FROM messaging_schema.messages
            WHERE conversation_id = c.id
            ORDER BY created_at DESC
            LIMIT 1
        ) m ON true
        LEFT JOIN messaging_schema.messages msg ON msg.conversation_id = c.id
        WHERE (c.owner_id = $1 OR c.renter_id = $1)
        AND c.status = $2
        GROUP BY c.id, c.rental_id, c.owner_id, c.renter_id, c.status, c.created_at, c.updated_at,
                 m.id, m.content, m.message_type, m.created_at, m.sender_id
        ORDER BY c.updated_at DESC
        LIMIT $3 OFFSET $4
        "#,
        user_id,
        status,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total = sqlx::query!(
        "SELECT COUNT(*) as count FROM messaging_schema.conversations WHERE (owner_id = $1 OR renter_id = $1) AND status = $2",
        user_id,
        status
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let conversation_responses: Vec<ConversationResponse> = conversations
        .into_iter()
        .map(|row| ConversationResponse {
            id: row.id,
            rental_id: row.rental_id,
            owner_id: row.owner_id,
            renter_id: row.renter_id,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_message: row.last_message_id.map(|_| MessageResponse {
                id: row.last_message_id.unwrap(),
                conversation_id: row.id,
                sender_id: row.last_message_sender_id.unwrap(),
                content: row.last_message_content.unwrap(),
                message_type: row.last_message_type.unwrap(),
                is_read: false,
                created_at: row.last_message_created_at.unwrap(),
            }),
            unread_count: row.unread_count.unwrap_or(0),
        })
        .collect();

    let response = ConversationListResponse {
        conversations: conversation_responses,
        total,
        limit,
        offset,
    };

    ok(response)
}

pub async fn get_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    let conversation = sqlx::query!(
        r#"
        SELECT id, rental_id, owner_id, renter_id, status, created_at, updated_at
        FROM messaging_schema.conversations
        WHERE id = $1 AND (owner_id = $2 OR renter_id = $2)
        "#,
        conversation_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let conversation = conversation.ok_or((axum::http::StatusCode::NOT_FOUND, "Conversation not found"))?;

    let last_message = sqlx::query!(
        r#"
        SELECT id, content, message_type, created_at, sender_id
        FROM messaging_schema.messages
        WHERE conversation_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        conversation_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let unread_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM messaging_schema.messages WHERE conversation_id = $1 AND is_read = false AND sender_id != $2",
        conversation_id,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let response = ConversationResponse {
        id: conversation.id,
        rental_id: conversation.rental_id,
        owner_id: conversation.owner_id,
        renter_id: conversation.renter_id,
        status: conversation.status,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
        last_message: last_message.map(|m| MessageResponse {
            id: m.id,
            conversation_id,
            sender_id: m.sender_id,
            content: m.content,
            message_type: m.message_type,
            is_read: false,
            created_at: m.created_at,
        }),
        unread_count,
    };

    ok(response)
}

pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(request): Json<SendMessageRequest>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Verify user is part of the conversation
    let conversation = sqlx::query!(
        "SELECT id FROM messaging_schema.conversations WHERE id = $1 AND (owner_id = $2 OR renter_id = $2) AND status = 'active'",
        conversation_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if conversation.is_none() {
        return err(axum::http::StatusCode::NOT_FOUND, "Conversation not found or access denied");
    }

    let message_type = request.message_type.unwrap_or_else(|| "text".to_string().to_string());

    let message = sqlx::query!(
        r#"
        INSERT INTO messaging_schema.messages (conversation_id, sender_id, content, message_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, created_at
        "#,
        conversation_id,
        user_id,
        request.content,
        message_type
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Update conversation updated_at
    sqlx::query!(
        "UPDATE messaging_schema.conversations SET updated_at = NOW() WHERE id = $1",
        conversation_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = MessageResponse {
        id: message.id,
        conversation_id,
        sender_id: user_id,
        content: request.content,
        message_type,
        is_read: false,
        created_at: message.created_at,
    };

    ok(response)
}

pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Query(filters): Query<ConversationFilters>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Verify user is part of the conversation
    let conversation = sqlx::query!(
        "SELECT id FROM messaging_schema.conversations WHERE id = $1 AND (owner_id = $2 OR renter_id = $2)",
        conversation_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if conversation.is_none() {
        return err(axum::http::StatusCode::NOT_FOUND, "Conversation not found or access denied");
    }

    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = filters.offset.unwrap_or(0);

    let messages = sqlx::query!(
        r#"
        SELECT id, sender_id, content, message_type, is_read, created_at
        FROM messaging_schema.messages
        WHERE conversation_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        conversation_id,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let total = sqlx::query!(
        "SELECT COUNT(*) as count FROM messaging_schema.messages WHERE conversation_id = $1",
        conversation_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?
    .count;

    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|row| MessageResponse {
            id: row.id,
            conversation_id,
            sender_id: row.sender_id,
            content: row.content,
            message_type: row.message_type,
            is_read: row.is_read,
            created_at: row.created_at,
        })
        .collect();

    let response = MessageListResponse {
        messages: message_responses,
        total,
        limit,
        offset,
    };

    ok(response)
}

pub async fn mark_messages_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> impl axum::response::IntoResponse {
    let user_id = match extract_user_id_from_token(&headers).await {
        Ok(id) => id,
        Err(e) => return err(e.0, e.1.as_str()),
    };

    // Verify user is part of the conversation
    let conversation = sqlx::query!(
        "SELECT id FROM messaging_schema.conversations WHERE id = $1 AND (owner_id = $2 OR renter_id = $2)",
        conversation_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if conversation.is_none() {
        return err(axum::http::StatusCode::NOT_FOUND, "Conversation not found or access denied");
    }

    // Mark all unread messages from other users as read
    let result = sqlx::query!(
        "UPDATE messaging_schema.messages SET is_read = true WHERE conversation_id = $1 AND sender_id != $2 AND is_read = false",
        conversation_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    ok(serde_json::json!({
        "conversation_id": conversation_id,
        "messages_marked_read": result.rows_affected()
    }))
}
