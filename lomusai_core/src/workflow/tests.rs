#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::workflow::step::BasicStep;
    use serde_json::json;
    use tokio::test;

    #[test]
    async fn test_simple_workflow() {
        // 创建步骤
        let step1 = BasicStep::create_simple(
            "step1".to_string(),
            "第一步".to_string(),
            |input| {
                println!("执行步骤1");
                Ok(json!({ "result": "Step 1 output" }))
            },
        );

        let step2 = BasicStep::create_simple(
            "step2".to_string(),
            "第二步".to_string(),
            |input| {
                println!("执行步骤2");
                Ok(json!({ "result": "Step 2 output" }))
            },
        );

        // 创建工作流
        let workflow = Workflow::new("workflow1".to_string(), "测试工作流".to_string())
            .add_step(step1, None, None)
            .add_step(step2, None, None)
            .add_dependency("step1", "step2")
            .build();

        // 执行工作流
        let instance = workflow.create_run(json!({}));
        let result = instance.run().await.unwrap();

        // 验证结果
        assert_eq!(result.results.len(), 2);
        assert!(matches!(
            result.results.get("step1").unwrap(),
            StepResult::Success { .. }
        ));
        assert!(matches!(
            result.results.get("step2").unwrap(),
            StepResult::Success { .. }
        ));
    }

    #[test]
    async fn test_workflow_with_condition() {
        // 创建步骤
        let step1 = BasicStep::create_simple(
            "step1".to_string(),
            "条件步骤".to_string(),
            |_| Ok(json!({ "status": "success" })),
        );

        let step2 = BasicStep::create_simple(
            "step2".to_string(),
            "成功路径".to_string(),
            |_| Ok(json!({ "result": "Success path" })),
        );

        let step3 = BasicStep::create_simple(
            "step3".to_string(),
            "失败路径".to_string(),
            |_| Ok(json!({ "result": "Failure path" })),
        );

        // 创建条件
        let success_condition = StepCondition::Reference {
            step_id: "step1".to_string(),
            path: "status".to_string(),
            query: json!({ "$eq": "success" }),
        };

        let failure_condition = StepCondition::Reference {
            step_id: "step1".to_string(),
            path: "status".to_string(),
            query: json!({ "$eq": "failed" }),
        };

        // 创建工作流
        let workflow = Workflow::new("conditional_workflow".to_string(), "条件工作流".to_string())
            .add_step(step1, None, None)
            .add_step(step2, None, Some(success_condition))
            .add_step(step3, None, Some(failure_condition))
            .add_dependency("step1", "step2")
            .add_dependency("step1", "step3")
            .build();

        // 执行工作流
        let instance = workflow.create_run(json!({}));
        let result = instance.run().await.unwrap();

        // 验证结果
        assert_eq!(result.results.len(), 3);
        
        assert!(matches!(
            result.results.get("step1").unwrap(),
            StepResult::Success { .. }
        ));
        
        assert!(matches!(
            result.results.get("step2").unwrap(),
            StepResult::Success { .. }
        ));
        
        // 步骤3应该被跳过，因为条件不满足
        assert!(matches!(
            result.results.get("step3").unwrap(),
            StepResult::Skipped { .. }
        ));
    }

    #[test]
    async fn test_workflow_with_error() {
        // 创建步骤
        let step1 = BasicStep::create_simple(
            "step1".to_string(),
            "成功步骤".to_string(),
            |_| Ok(json!({ "status": "ok" })),
        );

        let step2 = BasicStep::create_simple(
            "step2".to_string(),
            "失败步骤".to_string(),
            |_| Err(Error::Workflow("故意失败".to_string())),
        );

        let step3 = BasicStep::create_simple(
            "step3".to_string(),
            "依赖失败步骤的步骤".to_string(),
            |_| Ok(json!({ "result": "Should not execute" })),
        );

        // 创建条件
        let step2_success_condition = StepCondition::Reference {
            step_id: "step2".to_string(),
            path: "status".to_string(),
            query: json!({ "$eq": "ok" }),
        };

        // 创建工作流
        let workflow = Workflow::new("error_workflow".to_string(), "错误工作流".to_string())
            .add_step(step1, None, None)
            .add_step(step2, None, None)
            .add_step(step3, None, Some(step2_success_condition))
            .add_dependency("step1", "step2")
            .add_dependency("step2", "step3")
            .build();

        // 执行工作流
        let instance = workflow.create_run(json!({}));
        let result = instance.run().await.unwrap();

        // 验证结果
        assert_eq!(result.results.len(), 3);
        
        assert!(matches!(
            result.results.get("step1").unwrap(),
            StepResult::Success { .. }
        ));
        
        assert!(matches!(
            result.results.get("step2").unwrap(),
            StepResult::Failed { .. }
        ));
        
        // 步骤3应该被跳过，因为依赖的步骤2失败了
        assert!(matches!(
            result.results.get("step3").unwrap(),
            StepResult::Skipped { .. }
        ));
    }

    #[test]
    async fn test_workflow_with_trigger_data() {
        // 创建步骤
        let step1 = BasicStep::create_simple(
            "step1".to_string(),
            "使用触发数据的步骤".to_string(),
            |input| {
                Ok(json!({
                    "processed": format!("Processed: {}", input)
                }))
            },
        );

        // 创建工作流
        let workflow = Workflow::new("trigger_data_workflow".to_string(), "触发数据工作流".to_string())
            .add_step(step1, None, None)
            .build();

        // 执行工作流，传入触发数据
        let trigger_data = json!({
            "input": "Hello from trigger"
        });
        
        let instance = workflow.create_run(trigger_data);
        let result = instance.run().await.unwrap();

        // 验证结果
        assert_eq!(result.results.len(), 1);
        
        // 验证触发数据
        assert_eq!(result.trigger_data, json!({ "input": "Hello from trigger" }));
        
        // 验证步骤1成功
        assert!(matches!(
            result.results.get("step1").unwrap(),
            StepResult::Success { .. }
        ));
    }
} 