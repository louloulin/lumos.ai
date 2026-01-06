//! Configuration validation utilities
//!
//! Provides comprehensive validation for LumosAI configuration structures.

use crate::{Error, Result};
use std::collections::HashMap;
use validator::Validate;

/// Configuration validator with customizable rules and custom validation logic
#[derive(Debug, Clone)]
pub struct ConfigValidator {
    /// Strict validation mode (fail on warnings)
    strict_mode: bool,

    /// Environment-specific validation settings
    env_specific: HashMap<String, ValidationSettings>,
}

/// Validation rule trait for custom validation logic
pub trait ValidationRule: Send + Sync {
    /// Validate the given value and return validation result
    fn validate(&self, value: &serde_json::Value) -> ValidationResult;
    
    /// Get rule description
    fn description(&self) -> &'static str;
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    
    /// Validation messages (errors, warnings, info)
    pub messages: Vec<ValidationMessage>,
}

/// Validation message with severity level
#[derive(Debug, Clone)]
pub struct ValidationMessage {
    /// Message severity
    pub severity: ValidationSeverity,
    
    /// Message content
    pub message: String,
    
    /// Field path (e.g., "agents.defaults.temperature")
    pub field_path: Option<String>,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Environment-specific validation settings
#[derive(Debug, Clone)]
pub struct ValidationSettings {
    /// Enable strict validation for this environment
    pub strict: bool,
    
    /// Required fields for this environment
    pub required_fields: Vec<String>,
    
