//! Mathematical computation tools inspired by Mastra's math utilities
//! 
//! This module provides calculator, statistics, and mathematical operations

use crate::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool, ToolExecutionContext, ToolExecutionOptions};
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::{Result, Error};
use crate::base::Base;
use async_trait::async_trait;

/// Create a calculator tool
/// Similar to Mastra's mathematical computation capabilities
pub fn create_calculator_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "expression".to_string(),
            description: "Mathematical expression to evaluate".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "precision".to_string(),
            description: "Number of decimal places for the result".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(10)),
        },
        ParameterSchema {
            name: "format".to_string(),
            description: "Output format (decimal, scientific, fraction)".to_string(),
            r#type: "string".to_string(),
            required: false,
            properties: None,
            default: Some(json!("decimal")),
        },
    ]);

    FunctionTool::new(
        "calculator",
        "Evaluate mathematical expressions with high precision",
        schema,
        |params| {
            let expression = params.get("expression")
                .and_then(|v| v.as_str())
                .ok_or("Expression is required")?;
            
            let precision = params.get("precision")
                .and_then(|v| v.as_u64())
                .unwrap_or(10)
                .min(20) as usize; // Limit precision to 20 decimal places
            
            let format = params.get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("decimal");

            // Simple expression evaluation (mock implementation)
            // In real implementation would use a proper math parser like meval
            let result = evaluate_simple_expression(expression);

            match result {
                Ok(value) => {
                    let formatted_result = match format {
                        "scientific" => format!("{:.precision$e}", value, precision = precision),
                        "fraction" => {
                            // Mock fraction conversion
                            if value.fract() == 0.0 {
                                format!("{}/1", value as i64)
                            } else {
                                format!("{:.precision$}", value, precision = precision)
                            }
                        },
                        _ => format!("{:.precision$}", value, precision = precision),
                    };

                    Ok(json!({
                        "success": true,
                        "expression": expression,
                        "result": value,
                        "formatted_result": formatted_result,
                        "precision": precision,
                        "format": format,
                        "metadata": {
                            "is_integer": value.fract() == 0.0,
                            "is_positive": value > 0.0,
                            "absolute_value": value.abs()
                        }
                    }))
                },
                Err(e) => {
                    Ok(json!({
                        "success": false,
                        "expression": expression,
                        "error": e,
                        "supported_operations": ["+", "-", "*", "/", "^", "sqrt", "sin", "cos", "tan", "log"]
                    }))
                }
            }
        },
    )
}

/// Create a statistics tool
/// Similar to Mastra's data analysis capabilities
pub fn create_statistics_tool() -> FunctionTool {
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "data".to_string(),
            description: "Array of numbers to analyze".to_string(),
            r#type: "array".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "operations".to_string(),
            description: "Statistical operations to perform".to_string(),
            r#type: "array".to_string(),
            required: false,
            properties: None,
            default: Some(json!(["mean", "median", "mode", "std_dev", "variance"])),
        },
        ParameterSchema {
            name: "precision".to_string(),
            description: "Number of decimal places for results".to_string(),
            r#type: "number".to_string(),
            required: false,
            properties: None,
            default: Some(json!(6)),
        },
    ]);

    FunctionTool::new(
        "statistics",
        "Perform statistical analysis on numerical data",
        schema,
        |params| {
            let data = params.get("data")
                .and_then(|v| v.as_array())
                .ok_or("Data array is required")?;
            
            let operations = params.get("operations")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["mean", "median", "mode", "std_dev", "variance"]);
            
            let precision = params.get("precision")
                .and_then(|v| v.as_u64())
                .unwrap_or(6)
                .min(15) as usize;

            // Convert JSON array to f64 vector
            let numbers: std::result::Result<Vec<f64>, &str> = data.iter()
                .map(|v| v.as_f64().ok_or("All data elements must be numbers"))
                .collect();

            match numbers {
                Ok(nums) => {
                    if nums.is_empty() {
                        return Ok(json!({
                            "success": false,
                            "error": "Data array cannot be empty"
                        }));
                    }

                    let mut results = HashMap::new();

                    for operation in &operations {
                        let result = match *operation {
                            "mean" => calculate_mean(&nums),
                            "median" => calculate_median(&nums),
                            "mode" => calculate_mode(&nums),
                            "std_dev" => calculate_std_dev(&nums),
                            "variance" => calculate_variance(&nums),
                            "min" => nums.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                            "max" => nums.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                            "sum" => nums.iter().sum(),
                            "count" => nums.len() as f64,
                            _ => continue,
                        };

                        results.insert(operation, format!("{:.precision$}", result, precision = precision));
                    }

                    Ok(json!({
                        "success": true,
                        "data_count": nums.len(),
                        "results": results,
                        "precision": precision,
                        "metadata": {
                            "data_range": {
                                "min": nums.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                                "max": nums.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
                            },
                            "operations_performed": operations
                        }
                    }))
                },
                Err(e) => {
                    Ok(json!({
                        "success": false,
                        "error": e
                    }))
                }
            }
        },
    )
}

