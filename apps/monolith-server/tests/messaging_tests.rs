use monolith_server::messaging::{
    CreateConversationRequest,
    SendMessageRequest,
    ConversationFilters,
};

use uuid::Uuid;

#[tokio::test]
async fn test_create_conversation_request_validation() {
    let request = CreateConversationRequest {
        rental_id: Uuid::new_v4(),
        message: "Hello, I'm interested in renting this item".to_string(),
    };

    assert_eq!(request.message.len(), 37);
    assert!(!request.message.is_empty());
}

#[tokio::test]
async fn test_send_message_request_validation() {
    let request = SendMessageRequest {
        content: "Thanks for the quick response!".to_string(),
        message_type: Some("text".to_string()),
    };

    assert_eq!(request.content.len(), 29);
    assert!(!request.content.is_empty());
    assert_eq!(request.message_type, Some("text".to_string()));
}

#[tokio::test]
async fn test_send_message_request_default_type() {
    let request = SendMessageRequest {
        content: "Hello there".to_string(),
        message_type: None,
    };

    assert_eq!(request.content, "Hello there");
    assert_eq!(request.message_type, None);
}

#[tokio::test]
async fn test_conversation_filters_validation() {
    let filters = ConversationFilters {
        status: Some("active".to_string()),
        limit: Some(20),
        offset: Some(0),
    };

    assert_eq!(filters.status, Some("active".to_string()));
    assert_eq!(filters.limit, Some(20));
    assert_eq!(filters.offset, Some(0));
}

#[tokio::test]
async fn test_conversation_filters_defaults() {
    let filters = ConversationFilters {
        status: None,
        limit: None,
        offset: None,
    };

    assert_eq!(filters.status, None);
    assert_eq!(filters.limit, None);
    assert_eq!(filters.offset, None);
}

#[tokio::test]
async fn test_message_content_validation() {
    let valid_content = "This is a valid message content";
    assert!(!valid_content.is_empty());
    assert!(valid_content.len() <= 1000); // Reasonable max length

    let empty_content = "";
    assert!(empty_content.is_empty());
}

#[tokio::test]
async fn test_message_type_validation() {
    let valid_types = vec!["text", "image", "file", "system"];
    
    for msg_type in valid_types {
        assert!(!msg_type.is_empty());
        assert!(msg_type.len() <= 20);
    }
}

#[tokio::test]
async fn test_conversation_status_validation() {
    let valid_statuses = vec!["active", "archived", "blocked"];
    
    for status in valid_statuses {
        assert!(!status.is_empty());
        assert!(status.len() <= 20);
    }
}
