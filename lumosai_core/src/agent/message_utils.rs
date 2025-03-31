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

#[cfg(test)]
mod tests {
    use super::*;
    
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
} 