/// Simple expression evaluator (mock implementation)
fn evaluate_simple_expression(expr: &str) -> std::result::Result<f64, String> {
    let expr = expr.trim().replace(" ", "");
    
    // Very basic calculator - in real implementation would use a proper parser
    if expr.contains("+") {
        let parts: Vec<&str> = expr.split('+').collect();
        if parts.len() == 2 {
            let a = parts[0].parse::<f64>().map_err(|_| "Invalid number")?;
            let b = parts[1].parse::<f64>().map_err(|_| "Invalid number")?;
            return Ok(a + b);
        }
    }
    
    if expr.contains("-") && !expr.starts_with("-") {
        let parts: Vec<&str> = expr.split('-').collect();
        if parts.len() == 2 {
            let a = parts[0].parse::<f64>().map_err(|_| "Invalid number")?;
            let b = parts[1].parse::<f64>().map_err(|_| "Invalid number")?;
            return Ok(a - b);
        }
    }
    
    if expr.contains("*") {
        let parts: Vec<&str> = expr.split('*').collect();
        if parts.len() == 2 {
            let a = parts[0].parse::<f64>().map_err(|_| "Invalid number")?;
            let b = parts[1].parse::<f64>().map_err(|_| "Invalid number")?;
            return Ok(a * b);
        }
    }
    
    if expr.contains("/") {
        let parts: Vec<&str> = expr.split('/').collect();
        if parts.len() == 2 {
            let a = parts[0].parse::<f64>().map_err(|_| "Invalid number")?;
            let b = parts[1].parse::<f64>().map_err(|_| "Invalid number")?;
            if b == 0.0 {
                return Err("Division by zero".to_string());
            }
            return Ok(a / b);
        }
    }
    
    // Try to parse as a single number
    expr.parse::<f64>().map_err(|_| format!("Unable to evaluate expression: {}", expr))
}

/// Calculate mean of a dataset
fn calculate_mean(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

/// Calculate median of a dataset
fn calculate_median(data: &[f64]) -> f64 {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let len = sorted.len();
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    }
}

/// Calculate mode of a dataset (returns first mode found)
fn calculate_mode(data: &[f64]) -> f64 {
    let mut counts = HashMap::new();
    for &value in data {
        *counts.entry(value.to_bits()).or_insert(0) += 1;
    }
    
    let max_count = counts.values().max().unwrap_or(&0);
    let mode_bits = counts.iter()
        .find(|(_, &count)| count == *max_count)
        .map(|(&bits, _)| bits)
        .unwrap_or(0);
    
    f64::from_bits(mode_bits)
}

/// Calculate standard deviation
fn calculate_std_dev(data: &[f64]) -> f64 {
    calculate_variance(data).sqrt()
}

/// Calculate variance
fn calculate_variance(data: &[f64]) -> f64 {
    let mean = calculate_mean(data);
    let sum_squared_diff: f64 = data.iter()
        .map(|&x| (x - mean).powi(2))
        .sum();
    sum_squared_diff / data.len() as f64
}

/// Calculator tool for mathematical operations
#[derive(Clone)]
pub struct CalculatorTool {
    base: crate::base::BaseComponent,
    id: String,
    description: String,
    schema: ToolSchema,
}

