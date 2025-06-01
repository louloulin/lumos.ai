//! Advanced tool examples demonstrating Function Calling capabilities
//! 
//! This module showcases sophisticated tools that leverage the Function Calling
//! system for type-safe parameter handling and complex operations.

use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use async_trait::async_trait;

use lumosai_core::{
    base::{Base, BaseComponent, ComponentConfig},
    error::{Error, Result},
    logger::{Component, LogLevel},
    tool::{Tool, ToolSchema, ParameterSchema, ToolExecutionContext, ToolExecutionOptions},
};

/// File management tool with advanced Function Calling support
#[derive(Clone)]
pub struct FileManagerTool {
    base: BaseComponent,
    allowed_directories: Vec<String>,
}

impl FileManagerTool {
    pub fn new(allowed_directories: Vec<String>) -> Self {
        let component_config = ComponentConfig {
            name: Some("FileManagerTool".to_string()),
            component: Component::Tool,
            log_level: Some(LogLevel::Info),
        };

        Self {
            base: BaseComponent::new(component_config),
            allowed_directories,
        }
    }

    /// Check if a path is within allowed directories
    fn is_path_allowed(&self, path: &str) -> bool {
        let path = Path::new(path);
        for allowed_dir in &self.allowed_directories {
            if path.starts_with(allowed_dir) {
                return true;
            }
        }
        false
    }

    /// Execute file operation based on function call
    async fn execute_file_operation(&self, operation: &str, params: Value) -> Result<Value> {
        match operation {
            "read_file" => self.read_file(params).await,
            "write_file" => self.write_file(params).await,
            "list_directory" => self.list_directory(params).await,
            "create_directory" => self.create_directory(params).await,
            "delete_file" => self.delete_file(params).await,
            _ => Err(Error::InvalidInput(format!("Unknown operation: {}", operation)))
        }
    }

