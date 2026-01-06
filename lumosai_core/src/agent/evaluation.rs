//! Evaluation metrics system for agents

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::agent::types::RuntimeContext;
use crate::base::Base;
use crate::compat::Component;
use crate::error::Result;
use crate::logger::Logger;
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

        let input_words: std::collections::HashSet<&str> = input_lower.split_whitespace().collect();

        let output_words: std::collections::HashSet<&str> =
            output_lower.split_whitespace().collect();

        let intersection_count = input_words.intersection(&output_words).count();
        let union_count = input_words.union(&output_words).count();

        let score = if union_count > 0 {
            intersection_count as f64 / union_count as f64
        } else {
            0.0
        };

        let explanation = if score >= self.threshold {
            Some(format!("Output is relevant (score: {score:.3})"))
        } else {
            Some(format!("Output may not be relevant (score: {score:.3})"))
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
        metadata.insert(
            "length".to_string(),
            serde_json::Value::Number(length.into()),
        );
        metadata.insert(
            "min_length".to_string(),
            serde_json::Value::Number(self.min_length.into()),
        );
        metadata.insert(
            "max_length".to_string(),
            serde_json::Value::Number(self.max_length.into()),
        );

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
                serde_json::Value::Number(
                    serde_json::Number::from_f64(result.score)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
            metadata.insert(
                format!("{}_weight", result.metric_name),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(*weight)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
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
            explanation: Some(format!(
                "Composite score from {} metrics",
                self.metrics.len()
            )),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::NoopLogger;

    fn create_test_logger() -> Arc<dyn Logger> {
        Arc::new(NoopLogger::default())
    }

    fn create_test_context() -> RuntimeContext {
        RuntimeContext::default()
    }

    #[tokio::test]
    async fn test_relevance_metric_high_relevance() {
        let logger = create_test_logger();
        let metric = RelevanceMetric::new(logger, 0.5);
        let context = create_test_context();

        let result = metric
            .evaluate("What is Rust?", "Rust is a programming language", &context)
            .await
            .unwrap();

        assert_eq!(result.metric_name, "relevance");
        assert!(result.score > 0.0);
        assert!(result.explanation.is_some());
    }

    #[tokio::test]
    async fn test_relevance_metric_low_relevance() {
        let logger = create_test_logger();
        let metric = RelevanceMetric::new(logger, 0.5);
        let context = create_test_context();

        let result = metric
            .evaluate("What is Rust?", "The weather is nice today", &context)
            .await
            .unwrap();

        assert_eq!(result.metric_name, "relevance");
        assert!(result.score < 0.5);
    }

    #[tokio::test]
    async fn test_relevance_metric_empty_input() {
        let logger = create_test_logger();
        let metric = RelevanceMetric::new(logger, 0.5);
        let context = create_test_context();

        let result = metric.evaluate("", "Some output", &context).await.unwrap();

        assert_eq!(result.metric_name, "relevance");
        assert_eq!(result.score, 0.0);
    }

    #[tokio::test]
    async fn test_length_metric_appropriate_length() {
        let logger = create_test_logger();
        let metric = LengthMetric::new(logger, 10, 100);
        let context = create_test_context();

        let result = metric
            .evaluate("Input", "This is a medium length output", &context)
            .await
            .unwrap();

        assert_eq!(result.metric_name, "length");
        assert_eq!(result.score, 1.0);
        assert!(result.explanation.is_some());
        assert!(result.metadata.contains_key("length"));
    }

    #[tokio::test]
    async fn test_length_metric_too_short() {
        let logger = create_test_logger();
        let metric = LengthMetric::new(logger, 10, 100);
        let context = create_test_context();

        let result = metric.evaluate("Input", "Short", &context).await.unwrap();

        assert_eq!(result.metric_name, "length");
        assert!(result.score < 1.0);
    }

    #[tokio::test]
    async fn test_length_metric_too_long() {
        let logger = create_test_logger();
        let metric = LengthMetric::new(logger, 10, 50);
        let context = create_test_context();

        let long_output = "a".repeat(100);
        let result = metric
            .evaluate("Input", &long_output, &context)
            .await
            .unwrap();

        assert_eq!(result.metric_name, "length");
        assert!(result.score < 1.0);
    }

    #[tokio::test]
    async fn test_composite_metric() {
        let logger = create_test_logger();
        let mut composite = CompositeMetric::new("test_composite".to_string(), logger.clone());

        let relevance = Box::new(RelevanceMetric::new(logger.clone(), 0.5));
        let length = Box::new(LengthMetric::new(logger, 10, 100));

        composite.add_metric(relevance, 0.6);
        composite.add_metric(length, 0.4);

        let context = create_test_context();
        let result = composite
            .evaluate("What is Rust?", "Rust is a programming language", &context)
            .await
            .unwrap();

        assert_eq!(result.metric_name, "test_composite");
        assert!(result.score >= 0.0 && result.score <= 1.0);
        assert!(result.metadata.contains_key("relevance_score"));
        assert!(result.metadata.contains_key("length_score"));
    }

    #[tokio::test]
    async fn test_composite_metric_empty() {
        let logger = create_test_logger();
        let composite = CompositeMetric::new("empty_composite".to_string(), logger);
        let context = create_test_context();

        let result = composite
            .evaluate("Input", "Output", &context)
            .await
            .unwrap();

        assert_eq!(result.metric_name, "empty_composite");
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_evaluation_result_serialization() {
        let result = EvaluationResult {
            metric_name: "test_metric".to_string(),
            score: 0.85,
            explanation: Some("Test explanation".to_string()),
            metadata: {
                let mut m = HashMap::new();
                m.insert("key".to_string(), serde_json::json!("value"));
                m
            },
            timestamp: std::time::SystemTime::now(),
        };

        // Test serialization
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test_metric"));
        assert!(json.contains("0.85"));

        // Test deserialization
        let deserialized: EvaluationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.metric_name, "test_metric");
        assert_eq!(deserialized.score, 0.85);
    }

    #[test]
    fn test_metric_names() {
        let logger = create_test_logger();
        let relevance = RelevanceMetric::new(logger.clone(), 0.5);
        let length = LengthMetric::new(logger.clone(), 10, 100);
        let composite = CompositeMetric::new("test".to_string(), logger);

        assert_eq!(relevance.metric_name(), "relevance");
        assert_eq!(length.metric_name(), "length");
        assert_eq!(composite.metric_name(), "test");
    }

    #[test]
    fn test_metric_descriptions() {
        let logger = create_test_logger();
        let relevance = RelevanceMetric::new(logger.clone(), 0.5);
        let length = LengthMetric::new(logger.clone(), 10, 100);
        let composite = CompositeMetric::new("test".to_string(), logger);

        assert!(!relevance.description().is_empty());
        assert!(!length.description().is_empty());
        assert!(!composite.description().is_empty());
    }

    #[test]
    fn test_score_ranges() {
        let logger = create_test_logger();
        let relevance = RelevanceMetric::new(logger.clone(), 0.5);
        let length = LengthMetric::new(logger.clone(), 10, 100);
        let composite = CompositeMetric::new("test".to_string(), logger);

        assert_eq!(relevance.score_range(), (0.0, 1.0));
        assert_eq!(length.score_range(), (0.0, 1.0));
        assert_eq!(composite.score_range(), (0.0, 1.0));
    }
}
