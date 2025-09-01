use crate::error::{Error, Result};
use crate::agent::config::AgentConfig;
use serde_json::Value;
use std::collections::HashMap;

/// 配置验证器，用于验证Agent配置的有效性
pub struct ConfigValidator {
    /// 必需字段列表
    required_fields: Vec<String>,
    /// 字段验证规则
    validation_rules: HashMap<String, Box<dyn Fn(&Value) -> Result<()> + Send + Sync>>,
}

impl ConfigValidator {
    /// 创建新的配置验证器
    pub fn new() -> Self {
        let mut validator = Self {
            required_fields: vec![
                "name".to_string(),
                "model".to_string(),
            ],
            validation_rules: HashMap::new(),
        };
        
        validator.setup_default_rules();
        validator
    }
    
    /// 设置默认验证规则
    fn setup_default_rules(&mut self) {
        // 验证名称不为空
        self.add_rule("name", Box::new(|value: &Value| {
            if let Some(name) = value.as_str() {
                if name.trim().is_empty() {
                    return Err(Error::Validation("Agent name cannot be empty".to_string()));
                }
                if name.len() > 100 {
                    return Err(Error::Validation("Agent name too long (max 100 characters)".to_string()));
                }
                Ok(())
            } else {
                Err(Error::Validation("Agent name must be a string".to_string()))
            }
        }));
        
        // 验证模型配置
        self.add_rule("model", Box::new(|value: &Value| {
            if let Some(model) = value.as_str() {
                if model.trim().is_empty() {
                    return Err(Error::Validation("Model name cannot be empty".to_string()));
                }
                // 验证支持的模型格式
                let valid_patterns = [
                    "gpt-", "claude-", "qwen", "gemini", "llama", "mistral", "yi-"
                ];
                if !valid_patterns.iter().any(|pattern| model.contains(pattern)) {
                    return Err(Error::Validation(format!(
                        "Unsupported model '{}'. Supported models: GPT, Claude, Qwen, Gemini, Llama, Mistral, Yi", 
                        model
                    )));
                }
                Ok(())
            } else {
                Err(Error::Validation("Model must be a string".to_string()))
            }
        }));
        
        // 验证温度参数
        self.add_rule("temperature", Box::new(|value: &Value| {
            if let Some(temp) = value.as_f64() {
                if temp < 0.0 || temp > 2.0 {
                    return Err(Error::Validation("Temperature must be between 0.0 and 2.0".to_string()));
                }
                Ok(())
            } else {
                Err(Error::Validation("Temperature must be a number".to_string()))
            }
        }));
        
        // 验证最大令牌数
        self.add_rule("max_tokens", Box::new(|value: &Value| {
            if let Some(tokens) = value.as_u64() {
                if tokens == 0 || tokens > 100000 {
                    return Err(Error::Validation("Max tokens must be between 1 and 100000".to_string()));
                }
                Ok(())
            } else {
                Err(Error::Validation("Max tokens must be a positive integer".to_string()))
            }
        }));
    }
    
    /// 添加验证规则
    pub fn add_rule<F>(&mut self, field: &str, rule: F)
    where
        F: Fn(&Value) -> Result<()> + Send + Sync + 'static,
    {
        self.validation_rules.insert(field.to_string(), Box::new(rule));
    }
    
    /// 添加必需字段
    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
    }
    
    /// 验证Agent配置
    pub fn validate_agent_config(&self, config: &AgentConfig) -> Result<()> {
        // 转换为JSON进行验证
        let config_json = serde_json::to_value(config)
            .map_err(|e| Error::Validation(format!("Failed to serialize config: {}", e)))?;
        
        self.validate_json(&config_json)
    }
    
    /// 验证JSON配置
    pub fn validate_json(&self, config: &Value) -> Result<()> {
        let config_obj = config.as_object()
            .ok_or_else(|| Error::Validation("Configuration must be an object".to_string()))?;
        
        // 检查必需字段
        for required_field in &self.required_fields {
            if !config_obj.contains_key(required_field) {
                return Err(Error::Validation(format!("Missing required field: {}", required_field)));
            }
        }
        
        // 应用验证规则
        for (field, value) in config_obj {
            if let Some(rule) = self.validation_rules.get(field) {
                rule(value).map_err(|e| {
                    Error::Validation(format!("Validation failed for field '{}': {}", field, e))
                })?;
            }
        }
        
        Ok(())
    }
    
    /// 验证配置并返回详细的验证报告
    pub fn validate_with_report(&self, config: &Value) -> ValidationReport {
        let mut report = ValidationReport::new();
        
        let config_obj = match config.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error("Configuration must be an object".to_string());
                return report;
            }
        };
        
        // 检查必需字段
        for required_field in &self.required_fields {
            if !config_obj.contains_key(required_field) {
                report.add_error(format!("Missing required field: {}", required_field));
            }
        }
        
        // 应用验证规则
        for (field, value) in config_obj {
            if let Some(rule) = self.validation_rules.get(field) {
                if let Err(e) = rule(value) {
                    report.add_error(format!("Field '{}': {}", field, e));
                } else {
                    report.add_success(format!("Field '{}' is valid", field));
                }
            } else {
                report.add_warning(format!("Unknown field '{}' (will be ignored)", field));
            }
        }
        
        report
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub successes: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            successes: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn add_success(&mut self, success: String) {
        self.successes.push(success);
    }
    
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn summary(&self) -> String {
        format!(
            "Validation Report: {} errors, {} warnings, {} successes",
            self.errors.len(),
            self.warnings.len(),
            self.successes.len()
        )
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_valid_config() {
        let validator = ConfigValidator::new();
        let config = json!({
            "name": "test-agent",
            "model": "gpt-4",
            "temperature": 0.7,
            "max_tokens": 1000
        });
        
        assert!(validator.validate_json(&config).is_ok());
    }
    
    #[test]
    fn test_missing_required_field() {
        let validator = ConfigValidator::new();
        let config = json!({
            "model": "gpt-4"
            // missing "name"
        });
        
        let result = validator.validate_json(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing required field: name"));
    }
    
    #[test]
    fn test_invalid_temperature() {
        let validator = ConfigValidator::new();
        let config = json!({
            "name": "test-agent",
            "model": "gpt-4",
            "temperature": 3.0  // invalid: > 2.0
        });
        
        let result = validator.validate_json(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Temperature must be between 0.0 and 2.0"));
    }
    
    #[test]
    fn test_validation_report() {
        let validator = ConfigValidator::new();
        let config = json!({
            "name": "test-agent",
            "model": "gpt-4",
            "temperature": 3.0,  // invalid
            "unknown_field": "value"  // unknown
        });
        
        let report = validator.validate_with_report(&config);
        assert!(!report.is_valid());
        assert!(!report.errors.is_empty());
        assert!(!report.warnings.is_empty());
    }
}
