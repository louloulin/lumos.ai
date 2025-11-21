//! Agent message utilities

use crate::llm::{Message, Role};

/// Create a system message
pub fn system_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::System,
        content: content.into(),
        name: None,
        metadata: None,
    }
}

/// Create a user message
pub fn user_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::User,
        content: content.into(),
        name: None,
        metadata: None,
    }
}

/// Create an assistant message
pub fn assistant_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::Assistant,
        content: content.into(),
        name: None,
        metadata: None,
    }
}

/// Create a tool message
pub fn tool_message(content: impl Into<String>, tool_name: impl Into<String>) -> Message {
    Message {
        role: Role::Tool,
        content: content.into(),
        name: Some(tool_name.into()),
        metadata: None,
    }
}

/// Create a message with metadata
pub fn message_with_metadata(
    role: Role,
    content: impl Into<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
) -> Message {
    Message {
        role,
        content: content.into(),
        name: None,
        metadata: Some(metadata),
    }
}

/// Create a message with name
pub fn message_with_name(
    role: Role,
    content: impl Into<String>,
    name: impl Into<String>,
) -> Message {
    Message {
        role,
        content: content.into(),
        name: Some(name.into()),
        metadata: None,
    }
}

/// Format messages for display
pub fn format_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .enumerate()
        .map(|(i, msg)| {
            format!(
                "[{}] {}: {}",
                i + 1,
                format_role(&msg.role),
                msg.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format role for display
pub fn format_role(role: &Role) -> String {
    match role {
        Role::System => "System".to_string(),
        Role::User => "User".to_string(),
        Role::Assistant => "Assistant".to_string(),
        Role::Tool => "Tool".to_string(),
        Role::Function => "Function".to_string(),
        Role::Custom(s) => format!("Custom({})", s),
    }
}

/// Extract text content from messages
pub fn extract_text_content(messages: &[Message]) -> Vec<String> {
    messages.iter().map(|msg| msg.content.clone()).collect()
}

/// Filter messages by role
pub fn filter_messages_by_role(messages: &[Message], role: Role) -> Vec<Message> {
    messages
        .iter()
        .filter(|msg| msg.role == role)
        .cloned()
        .collect()
}

/// Count messages by role
pub fn count_messages_by_role(messages: &[Message], role: Role) -> usize {
    messages.iter().filter(|msg| msg.role == role).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_message_creation() {
        let system = system_message("System instruction");
        assert_eq!(system.role, Role::System);
        assert_eq!(system.content, "System instruction");

        let user = user_message("User query");
        assert_eq!(user.role, Role::User);
        assert_eq!(user.content, "User query");

        let assistant = assistant_message("Assistant response");
        assert_eq!(assistant.role, Role::Assistant);
        assert_eq!(assistant.content, "Assistant response");

        let tool = tool_message("Tool result", "calculator");
        assert_eq!(tool.role, Role::Tool);
        assert_eq!(tool.content, "Tool result");
        assert_eq!(tool.name, Some("calculator".to_string()));
    }

    #[test]
    fn test_message_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), serde_json::json!("value1"));
        metadata.insert("key2".to_string(), serde_json::json!(42));

        let msg = message_with_metadata(Role::User, "Test content", metadata.clone());
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Test content");
        assert!(msg.metadata.is_some());
        assert_eq!(msg.metadata.as_ref().unwrap(), &metadata);
    }

    #[test]
    fn test_message_with_name() {
        let msg = message_with_name(Role::Assistant, "Test content", "agent-1");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "Test content");
        assert_eq!(msg.name, Some("agent-1".to_string()));
    }

    #[test]
    fn test_format_role() {
        assert_eq!(format_role(&Role::System), "System");
        assert_eq!(format_role(&Role::User), "User");
        assert_eq!(format_role(&Role::Assistant), "Assistant");
        assert_eq!(format_role(&Role::Tool), "Tool");
        assert_eq!(format_role(&Role::Function), "Function");
        assert_eq!(format_role(&Role::Custom("test".to_string())), "Custom(test)");
    }

    #[test]
    fn test_format_messages() {
        let messages = vec![
            system_message("System message"),
            user_message("User message"),
            assistant_message("Assistant message"),
        ];

        let formatted = format_messages(&messages);
        assert!(formatted.contains("System"));
        assert!(formatted.contains("User"));
        assert!(formatted.contains("Assistant"));
        assert!(formatted.contains("System message"));
        assert!(formatted.contains("User message"));
        assert!(formatted.contains("Assistant message"));
    }

    #[test]
    fn test_extract_text_content() {
        let messages = vec![
            system_message("System message"),
            user_message("User message"),
            assistant_message("Assistant message"),
        ];

        let contents = extract_text_content(&messages);
        assert_eq!(contents.len(), 3);
        assert_eq!(contents[0], "System message");
        assert_eq!(contents[1], "User message");
        assert_eq!(contents[2], "Assistant message");
    }

    #[test]
    fn test_filter_messages_by_role() {
        let messages = vec![
            system_message("System message"),
            user_message("User message 1"),
            assistant_message("Assistant message"),
            user_message("User message 2"),
        ];

        let user_messages = filter_messages_by_role(&messages, Role::User);
        assert_eq!(user_messages.len(), 2);
        assert_eq!(user_messages[0].content, "User message 1");
        assert_eq!(user_messages[1].content, "User message 2");

        let system_messages = filter_messages_by_role(&messages, Role::System);
        assert_eq!(system_messages.len(), 1);
        assert_eq!(system_messages[0].content, "System message");
    }

    #[test]
    fn test_count_messages_by_role() {
        let messages = vec![
            system_message("System message"),
            user_message("User message 1"),
            assistant_message("Assistant message"),
            user_message("User message 2"),
            assistant_message("Assistant message 2"),
        ];

        assert_eq!(count_messages_by_role(&messages, Role::System), 1);
        assert_eq!(count_messages_by_role(&messages, Role::User), 2);
        assert_eq!(count_messages_by_role(&messages, Role::Assistant), 2);
        assert_eq!(count_messages_by_role(&messages, Role::Tool), 0);
    }

    #[test]
    fn test_empty_messages() {
        let messages = vec![];
        assert_eq!(extract_text_content(&messages).len(), 0);
        assert_eq!(filter_messages_by_role(&messages, Role::User).len(), 0);
        assert_eq!(count_messages_by_role(&messages, Role::User), 0);
        assert_eq!(format_messages(&messages), "");
    }
}