    /// Disabled validations for this environment
    pub disabled_rules: Vec<String>,
}

impl ConfigValidator {
    /// Create a new configuration validator with default rules
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            env_specific: Self::default_env_settings(),
        }
    }

    /// Create validator with strict mode enabled
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            env_specific: Self::default_env_settings(),
        }
    }

    /// Enable/disable strict mode
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Add environment-specific validation settings
    pub fn with_env_settings(mut self, env: String, settings: ValidationSettings) -> Self {
        self.env_specific.insert(env, settings);
        self
    }

    /// Validate a configuration object
    pub fn validate<T: Validate + Send + Sync>(&self, config: &T) -> Result<()> {
        // Perform standard validation using validator crate
        if let Err(e) = config.validate() {
            return Err(Error::Configuration(format!(
                "Configuration validation failed: {}",
                e
            )));
        }

        // TODO: Add custom validation rules
        // This would involve converting the config to JSON Value and applying custom rules

        Ok(())
    }

    /// Validate with custom rules for specific environment
    pub fn validate_with_env<T: Validate + Send + Sync>(
        &self,
        config: &T,
        environment: &str,
    ) -> Result<()> {
        // First, perform standard validation
        self.validate(config)?;

        // Get environment-specific settings
        let env_settings = self.env_specific.get(environment);

        // TODO: Apply environment-specific validation rules

        Ok(())
    }

    /// Validate LLM provider configuration
    pub fn validate_llm_config(
        &self,
        config: &crate::config::LlmConfig,
    ) -> Result<Vec<ValidationMessage>> {
        let mut messages = Vec::new();

        // Check if default provider is configured
        if config.default_provider.is_empty() {
            messages.push(ValidationMessage {
                severity: ValidationSeverity::Error,
                message: "Default LLM provider must be specified".to_string(),
                field_path: Some("llm.default_provider".to_string()),
            });
        }

        // Check if provider configurations exist
        if config.providers.is_empty() {
            messages.push(ValidationMessage {
                severity: ValidationSeverity::Warning,
                message: "No LLM provider configurations found".to_string(),
                field_path: Some("llm.providers".to_string()),
            });
        }

        // Check API key configuration
        if config.api_keys.env_vars.is_empty() && config.api_keys.encrypted.is_empty() {
            messages.push(ValidationMessage {
                severity: ValidationSeverity::Warning,
                message: "No API keys configured - agents may not function properly".to_string(),
                field_path: Some("llm.api_keys".to_string()),
            });
        }

        // Validate rate limiting if configured
        if let Some(rate_limit) = &config.rate_limit {
            if rate_limit.requests_per_minute == 0 {
                messages.push(ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "Rate limit requests_per_minute must be > 0".to_string(),
                    field_path: Some("llm.rate_limit.requests_per_minute".to_string()),
                });
            }

            if rate_limit.tokens_per_minute == 0 {
                messages.push(ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "Rate limit tokens_per_minute must be > 0".to_string(),
                    field_path: Some("llm.rate_limit.tokens_per_minute".to_string()),
                });
            }
        }

        Ok(messages)
    }

    /// Validate storage configuration
    pub fn validate_storage_config(
        &self,
        config: &crate::config::StorageConfig,
    ) -> Result<Vec<ValidationMessage>> {
        let mut messages = Vec::new();

        // Check vector storage configuration
        if config.vector.embedding_provider.is_empty() {
            messages.push(ValidationMessage {
                severity: ValidationSeverity::Warning,
                message: "No embedding provider configured for vector storage".to_string(),
                field_path: Some("storage.vector.embedding_provider".to_string()),
            });
        }

        if config.vector.dimension == 0 {
            messages.push(ValidationMessage {
                severity: ValidationSeverity::Warning,
                message: "Vector dimension is 0 - this may cause issues with embeddings".to_string(),
                field_path: Some("storage.vector.dimension".to_string()),
            });
        }

        // Check storage backends
        if config.backends.is_empty() {
            messages.push(ValidationMessage {
                severity: ValidationSeverity::Info,
                message: "No storage backends configured - using memory storage".to_string(),
                field_path: Some("storage.backends".to_string()),
            });
        }

        Ok(messages)
    }

    /// Validate security configuration
    pub fn validate_security_config(
        &self,
        config: &crate::config::SecurityConfig,
    ) -> Result<Vec<ValidationMessage>> {
        let mut messages = Vec::new();

        // Check authentication configuration
        if config.auth.enabled {
            if let Some(auth_type) = &config.auth.auth_type {
                match auth_type.as_str() {
                    "jwt" => {
                        if config.auth.jwt.secret.is_none() {
                            messages.push(ValidationMessage {
                                severity: ValidationSeverity::Error,
                                message: "JWT authentication enabled but no secret configured".to_string(),
                                field_path: Some("security.auth.jwt.secret".to_string()),
                            });
                        }
                    }
                    "oauth2" => {
                        if config.auth.oauth2.client_id.is_none() || config.auth.oauth2.client_secret.is_none()
                        {
                            messages.push(ValidationMessage {
                                severity: ValidationSeverity::Error,
                                message: "OAuth2 authentication enabled but client credentials missing".to_string(),
                                field_path: Some("security.auth.oauth2".to_string()),
                            });
                        }
                    }
                    _ => {
                        messages.push(ValidationMessage {
                            severity: ValidationSeverity::Warning,
                            message: format!("Unknown authentication type: {}", auth_type),
                            field_path: Some("security.auth.auth_type".to_string()),
                        });
                    }
                }
            } else {
                messages.push(ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "Authentication enabled but no auth type specified".to_string(),
                    field_path: Some("security.auth.auth_type".to_string()),
                });
            }
        }

        // Check encryption configuration
        if config.encryption.enabled {
            if config.encryption.key_management.source.is_empty() {
                messages.push(ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "Encryption enabled but no key source configured".to_string(),
                    field_path: Some("security.encryption.key_management.source".to_string()),
                });
            }
        }

        // Check audit configuration
        if config.audit.enabled {
            if config.audit.events.is_empty() {
                messages.push(ValidationMessage {
                    severity: ValidationSeverity::Warning,
                    message: "Audit logging enabled but no events specified".to_string(),
                    field_path: Some("security.audit.events".to_string()),
                });
            }
        }

        Ok(messages)
    }

    /// Validate monitoring configuration
    pub fn validate_monitoring_config(
        &self,
        config: &crate::config::MonitoringConfig,
    ) -> Result<Vec<ValidationMessage>> {
        let mut messages = Vec::new();

        if config.enabled {
            // Check metrics configuration
            if config.metrics.enabled {
                if config.metrics.backend.is_none() && config.metrics.endpoint.is_none() {
                    messages.push(ValidationMessage {
                        severity: ValidationSeverity::Warning,
                        message: "Metrics enabled but no backend or endpoint configured".to_string(),
                        field_path: Some("monitoring.metrics".to_string()),
                    });
                }
            }

            // Check tracing configuration
            if config.tracing.enabled {
                if config.tracing.backend.is_none() {
                    messages.push(ValidationMessage {
                        severity: ValidationSeverity::Info,
                        message: "Tracing enabled but no backend configured - using default".to_string(),
                        field_path: Some("monitoring.tracing.backend".to_string()),
                    });
                }

                if let Some(sampling_rate) = config.tracing.sampling_rate {
                    if sampling_rate < 0.0 || sampling_rate > 1.0 {
                        messages.push(ValidationMessage {
                            severity: ValidationSeverity::Error,
                            message: "Tracing sampling rate must be between 0.0 and 1.0".to_string(),
                            field_path: Some("monitoring.tracing.sampling_rate".to_string()),
                        });
                    }
                }
            }

            // Check health check configuration
            if config.health_check.enabled {
                if config.health_check.interval == 0 {
                    messages.push(ValidationMessage {
                        severity: ValidationSeverity::Error,
                        message: "Health check interval must be > 0".to_string(),
                        field_path: Some("monitoring.health_check.interval".to_string()),
                    });
                }

                if config.health_check.timeout == 0 {
                    messages.push(ValidationMessage {
                        severity: ValidationSeverity::Error,
                        message: "Health check timeout must be > 0".to_string(),
                        field_path: Some("monitoring.health_check.timeout".to_string()),
                    });
                }

                if config.health_check.timeout >= config.health_check.interval {
                    messages.push(ValidationMessage {
                        severity: ValidationSeverity::Warning,
                        message: "Health check timeout is >= interval - may cause false failures".to_string(),
                        field_path: Some("monitoring.health_check".to_string()),
                    });
                }
            }
        }

        Ok(messages)
    }



    /// Get default environment-specific settings
    fn default_env_settings() -> HashMap<String, ValidationSettings> {
        let mut settings = HashMap::new();

        // Production environment settings
        settings.insert(
            "production".to_string(),
            ValidationSettings {
                strict: true,
                required_fields: vec![
                    "llm.default_provider".to_string(),
                    "security.auth.enabled".to_string(),
                    "monitoring.enabled".to_string(),
                ],
                disabled_rules: vec![],
            },
        );

        // Development environment settings
        settings.insert(
            "development".to_string(),
            ValidationSettings {
                strict: false,
                required_fields: vec![],
                disabled_rules: vec!["api_key_presence".to_string()],
            },
        );

        // Testing environment settings
        settings.insert(
            "testing".to_string(),
            ValidationSettings {
                strict: false,
                required_fields: vec![],
                disabled_rules: vec![
                    "api_key_presence".to_string(),
                    "url_validation".to_string(),
                ],
            },
        );

        settings
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in validation rules

/// Temperature range validation rule
#[derive(Debug, Clone)]
struct TemperatureRangeRule;

impl ValidationRule for TemperatureRangeRule {
    fn validate(&self, value: &serde_json::Value) -> ValidationResult {
        if let Some(temp) = value.as_f64() {
            let is_valid = temp >= 0.0 && temp <= 2.0;
            let messages = if !is_valid {
                vec![ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "Temperature must be between 0.0 and 2.0".to_string(),
                    field_path: None,
                }]
            } else {
                vec![]
            };

            ValidationResult { is_valid, messages }
        } else {
            ValidationResult {
                is_valid: false,
                messages: vec![ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "Temperature must be a number".to_string(),
                    field_path: None,
                }],
            }
        }
    }

    fn description(&self) -> &'static str {
        "Validates that temperature is between 0.0 and 2.0"
    }
}

