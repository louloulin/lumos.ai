//! Evaluation metrics system for agents

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::agent::types::RuntimeContext;
use crate::base::Base;
use crate::error::Result;
use crate::logger::{Component, Logger};
use crate::telemetry::TelemetrySink;

/// Evaluation metric result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Metric name
    pub metric_name: String,
    /// Score (typically 0.0 to 1.0)
    pub score: f64,
    /// Optional explanation
    pub explanation: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

/// Trait for evaluation metrics
#[async_trait]
pub trait EvaluationMetric: Base + Send + Sync {
    /// Evaluate the input/output pair and return a score
    async fn evaluate(
        &self,
        input: &str,
        output: &str,
        context: &RuntimeContext,
    ) -> Result<EvaluationResult>;
    
    /// Get the name of this metric
    fn metric_name(&self) -> &str;
    
    /// Get the description of this metric
    fn description(&self) -> &str {
        "No description provided"
    }
    
    /// Get the expected score range
    fn score_range(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

/// Simple relevance metric that checks if the output is relevant to the input
pub struct RelevanceMetric {
    /// Logger
    logger: Arc<dyn Logger>,
    /// Threshold for relevance
    threshold: f64,
}

impl RelevanceMetric {
    /// Create a new relevance metric
    pub fn new(logger: Arc<dyn Logger>, threshold: f64) -> Self {
        Self { logger, threshold }
    }
}

impl Base for RelevanceMetric {
    fn name(&self) -> Option<&str> {
        Some("RelevanceMetric")
    }

    fn component(&self) -> Component {
        Component::Agent
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl EvaluationMetric for RelevanceMetric {
    async fn evaluate(
        &self,
        input: &str,
        output: &str,
        _context: &RuntimeContext,
    ) -> Result<EvaluationResult> {
        // Simple relevance check based on keyword overlap
        let input_lower = input.to_lowercase();
        let output_lower = output.to_lowercase();

        let input_words: std::collections::HashSet<&str> = input_lower
            .split_whitespace()
            .collect();

        let output_words: std::collections::HashSet<&str> = output_lower
            .split_whitespace()
            .collect();
        
        let intersection_count = input_words.intersection(&output_words).count();
        let union_count = input_words.union(&output_words).count();
        
        let score = if union_count > 0 {
            intersection_count as f64 / union_count as f64
        } else {
            0.0
        };
        
        let explanation = if score >= self.threshold {
            Some(format!("Output is relevant (score: {:.3})", score))
        } else {
            Some(format!("Output may not be relevant (score: {:.3})", score))
        };
        
        Ok(EvaluationResult {
            metric_name: self.metric_name().to_string(),
            score,
            explanation,
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        })
    }
    
    fn metric_name(&self) -> &str {
        "relevance"
    }
    
    fn description(&self) -> &str {
        "Measures the relevance of the output to the input based on keyword overlap"
    }
}

/// Length metric that evaluates output length appropriateness
pub struct LengthMetric {
    /// Logger
    logger: Arc<dyn Logger>,
    /// Minimum expected length
    min_length: usize,
    /// Maximum expected length
    max_length: usize,
}

impl LengthMetric {
    /// Create a new length metric
    pub fn new(logger: Arc<dyn Logger>, min_length: usize, max_length: usize) -> Self {
        Self {
            logger,
            min_length,
            max_length,
        }
    }
}

impl Base for LengthMetric {
    fn name(&self) -> Option<&str> {
        Some("LengthMetric")
    }

    fn component(&self) -> Component {
        Component::Agent
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl EvaluationMetric for LengthMetric {
    async fn evaluate(
        &self,
        _input: &str,
        output: &str,
        _context: &RuntimeContext,
    ) -> Result<EvaluationResult> {
        let length = output.len();
        
        let score = if length < self.min_length {
            // Too short
            length as f64 / self.min_length as f64
        } else if length > self.max_length {
            // Too long
            self.max_length as f64 / length as f64
        } else {
            // Just right
            1.0
        };
        
        let explanation = Some(format!(
            "Output length: {} characters (expected: {}-{})",
            length, self.min_length, self.max_length
        ));
        
        let mut metadata = HashMap::new();
        metadata.insert("length".to_string(), serde_json::Value::Number(length.into()));
        metadata.insert("min_length".to_string(), serde_json::Value::Number(self.min_length.into()));
        metadata.insert("max_length".to_string(), serde_json::Value::Number(self.max_length.into()));
        
        Ok(EvaluationResult {
            metric_name: self.metric_name().to_string(),
            score,
            explanation,
            metadata,
            timestamp: std::time::SystemTime::now(),
        })
    }
    
    fn metric_name(&self) -> &str {
        "length"
    }
    
    fn description(&self) -> &str {
        "Evaluates whether the output length is within expected bounds"
    }
}

/// Composite metric that combines multiple metrics
pub struct CompositeMetric {
    /// Logger
    logger: Arc<dyn Logger>,
    /// List of metrics with their weights
    metrics: Vec<(Box<dyn EvaluationMetric>, f64)>,
    /// Name of this composite metric
    name: String,
}

impl CompositeMetric {
    /// Create a new composite metric
    pub fn new(name: String, logger: Arc<dyn Logger>) -> Self {
        Self {
            logger,
            metrics: Vec::new(),
            name,
        }
    }
    
    /// Add a metric with a weight
    pub fn add_metric(&mut self, metric: Box<dyn EvaluationMetric>, weight: f64) {
        self.metrics.push((metric, weight));
    }
}

impl Base for CompositeMetric {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn component(&self) -> Component {
        Component::Agent
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.logger = logger;
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        None
    }

    fn set_telemetry(&mut self, _telemetry: Arc<dyn TelemetrySink>) {
        // No-op for now
    }
}

#[async_trait]
impl EvaluationMetric for CompositeMetric {
    async fn evaluate(
        &self,
        input: &str,
        output: &str,
        context: &RuntimeContext,
    ) -> Result<EvaluationResult> {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        let mut metadata = HashMap::new();
        
        for (metric, weight) in &self.metrics {
            let result = metric.evaluate(input, output, context).await?;
            total_score += result.score * weight;
            total_weight += weight;
            
            // Add individual metric results to metadata
            metadata.insert(
                format!("{}_score", result.metric_name),
                serde_json::Value::Number(serde_json::Number::from_f64(result.score).unwrap_or_else(|| serde_json::Number::from(0))),
            );
            metadata.insert(
                format!("{}_weight", result.metric_name),
                serde_json::Value::Number(serde_json::Number::from_f64(*weight).unwrap_or_else(|| serde_json::Number::from(0))),
            );
        }
        
        let final_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };
        
        Ok(EvaluationResult {
            metric_name: self.metric_name().to_string(),
            score: final_score,
            explanation: Some(format!("Composite score from {} metrics", self.metrics.len())),
            metadata,
            timestamp: std::time::SystemTime::now(),
        })
    }
    
    fn metric_name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "Composite metric that combines multiple evaluation metrics"
    }
}