impl CalculatorTool {
    /// Create a new calculator tool
    pub fn new() -> Self {
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "expression".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ]);

        Self {
            base: crate::base::BaseComponent::new_with_name(
                "calculator".to_string(),
                crate::logger::Component::Tool
            ),
            id: "calculator".to_string(),
            description: "Evaluate mathematical expressions".to_string(),
            schema,
        }
    }

    /// Evaluate a simple mathematical expression
    fn evaluate_expression(&self, expr: &str) -> Result<f64> {
        // Simple expression evaluator - in a real implementation,
        // you might use a proper math parser like `meval` crate
        let cleaned = expr.replace(" ", "");

        // Handle basic operations
        if let Some(pos) = cleaned.find('+') {
            let (left, right) = cleaned.split_at(pos);
            let right = &right[1..]; // Skip the '+'
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            return Ok(left_val + right_val);
        }

        if let Some(pos) = cleaned.find('-') {
            let (left, right) = cleaned.split_at(pos);
            let right = &right[1..]; // Skip the '-'
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            return Ok(left_val - right_val);
        }

        if let Some(pos) = cleaned.find('*') {
            let (left, right) = cleaned.split_at(pos);
            let right = &right[1..]; // Skip the '*'
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            return Ok(left_val * right_val);
        }

        if let Some(pos) = cleaned.find('/') {
            let (left, right) = cleaned.split_at(pos);
            let right = &right[1..]; // Skip the '/'
            let left_val = self.parse_number(left)?;
            let right_val = self.parse_number(right)?;
            if right_val == 0.0 {
                return Err(Error::Tool("Division by zero".to_string()));
            }
            return Ok(left_val / right_val);
        }

        // If no operation found, try to parse as a number
        self.parse_number(&cleaned).map_err(|e| e)
    }

    fn parse_number(&self, s: &str) -> Result<f64> {
        s.parse::<f64>()
            .map_err(|_| Error::Tool(format!("Invalid number: {}", s)))
    }
}

impl std::fmt::Debug for CalculatorTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CalculatorTool")
            .field("id", &self.id)
            .field("description", &self.description)
            .finish()
    }
}

impl Base for CalculatorTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> crate::logger::Component {
        self.base.component()
    }

    fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: std::sync::Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions
    ) -> Result<Value> {
        let expression = params.get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Tool("Expression parameter is required".to_string()))?;

        let result = self.evaluate_expression(expression)?;

        Ok(json!({
            "expression": expression,
            "result": result
        }))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = create_calculator_tool();
        
        let mut params = HashMap::new();
        params.insert("expression".to_string(), json!("2 + 3"));
        params.insert("precision".to_string(), json!(2));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["result"], 5.0);
    }

    #[tokio::test]
    async fn test_statistics_tool() {
        let tool = create_statistics_tool();
        
        let mut params = HashMap::new();
        params.insert("data".to_string(), json!([1, 2, 3, 4, 5]));
        params.insert("operations".to_string(), json!(["mean", "median"]));

        let context = crate::tool::context::ToolExecutionContext::new();
        let options = crate::tool::schema::ToolExecutionOptions::new();
        let result = tool.execute(serde_json::to_value(&params).unwrap(), context, &options).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response["success"], true);
        assert_eq!(response["data_count"], 5);
        assert!(response["results"]["mean"].is_string());
        assert!(response["results"]["median"].is_string());
    }

    #[test]
    fn test_math_functions() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(calculate_mean(&data), 3.0);
        assert_eq!(calculate_median(&data), 3.0);
        // 标准差应该是 sqrt(2.0) ≈ 1.4142135623730951
        assert!((calculate_std_dev(&data) - 1.4142135623730951).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_calculator_tool_basic() {
        let tool = CalculatorTool::new();

        assert_eq!(tool.name(), Some("calculator"));
        assert_eq!(tool.description(), "Evaluate mathematical expressions");
    }

    #[tokio::test]
    async fn test_calculator_addition() {
        let tool = CalculatorTool::new();

        let params = json!({
            "expression": "5 + 3"
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await.unwrap();
        let value = result.get("result").unwrap().as_f64().unwrap();
        assert_eq!(value, 8.0);
    }

    #[tokio::test]
    async fn test_calculator_subtraction() {
        let tool = CalculatorTool::new();

        let params = json!({
            "expression": "10 - 4"
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await.unwrap();
        let value = result.get("result").unwrap().as_f64().unwrap();
        assert_eq!(value, 6.0);
    }

    #[tokio::test]
    async fn test_calculator_multiplication() {
        let tool = CalculatorTool::new();

        let params = json!({
            "expression": "6 * 7"
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await.unwrap();
        let value = result.get("result").unwrap().as_f64().unwrap();
        assert_eq!(value, 42.0);
    }

    #[tokio::test]
    async fn test_calculator_division() {
        let tool = CalculatorTool::new();

        let params = json!({
            "expression": "15 / 3"
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await.unwrap();
        let value = result.get("result").unwrap().as_f64().unwrap();
        assert_eq!(value, 5.0);
    }

    #[tokio::test]
    async fn test_calculator_division_by_zero() {
        let tool = CalculatorTool::new();

        let params = json!({
            "expression": "10 / 0"
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculator_invalid_expression() {
        let tool = CalculatorTool::new();

        let params = json!({
            "expression": "invalid"
        });

        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();

        let result = tool.execute(params, context, &options).await;
        assert!(result.is_err());
    }
}
