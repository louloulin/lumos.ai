/*!
# Console Module Tests

测试流式聊天系统的核心功能。

## 测试覆盖

- 消息创建和管理
- 对话状态管理
- 时间格式化
- 消息分组
*/

#[cfg(test)]
mod tests {
    use super::*;
    use web_pages::console::*;
    use chrono::Utc;

    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: Some("Hello, AI!".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        assert_eq!(message.role, ChatRole::User);
        assert_eq!(message.content, Some("Hello, AI!".to_string()));
        assert!(message.tool_calls.is_none());
    }

    #[test]
    fn test_chat_conversation_creation() {
        let message = ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: Some("Test message".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        let conversation = ChatConversation {
            id: 1,
            title: "Test Conversation".to_string(),
            messages: vec![message],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(conversation.title, "Test Conversation");
        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.messages[0].role, ChatRole::User);
    }

    #[test]
    fn test_chat_state_default() {
        let state = ChatState::default();

        assert!(state.current_conversation.is_none());
        assert!(state.conversations.is_empty());
        assert_eq!(state.pending_state, PendingChatState::None);
        assert!(!state.is_streaming);
        assert!(!state.is_locked);
    }

    #[test]
    fn test_truncate_title() {
        // 测试短标题
        let short_title = "Short title";
        assert_eq!(truncate_title(short_title), "Short title");

        // 测试长标题
        let long_title = "This is a very long title that should be truncated because it exceeds the maximum length";
        let truncated = truncate_title(long_title);
        assert!(truncated.len() <= 50);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_generate_message_id() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        
        // IDs should be different
        assert_ne!(id1, id2);
        
        // IDs should be positive
        assert!(id1 > 0);
        assert!(id2 > 0);
    }

    #[test]
    fn test_chat_roles() {
        let user_role = ChatRole::User;
        let assistant_role = ChatRole::Assistant;
        let system_role = ChatRole::System;
        let tool_role = ChatRole::Tool;

        assert_ne!(user_role, assistant_role);
        assert_ne!(assistant_role, system_role);
        assert_ne!(system_role, tool_role);
    }

    #[test]
    fn test_pending_chat_state() {
        let none_state = PendingChatState::None;
        assert_eq!(none_state, PendingChatState::None);

        // 测试状态变化
        let message = ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: Some("Test".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        // 注意：这里我们不能直接测试 PendingUserChat，因为它需要 PendingChat 结构
        // 这个测试主要验证枚举的基本功能
    }

    #[test]
    fn test_conversation_with_multiple_messages() {
        let user_message = ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: Some("Hello".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        let assistant_message = ChatMessage {
            id: 2,
            role: ChatRole::Assistant,
            content: Some("Hi there!".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        let conversation = ChatConversation {
            id: 1,
            title: "Test Chat".to_string(),
            messages: vec![user_message, assistant_message],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(conversation.messages.len(), 2);
        assert_eq!(conversation.messages[0].role, ChatRole::User);
        assert_eq!(conversation.messages[1].role, ChatRole::Assistant);
    }

    #[test]
    fn test_message_serialization() {
        let message = ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: Some("Test message".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        // 测试序列化
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("User"));
        assert!(serialized.contains("Test message"));

        // 测试反序列化
        let deserialized: ChatMessage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, message.id);
        assert_eq!(deserialized.role, message.role);
        assert_eq!(deserialized.content, message.content);
    }

    #[test]
    fn test_conversation_serialization() {
        let message = ChatMessage {
            id: 1,
            role: ChatRole::User,
            content: Some("Test".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            tool_calls: None,
            tool_call_id: None,
        };

        let conversation = ChatConversation {
            id: 1,
            title: "Test".to_string(),
            messages: vec![message],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        // 测试序列化
        let serialized = serde_json::to_string(&conversation).unwrap();
        assert!(serialized.contains("Test"));

        // 测试反序列化
        let deserialized: ChatConversation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, conversation.id);
        assert_eq!(deserialized.title, conversation.title);
        assert_eq!(deserialized.messages.len(), 1);
    }
}