/// URL validation rule
#[derive(Debug, Clone)]
struct UrlValidationRule;

impl ValidationRule for UrlValidationRule {
    fn validate(&self, value: &serde_json::Value) -> ValidationResult {
        if let Some(url_str) = value.as_str() {
            // Simple URL validation using regex instead of url::Url to avoid dependency issues
            let url_regex = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
            let is_valid = url_regex.is_match(url_str);
            let messages = if !is_valid {
                vec![ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: format!("Invalid URL: {}", url_str),
                    field_path: None,
                }]
            } else {
                vec![]
            };

            ValidationResult { is_valid, messages }
        } else {
            ValidationResult {
                is_valid: false,
                messages: vec![ValidationMessage {
                    severity: ValidationSeverity::Error,
                    message: "URL must be a string".to_string(),
                    field_path: None,
                }],
            }
        }
    }

    fn description(&self) -> &'static str {
        "Validates that a string is a valid URL"
    }
}

/// API key presence validation rule (for production environments)
#[derive(Debug, Clone)]
struct ApiKeyPresenceRule;

impl ValidationRule for ApiKeyPresenceRule {
    fn validate(&self, _value: &serde_json::Value) -> ValidationResult {
        // This would check environment variables or other sources for API keys
        // For now, it's a placeholder that always passes
        ValidationResult {
            is_valid: true,
            messages: vec![],
        }
    }

