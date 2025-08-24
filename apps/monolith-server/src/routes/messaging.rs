use axum::{routing::{get, post}, Router};
use crate::state::AppState;
use crate::messaging::{
    create_conversation,
    list_conversations,
    get_conversation,
    send_message,
    list_messages,
    mark_messages_read,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/conversations", post(create_conversation))
        .route("/conversations", get(list_conversations))
        .route("/conversations/:conversation_id", get(get_conversation))
        .route("/conversations/:conversation_id/messages", post(send_message))
        .route("/conversations/:conversation_id/messages", get(list_messages))
        .route("/conversations/:conversation_id/read", post(mark_messages_read))
}
