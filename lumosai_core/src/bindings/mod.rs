//! Language bindings for Lumos.ai
//! 
//! This module provides bindings for different programming languages,
//! enabling developers to use Lumos.ai from their preferred language.

pub mod typescript;

pub use typescript::{
    TypeScriptBindings, TSAgentConfig, TSAgentResponse, TSToolDefinition,
    TSParameterSchema, TSPropertySchema, TSMemoryConfig, TSToolCall, TSUsageStats
};

/// Supported binding languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingLanguage {
    TypeScript,
    Python,
    Go,
    Java,
}

impl BindingLanguage {
    /// Get the file extension for the binding language
    pub fn extension(&self) -> &'static str {
        match self {
            BindingLanguage::TypeScript => "ts",
            BindingLanguage::Python => "py",
            BindingLanguage::Go => "go",
            BindingLanguage::Java => "java",
        }
    }

    /// Get the package manager for the binding language
    pub fn package_manager(&self) -> &'static str {
        match self {
            BindingLanguage::TypeScript => "npm",
            BindingLanguage::Python => "pip",
            BindingLanguage::Go => "go",
            BindingLanguage::Java => "maven",
        }
    }
}

/// Generate bindings for a specific language
pub fn generate_bindings(language: BindingLanguage) -> crate::Result<String> {
    match language {
        BindingLanguage::TypeScript => {
            Ok(typescript::TypeScriptBindings::generate_type_definitions())
        }
        _ => Err(crate::Error::Other(format!(
            "Bindings for {:?} are not yet implemented", 
            language
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_language_properties() {
        assert_eq!(BindingLanguage::TypeScript.extension(), "ts");
        assert_eq!(BindingLanguage::TypeScript.package_manager(), "npm");
        
        assert_eq!(BindingLanguage::Python.extension(), "py");
        assert_eq!(BindingLanguage::Python.package_manager(), "pip");
    }

    #[test]
    fn test_generate_typescript_bindings() {
        let result = generate_bindings(BindingLanguage::TypeScript);
        assert!(result.is_ok());
        
        let bindings = result.unwrap();
        assert!(bindings.contains("export interface AgentConfig"));
    }

    #[test]
    fn test_unsupported_language() {
        let result = generate_bindings(BindingLanguage::Python);
        assert!(result.is_err());
    }
}