    fn description(&self) -> &'static str {
        "Validates that required API keys are present in the environment"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validator_creation() {
        let validator = ConfigValidator::new();
        assert!(!validator.strict_mode);
    }

    #[test]
    fn test_strict_config_validator() {
        let validator = ConfigValidator::strict();
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_temperature_validation_rule() {
        let rule = TemperatureRangeRule;

        // Valid temperature
        let valid_temp = serde_json::json!(0.7);
        let result = rule.validate(&valid_temp);
        assert!(result.is_valid);
        assert!(result.messages.is_empty());

        // Invalid temperature
        let invalid_temp = serde_json::json!(3.0);
        let result = rule.validate(&invalid_temp);
        assert!(!result.is_valid);
        assert!(!result.messages.is_empty());

        // Non-numeric value
        let non_numeric = serde_json::json!("invalid");
        let result = rule.validate(&non_numeric);
        assert!(!result.is_valid);
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_url_validation_rule() {
        let rule = UrlValidationRule;

        // Valid URL
        let valid_url = serde_json::json!("https://example.com");
        let result = rule.validate(&valid_url);
        assert!(result.is_valid);

        // Invalid URL
        let invalid_url = serde_json::json!("not-a-url");
        let result = rule.validate(&invalid_url);
        assert!(!result.is_valid);

        // Non-string value
        let non_string = serde_json::json!(123);
        let result = rule.validate(&non_string);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_llm_config_validation() {
        let validator = ConfigValidator::new();
        
        // Empty LLM config should produce warnings
        let config = crate::config::LlmConfig::default();
        let messages = validator.validate_llm_config(&config).unwrap();
        
        assert!(!messages.is_empty());
        let warnings: Vec<_> = messages.iter()
            .filter(|m| matches!(m.severity, ValidationSeverity::Warning))
            .collect();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_security_config_validation() {
        let validator = ConfigValidator::new();
        
        // Auth enabled without configuration should produce errors
        let mut config = crate::config::SecurityConfig::default();
        config.auth.enabled = true;
        
        let messages = validator.validate_security_config(&config).unwrap();
        assert!(!messages.is_empty());
        
        let errors: Vec<_> = messages.iter()
            .filter(|m| matches!(m.severity, ValidationSeverity::Error))
            .collect();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_monitoring_config_validation() {
        let validator = ConfigValidator::new();
        
        let config = crate::config::MonitoringConfig::default();
        let messages = validator.validate_monitoring_config(&config).unwrap();
        assert!(messages.is_empty()); // Default config should be valid
    }
}