    async fn read_file(&self, params: Value) -> Result<Value> {
        let path: String = serde_json::from_value(params["path"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;

        if !self.is_path_allowed(&path) {
            return Err(Error::InvalidInput(format!("Access denied to path: {}", path)));
        }

        match fs::read_to_string(&path) {
            Ok(content) => Ok(json!({
                "success": true,
                "content": content,
                "path": path
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": path
            }))
        }
    }

    async fn write_file(&self, params: Value) -> Result<Value> {
        let path: String = serde_json::from_value(params["path"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;
        let content: String = serde_json::from_value(params["content"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'content' parameter".to_string()))?;

        if !self.is_path_allowed(&path) {
            return Err(Error::InvalidInput(format!("Access denied to path: {}", path)));
        }

        match fs::write(&path, &content) {
            Ok(_) => Ok(json!({
                "success": true,
                "message": "File written successfully",
                "path": path,
                "bytes_written": content.len()
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": path
            }))
        }
    }

    async fn list_directory(&self, params: Value) -> Result<Value> {
        let path: String = serde_json::from_value(params["path"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;

        if !self.is_path_allowed(&path) {
            return Err(Error::InvalidInput(format!("Access denied to path: {}", path)));
        }

        match fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                let mut directories = Vec::new();

                for entry in entries {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        if entry.path().is_dir() {
                            directories.push(file_name);
                        } else {
                            files.push(file_name);
                        }
                    }
                }

                Ok(json!({
                    "success": true,
                    "path": path,
                    "files": files,
                    "directories": directories,
                    "total_items": files.len() + directories.len()
                }))
            },
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": path
            }))
        }
    }

    async fn create_directory(&self, params: Value) -> Result<Value> {
        let path: String = serde_json::from_value(params["path"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;

        if !self.is_path_allowed(&path) {
            return Err(Error::InvalidInput(format!("Access denied to path: {}", path)));
        }

        match fs::create_dir_all(&path) {
            Ok(_) => Ok(json!({
                "success": true,
                "message": "Directory created successfully",
                "path": path
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": path
            }))
        }
    }

    async fn delete_file(&self, params: Value) -> Result<Value> {
        let path: String = serde_json::from_value(params["path"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'path' parameter".to_string()))?;

        if !self.is_path_allowed(&path) {
            return Err(Error::InvalidInput(format!("Access denied to path: {}", path)));
        }

        let result = if Path::new(&path).is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };

        match result {
            Ok(_) => Ok(json!({
                "success": true,
                "message": "File/directory deleted successfully",
                "path": path
            })),
            Err(e) => Ok(json!({
                "success": false,
                "error": e.to_string(),
                "path": path
            }))
        }
    }
}

impl std::fmt::Debug for FileManagerTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileManagerTool")
            .field("allowed_directories", &self.allowed_directories)
            .finish()
    }
}

impl Base for FileManagerTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> Component {
        self.base.component()
    }
    
    fn logger(&self) -> std::sync::Arc<dyn lumosai_core::logger::Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: std::sync::Arc<dyn lumosai_core::logger::Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<std::sync::Arc<dyn lumosai_core::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn lumosai_core::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for FileManagerTool {
    fn id(&self) -> &str {
        "file_manager"
    }

    fn description(&self) -> &str {
        "Advanced file management tool with support for reading, writing, listing, creating and deleting files and directories"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(vec![
            ParameterSchema {
                name: "operation".to_string(),
                r#type: "string".to_string(),
                description: "The file operation to perform: read_file, write_file, list_directory, create_directory, delete_file".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "path".to_string(),
                r#type: "string".to_string(),
                description: "The file or directory path".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "content".to_string(),
                r#type: "string".to_string(),
                description: "Content to write (only for write_file operation)".to_string(),
                required: false,
                properties: None,
                default: None,
            },
        ])
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions,
    ) -> Result<Value> {
        let operation: String = serde_json::from_value(params["operation"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'operation' parameter".to_string()))?;

        self.execute_file_operation(&operation, params).await
    }
}

/// Data analysis tool demonstrating complex parameter handling
#[derive(Clone)]
pub struct DataAnalysisTool {
    base: BaseComponent,
}

impl DataAnalysisTool {
    pub fn new() -> Self {
        let component_config = ComponentConfig {
            name: Some("DataAnalysisTool".to_string()),
            component: Component::Tool,
            log_level: Some(LogLevel::Info),
        };

        Self {
            base: BaseComponent::new(component_config),
        }
    }

    async fn analyze_data(&self, data: Vec<f64>, analysis_type: &str) -> Result<Value> {
        match analysis_type {
            "statistics" => self.calculate_statistics(data).await,
            "trend" => self.analyze_trend(data).await,
            "outliers" => self.detect_outliers(data).await,
            _ => Err(Error::InvalidInput(format!("Unknown analysis type: {}", analysis_type)))
        }
    }

    async fn calculate_statistics(&self, data: Vec<f64>) -> Result<Value> {
        if data.is_empty() {
            return Ok(json!({
                "success": false,
                "error": "No data provided"
            }));
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        Ok(json!({
            "success": true,
            "analysis_type": "statistics",
            "results": {
                "count": data.len(),
                "sum": sum,
                "mean": mean,
                "min": min,
                "max": max,
                "variance": variance,
                "standard_deviation": std_dev
            }
        }))
    }

    async fn analyze_trend(&self, data: Vec<f64>) -> Result<Value> {
        if data.len() < 2 {
            return Ok(json!({
                "success": false,
                "error": "Need at least 2 data points for trend analysis"
            }));
        }

        let n = data.len() as f64;
        let x_values: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();
        
        let sum_x: f64 = x_values.iter().sum();
        let sum_y: f64 = data.iter().sum();
        let sum_xy: f64 = x_values.iter().zip(data.iter()).map(|(x, y)| x * y).sum();
        let sum_x2: f64 = x_values.iter().map(|x| x * x).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;
        
        let trend_direction = if slope > 0.01 {
            "increasing"
        } else if slope < -0.01 {
            "decreasing"
        } else {
            "stable"
        };

        Ok(json!({
            "success": true,
            "analysis_type": "trend",
            "results": {
                "slope": slope,
                "intercept": intercept,
                "trend_direction": trend_direction,
                "data_points": data.len()
            }
        }))
    }

    async fn detect_outliers(&self, data: Vec<f64>) -> Result<Value> {
        if data.len() < 4 {
            return Ok(json!({
                "success": false,
                "error": "Need at least 4 data points for outlier detection"
            }));
        }

        let mut sorted_data = data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let q1_index = sorted_data.len() / 4;
        let q3_index = 3 * sorted_data.len() / 4;
        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;
        
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        let outliers: Vec<f64> = data.iter()
            .filter(|&&x| x < lower_bound || x > upper_bound)
            .cloned()
            .collect();

        Ok(json!({
            "success": true,
            "analysis_type": "outliers",
            "results": {
                "q1": q1,
                "q3": q3,
                "iqr": iqr,
                "lower_bound": lower_bound,
                "upper_bound": upper_bound,
                "outliers": outliers,
                "outlier_count": outliers.len()
            }
        }))
    }
}

impl std::fmt::Debug for DataAnalysisTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataAnalysisTool").finish()
    }
}

impl Base for DataAnalysisTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }
    
    fn component(&self) -> Component {
        self.base.component()
    }
    
    fn logger(&self) -> std::sync::Arc<dyn lumosai_core::logger::Logger> {
        self.base.logger()
    }
    
    fn set_logger(&mut self, logger: std::sync::Arc<dyn lumosai_core::logger::Logger>) {
        self.base.set_logger(logger);
    }
    
    fn telemetry(&self) -> Option<std::sync::Arc<dyn lumosai_core::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }
    
    fn set_telemetry(&mut self, telemetry: std::sync::Arc<dyn lumosai_core::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for DataAnalysisTool {
    fn id(&self) -> &str {
        "data_analysis"
    }

    fn description(&self) -> &str {
        "Advanced data analysis tool for statistical analysis, trend detection, and outlier identification"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(vec![
            ParameterSchema {
                name: "data".to_string(),
                r#type: "array".to_string(),
                description: "Array of numerical data points to analyze".to_string(),
                required: true,
                properties: None,
                default: None,
            },
            ParameterSchema {
                name: "analysis_type".to_string(),
                r#type: "string".to_string(),
                description: "Type of analysis: statistics, trend, or outliers".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ])
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions,
    ) -> Result<Value> {
        let data: Vec<f64> = serde_json::from_value(params["data"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'data' parameter".to_string()))?;
        
        let analysis_type: String = serde_json::from_value(params["analysis_type"].clone())
            .map_err(|_| Error::InvalidInput("Missing or invalid 'analysis_type' parameter".to_string()))?;

        self.analyze_data(data, &analysis_type).await
    }
}
