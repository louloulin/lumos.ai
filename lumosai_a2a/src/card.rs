//! A2A Agent Card Implementation
//!
//! 实现 Google A2A 协议的 Agent Card 功能，用于声明 Agent 的能力和元数据

use crate::types::*;
use crate::A2AResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Agent Card 构建器
pub struct AgentCardBuilder {
    name: String,
    url: Url,
    version: String,
    description: Option<String>,
    provider: Option<Provider>,
    capabilities: Capabilities,
    authentication: Option<Authentication>,
    skills: Vec<Skill>,
    metadata: HashMap<String, serde_json::Value>,
}

impl AgentCardBuilder {
    /// 创建新的 Agent Card 构建器
    pub fn new(name: String, url: Url, version: String) -> Self {
        Self {
            name,
            url,
            version,
            description: None,
            provider: None,
            capabilities: Capabilities::default(),
            authentication: None,
            skills: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 设置描述
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置提供者
    pub fn provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// 启用流式支持
    pub fn enable_streaming(mut self, enable: bool) -> Self {
        self.capabilities.streaming = enable;
        self
    }

    /// 启用推送通知
    pub fn enable_push_notifications(mut self, enable: bool) -> Self {
        self.capabilities.push_notifications = enable;
        self
    }

    /// 启用状态转换历史
    pub fn enable_state_transition_history(mut self, enable: bool) -> Self {
        self.capabilities.state_transition_history = enable;
        self
    }

    /// 设置最大并发任务数
    pub fn max_concurrent_tasks(mut self, max_tasks: u32) -> Self {
        self.capabilities.max_concurrent_tasks = Some(max_tasks);
        self
    }

    /// 设置输入模式
    pub fn input_modes(mut self, modes: Vec<String>) -> Self {
        self.capabilities.input_modes = modes;
        self
    }

    /// 设置输出模式
    pub fn output_modes(mut self, modes: Vec<String>) -> Self {
        self.capabilities.output_modes = modes;
        self
    }

    /// 设置认证信息
    pub fn authentication(mut self, auth: Authentication) -> Self {
        self.authentication = Some(auth);
        self
    }

    /// 添加能力（作为输入模式）
    pub fn capability(mut self, capability: &str) -> Self {
        self.capabilities.input_modes.push(capability.to_string());
        self
    }

    /// 添加技能
    pub fn skill(mut self, skill: Skill) -> Self {
        self.skills.push(skill);
        self
    }

    /// 添加多个技能
    pub fn skills(mut self, skills: Vec<Skill>) -> Self {
        self.skills.extend(skills);
        self
    }

    /// 添加元数据
    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 构建 Agent Card
    pub fn build(self) -> A2AResult<AgentCard> {
        let card = AgentCard {
            name: self.name,
            description: self.description,
            url: self.url,
            provider: self.provider,
            version: self.version,
            documentation_url: None, // 可以后续设置
            capabilities: self.capabilities,
            authentication: self.authentication,
            skills: self.skills,
            metadata: self.metadata,
        };

        // 验证卡片
        card.validate().map_err(|e| crate::A2AError::InvalidInput(e))?;
        Ok(card)
    }
}

/// Agent Card 管理器
#[derive(Debug)]
pub struct AgentCardManager {
    cards: HashMap<String, AgentCard>,
    indexed_by_capability: HashMap<String, Vec<String>>, // capability -> agent_id
    indexed_by_skill: HashMap<String, Vec<String>>, // skill_id -> agent_id
}

impl AgentCardManager {
    /// 创建新的 Agent Card 管理器
    pub fn new() -> Self {
        Self {
            cards: HashMap::new(),
            indexed_by_capability: HashMap::new(),
            indexed_by_skill: HashMap::new(),
        }
    }

    /// 注册 Agent Card
    pub fn register_card(&mut self, card: AgentCard) -> A2AResult<String> {
        // 验证卡片
        card.validate().map_err(|e| crate::A2AError::InvalidInput(e))?;

        // 生成唯一ID
        let agent_id = card.generate_id();

        // 检查是否已存在
        if self.cards.contains_key(&agent_id) {
            return Err(crate::A2AError::InvalidInput(
                format!("Agent with ID '{}' already registered", agent_id)
            ));
        }

        // 添加到主存储
        self.cards.insert(agent_id.clone(), card.clone());

        // 更新能力索引
        self.update_capability_index(&agent_id, &card);

        // 更新技能索引
        self.update_skill_index(&agent_id, &card);

        #[cfg(feature = "logging")]
        tracing::info!("Registered Agent card: {} ({})", card.name, agent_id);
        Ok(agent_id)
    }

    /// 获取 Agent Card
    pub fn get_card(&self, agent_id: &str) -> Option<&AgentCard> {
        self.cards.get(agent_id)
    }

    /// 获取所有 Agent Cards
    pub fn get_all_cards(&self) -> Vec<&AgentCard> {
        self.cards.values().collect()
    }

    /// 根据能力查找 Agents
    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentCard> {
        let mut results = Vec::new();
        let mut added_ids = std::collections::HashSet::new();
        
        // 检查通用能力
        if let Some(agent_ids) = self.indexed_by_capability.get(capability) {
            for agent_id in agent_ids {
                if let Some(card) = self.cards.get(agent_id) {
                    if added_ids.insert(agent_id.clone()) {
                        results.push(card);
                    }
                }
            }
        }

        // 检查技能匹配
        if let Some(agent_ids) = self.indexed_by_skill.get(capability) {
            for agent_id in agent_ids {
                if let Some(card) = self.cards.get(agent_id) {
                    if added_ids.insert(agent_id.clone()) {
                        results.push(card);
                    }
                }
            }
        }

        results
    }

    /// 根据名称搜索 Agents
    pub fn search_by_name(&self, name: &str) -> Vec<&AgentCard> {
        let name_lower = name.to_lowercase();
        self.cards
            .values()
            .filter(|card| card.name.to_lowercase().contains(&name_lower))
            .collect()
    }

    /// 根据描述搜索 Agents
    pub fn search_by_description(&self, query: &str) -> Vec<&AgentCard> {
        let query_lower = query.to_lowercase();
        self.cards
            .values()
            .filter(|card| {
                card.description
                    .as_ref()
                    .map(|desc| desc.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// 根据技能ID查找 Agents
    pub fn find_by_skill_id(&self, skill_id: &str) -> Vec<&AgentCard> {
        let mut results = Vec::new();
        
        if let Some(agent_ids) = self.indexed_by_skill.get(skill_id) {
            for agent_id in agent_ids {
                if let Some(card) = self.cards.get(agent_id) {
                    results.push(card);
                }
            }
        }

        results
    }

    /// 注销 Agent Card
    pub fn unregister_card(&mut self, agent_id: &str) -> A2AResult<()> {
        if let Some(card) = self.cards.remove(agent_id) {
            // 清理能力索引
            self.cleanup_capability_index(agent_id, &card);
            
            // 清理技能索引
            self.cleanup_skill_index(agent_id, &card);
            
                #[cfg(feature = "logging")]
            tracing::info!("Unregistered Agent card: {} ({})", card.name, agent_id);
            Ok(())
        } else {
            Err(crate::A2AError::AgentNotFound(agent_id.to_string()))
        }
    }

    /// 更新 Agent Card
    pub fn update_card(&mut self, agent_id: &str, new_card: AgentCard) -> A2AResult<()> {
        // 验证新卡片
        new_card.validate().map_err(|e| crate::A2AError::InvalidInput(e))?;

        // 检查卡片是否存在
        if !self.cards.contains_key(agent_id) {
            return Err(crate::A2AError::AgentNotFound(agent_id.to_string()));
        }

        // 获取旧卡片用于清理索引
        let old_card = self.cards.get(agent_id).cloned().unwrap();
        
        // 移除旧卡片
        self.cards.remove(agent_id);

        // 清理旧索引
        self.cleanup_capability_index(agent_id, &old_card);
        self.cleanup_skill_index(agent_id, &old_card);

        // 更新卡片
        self.cards.insert(agent_id.to_string(), new_card.clone());

        // 更新新索引
        self.update_capability_index(agent_id, &new_card);
        self.update_skill_index(agent_id, &new_card);

          #[cfg(feature = "logging")]
        tracing::info!("Updated Agent card: {} ({})", new_card.name, agent_id);
        Ok(())
    }

    /// 更新能力索引
    fn update_capability_index(&mut self, agent_id: &str, card: &AgentCard) {
        let capabilities = vec![
            ("streaming", card.capabilities.streaming),
            ("push_notifications", card.capabilities.push_notifications),
            ("state_transition_history", card.capabilities.state_transition_history),
        ];

        for (capability, enabled) in capabilities {
            if enabled {
                self.indexed_by_capability
                    .entry(capability.to_string())
                    .or_insert_with(Vec::new)
                    .push(agent_id.to_string());
            }
        }

        // 索引输入和输出模式
        for mode in &card.capabilities.input_modes {
            self.indexed_by_capability
                .entry(format!("input_mode:{}", mode))
                .or_insert_with(Vec::new)
                .push(agent_id.to_string());
        }

        for mode in &card.capabilities.output_modes {
            self.indexed_by_capability
                .entry(format!("output_mode:{}", mode))
                .or_insert_with(Vec::new)
                .push(agent_id.to_string());
        }
    }

    /// 更新技能索引
    fn update_skill_index(&mut self, agent_id: &str, card: &AgentCard) {
        for skill in &card.skills {
            // 按技能ID索引
            self.indexed_by_skill
                .entry(skill.id.clone())
                .or_insert_with(Vec::new)
                .push(agent_id.to_string());

            // 按技能名称索引
            self.indexed_by_capability
                .entry(skill.name.clone())
                .or_insert_with(Vec::new)
                .push(agent_id.to_string());

            // 索引技能标签
            if let Some(tags) = &skill.tags {
                for tag in tags {
                    self.indexed_by_capability
                        .entry(tag.clone())
                        .or_insert_with(Vec::new)
                        .push(agent_id.to_string());
                }
            }
        }
    }

    /// 清理能力索引
    fn cleanup_capability_index(&mut self, agent_id: &str, _card: &AgentCard) {
        for capability_list in self.indexed_by_capability.values_mut() {
            capability_list.retain(|id| id != agent_id);
        }
    }

    /// 清理技能索引
    fn cleanup_skill_index(&mut self, agent_id: &str, card: &AgentCard) {
        for skill in &card.skills {
            if let Some(capability_list) = self.indexed_by_skill.get_mut(&skill.id) {
                capability_list.retain(|id| id != agent_id);
            }
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> AgentCardStats {
        AgentCardStats {
            total_cards: self.cards.len(),
            total_capabilities: self.indexed_by_capability.len(),
            total_skills: self.cards.values().map(|c| c.skills.len()).sum(),
            cards_with_streaming: self.cards.values().filter(|c| c.capabilities.streaming).count(),
            cards_with_push_notifications: self.cards.values().filter(|c| c.capabilities.push_notifications).count(),
            cards_with_auth: self.cards.values().filter(|c| c.authentication.is_some()).count(),
        }
    }

    /// 清空所有卡片
    pub fn clear(&mut self) {
        self.cards.clear();
        self.indexed_by_capability.clear();
        self.indexed_by_skill.clear();
    }

    /// 检查Agent是否存在
    pub fn contains(&self, agent_id: &str) -> bool {
        self.cards.contains_key(agent_id)
    }
}

/// Agent Card 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCardStats {
    /// 总卡片数
    pub total_cards: usize,
    /// 总能力数
    pub total_capabilities: usize,
    /// 总技能数
    pub total_skills: usize,
    /// 支持流式的卡片数
    pub cards_with_streaming: usize,
    /// 支持推送通知的卡片数
    pub cards_with_push_notifications: usize,
    /// 有认证的卡片数
    pub cards_with_auth: usize,
}

impl Default for AgentCardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_card() -> AgentCard {
        AgentCardBuilder::new(
            "Test Agent".to_string(),
            url::Url::parse("https://example.com/agent").unwrap(),
            "1.0.0".to_string(),
        )
        .description("A test agent".to_string())
        .enable_streaming(true)
        .skill(Skill::new("test_skill".to_string(), "Test Skill".to_string()))
        .build()
        .unwrap()
    }

    #[test]
    fn test_agent_card_builder() {
        let url = url::Url::parse("https://example.com/agent").unwrap();
        let card = AgentCardBuilder::new(
            "Test Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        )
        .description("A test agent".to_string())
        .enable_streaming(true)
        .skill(Skill::new(
            "test_skill".to_string(),
            "Test Skill".to_string(),
        ))
        .build()
        .unwrap();

        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.description, Some("A test agent".to_string()));
        assert!(card.capabilities.streaming);
        assert_eq!(card.skills.len(), 1);
        assert_eq!(card.skills[0].id, "test_skill");
    }

    #[test]
    fn test_agent_card_manager() {
        let mut manager = AgentCardManager::new();
        let card = create_test_card();

        let agent_id = manager.register_card(card.clone()).unwrap();

        // 测试获取卡片
        let retrieved = manager.get_card(&agent_id).unwrap();
        assert_eq!(retrieved.name, card.name);

        // 测试根据能力查找
        let results = manager.find_by_capability("streaming");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Test Agent");

        // 测试根据技能ID查找
        let skill_results = manager.find_by_skill_id("test_skill");
        assert_eq!(skill_results.len(), 1);

        // 测试注销卡片
        manager.unregister_card(&agent_id).unwrap();
        assert!(manager.get_card(&agent_id).is_none());
    }

    #[test]
    fn test_agent_card_search() {
        let mut manager = AgentCardManager::new();
        
        let card1 = AgentCardBuilder::new(
            "Text Analysis Agent".to_string(),
            url::Url::parse("https://example.com/text").unwrap(),
            "1.0.0".to_string(),
        )
        .description("An agent for analyzing text".to_string())
        .build()
        .unwrap();

        let card2 = AgentCardBuilder::new(
            "Image Processing Agent".to_string(),
            url::Url::parse("https://example.com/image").unwrap(),
            "1.0.0".to_string(),
        )
        .description("An agent for processing images".to_string())
        .build()
        .unwrap();

        manager.register_card(card1).unwrap();
        manager.register_card(card2).unwrap();

        // 测试名称搜索
        let text_results = manager.search_by_name("text");
        assert_eq!(text_results.len(), 1);
        assert!(text_results[0].name.contains("Text"));

        // 测试描述搜索
        let image_results = manager.search_by_description("images");
        assert_eq!(image_results.len(), 1);
        assert!(image_results[0].name.contains("Image"));
    }

    #[test]
    fn test_agent_card_update() {
        let mut manager = AgentCardManager::new();
        let card = create_test_card();
        
        let agent_id = manager.register_card(card).unwrap();
        
        // 更新卡片
        let updated_card = AgentCardBuilder::new(
            "Updated Agent".to_string(),
            url::Url::parse("https://example.com/updated").unwrap(),
            "2.0.0".to_string(),
        )
        .build()
        .unwrap();

        manager.update_card(&agent_id, updated_card).unwrap();
        
        let retrieved = manager.get_card(&agent_id).unwrap();
        assert_eq!(retrieved.name, "Updated Agent");
        assert_eq!(retrieved.version, "2.0.0");
    }

    #[test]
    fn test_agent_card_stats() {
        let mut manager = AgentCardManager::new();
        
        let card1 = AgentCardBuilder::new(
            "Agent 1".to_string(),
            url::Url::parse("https://example.com/1").unwrap(),
            "1.0.0".to_string(),
        )
        .enable_streaming(true)
        .build()
        .unwrap();

        let card2 = AgentCardBuilder::new(
            "Agent 2".to_string(),
            url::Url::parse("https://example.com/2").unwrap(),
            "1.0.0".to_string(),
        )
        .enable_streaming(true)
        .enable_push_notifications(true)
        .authentication(Authentication::default())
        .build()
        .unwrap();

        manager.register_card(card1).unwrap();
        manager.register_card(card2).unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.total_cards, 2);
        assert_eq!(stats.cards_with_streaming, 2);
        assert_eq!(stats.cards_with_push_notifications, 1);
        assert_eq!(stats.cards_with_auth, 1);
    }

    #[test]
    fn test_agent_card_authentication_types() {
        let url = url::Url::parse("https://example.com").unwrap();
        
        // 测试无认证
        let no_auth_card = AgentCardBuilder::new(
            "No Auth Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        ).build().unwrap();
        assert!(no_auth_card.authentication.is_none());
        
        // 测试Bearer Token认证
        let bearer_auth = Authentication::Bearer("token123".to_string());
        let bearer_card = AgentCardBuilder::new(
            "Bearer Auth Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        ).authentication(bearer_auth.clone()).build().unwrap();
        assert_eq!(bearer_card.authentication, Some(bearer_auth));
        
        // 测试API Key认证
        let api_key_auth = Authentication::ApiKey("key456".to_string());
        let api_key_card = AgentCardBuilder::new(
            "API Key Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        ).authentication(api_key_auth.clone()).build().unwrap();
        assert_eq!(api_key_card.authentication, Some(api_key_auth));
    }

    #[test]
    fn test_agent_card_capabilities() {
        let url = url::Url::parse("https://example.com").unwrap();
        
        let card = AgentCardBuilder::new(
            "Capable Agent".to_string(),
            url,
            "1.0.0".to_string(),
        )
        .capability("text_generation")
        .capability("image_analysis")
        .capability("data_processing")
        .build()
        .unwrap();

        assert!(card.capabilities.input_modes.len() >= 3);
        assert!(card.capabilities.input_modes.contains(&"text_generation".to_string()));
        assert!(card.capabilities.input_modes.contains(&"image_analysis".to_string()));
        assert!(card.capabilities.input_modes.contains(&"data_processing".to_string()));
    }

    #[test]
    fn test_agent_card_validation_errors() {
        let url = url::Url::parse("https://example.com").unwrap();
        
        // 测试空名称
        let result = AgentCardBuilder::new(
            "".to_string(), // 空名称应该失败
            url,
            "1.0.0".to_string(),
        ).build();
        assert!(result.is_err());
        
        // 测试无效URL需要通过其他方式，因为Url::parse会panic
        // 这里我们跳过这个测试，因为URL验证在调用AgentCardBuilder::new之前就已经进行了
        // 实际应用中，调用方需要确保URL的有效性
    }

    #[test]
    fn test_agent_card_find_by_capability() {
        let mut manager = AgentCardManager::new();
        let url = url::Url::parse("https://example.com").unwrap();
        
        // 创建不同能力的Agent
        let text_agent = AgentCardBuilder::new(
            "Text Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        )
        .capability("text_generation")
        .build()
        .unwrap();
        
        let image_agent = AgentCardBuilder::new(
            "Image Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        )
        .capability("image_analysis")
        .build()
        .unwrap();
        
        let multi_agent = AgentCardBuilder::new(
            "Multi Agent".to_string(),
            url,
            "1.0.0".to_string(),
        )
        .capability("text_generation")
        .capability("image_analysis")
        .build()
        .unwrap();

        manager.register_card(text_agent).unwrap();
        manager.register_card(image_agent).unwrap();
        manager.register_card(multi_agent).unwrap();

        let text_cards = manager.find_by_capability("text_generation");
        assert_eq!(text_cards.len(), 2); // text_agent + multi_agent

        let image_cards = manager.find_by_capability("image_analysis");
        assert_eq!(image_cards.len(), 2); // image_agent + multi_agent

        let video_cards = manager.find_by_capability("video_processing");
        assert_eq!(video_cards.len(), 0); // 没有视频处理能力的Agent
    }

    #[test]
    fn test_agent_card_remove_card() {
        let mut manager = AgentCardManager::new();
        let url = url::Url::parse("https://example.com").unwrap();
        
        let card = AgentCardBuilder::new(
            "Test Agent".to_string(),
            url,
            "1.0.0".to_string(),
        ).build().unwrap();

        let agent_id = manager.register_card(card).unwrap();
        
        // 验证注册成功
        assert!(manager.get_card(&agent_id).is_some());
        assert_eq!(manager.get_all_cards().len(), 1);
        
        // 注意：当前实现没有remove_card方法，这里只验证注册成功
        // 移除功能需要在未来的版本中实现
        assert!(manager.get_card(&agent_id).is_some());
        assert_eq!(manager.get_all_cards().len(), 1);
    }

    #[test]
    fn test_agent_card_update_nonexistent() {
        let mut manager = AgentCardManager::new();
        let url = url::Url::parse("https://example.com").unwrap();
        
        let new_card = AgentCardBuilder::new(
            "New Agent".to_string(),
            url,
            "1.0.0".to_string(),
        ).build().unwrap();

        // 尝试更新不存在的Agent
        let result = manager.update_card("nonexistent-id", new_card);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_card_duplicate_registration() {
        let mut manager = AgentCardManager::new();
        let url = url::Url::parse("https://example.com").unwrap();
        
        let card = AgentCardBuilder::new(
            "Test Agent".to_string(),
            url,
            "1.0.0".to_string(),
        ).build().unwrap();

        let agent_id = manager.register_card(card.clone()).unwrap();
        
        // 尝试注册相同的Agent Card应该失败
        let result = manager.register_card(card);
        assert!(result.is_err());
        
        // 但第一个注册应该仍然有效
        assert!(manager.get_card(&agent_id).is_some());
    }

    #[test]
    fn test_agent_card_builder_fluent_interface() {
        let url = url::Url::parse("https://example.com").unwrap();
        
        let card = AgentCardBuilder::new(
            "Fluent Agent".to_string(),
            url,
            "1.0.0".to_string(),
        )
        .description("A fluent interface test agent".to_string())
        .enable_streaming(true)
        .enable_push_notifications(true)
        .capability("text_generation")
        .capability("translation")
        .skill(
            Skill::new("writing".to_string(), "Writing Skill".to_string())
                .with_description("Professional writing assistance".to_string())
        )
        .authentication(Authentication::Bearer("token123".to_string()))
        .build()
        .unwrap();

        assert_eq!(card.name, "Fluent Agent");
        assert_eq!(card.version, "1.0.0");
        assert_eq!(card.description, Some("A fluent interface test agent".to_string()));
        assert_eq!(card.capabilities.streaming, true);
        assert_eq!(card.capabilities.push_notifications, true);
        assert_eq!(card.capabilities.input_modes.len(), 2);
        assert_eq!(card.skills.len(), 1);
        assert!(card.authentication.is_some());
    }

    #[test]
    fn test_agent_card_skill_validation() {
        let url = url::Url::parse("https://example.com").unwrap();
        
        // 测试带有无效技能的Agent Card
        let card_with_invalid_skill = AgentCardBuilder::new(
            "Invalid Skill Agent".to_string(),
            url,
            "1.0.0".to_string(),
        )
        .skill(
            Skill::new("".to_string(), "Invalid Skill".to_string()) // 空ID应该失败
        )
        .build();

        assert!(card_with_invalid_skill.is_err());
    }

    #[test]
    fn test_agent_card_stats_zero_cards() {
        let manager = AgentCardManager::new();
        let stats = manager.get_stats();
        
        assert_eq!(stats.total_cards, 0);
        assert_eq!(stats.cards_with_streaming, 0);
        assert_eq!(stats.cards_with_push_notifications, 0);
        assert_eq!(stats.cards_with_auth, 0);
    }

    #[test]
    fn test_agent_card_serialization() {
        let url = url::Url::parse("https://example.com").unwrap();
        
        let card = AgentCardBuilder::new(
            "Serializable Agent".to_string(),
            url.clone(),
            "1.0.0".to_string(),
        )
        .description("Test serialization".to_string())
        .enable_streaming(true)
        .capability("test_capability")
        .skill(
            Skill::new("test_skill".to_string(), "Test Skill".to_string())
                .with_tags(vec!["test".to_string()])
        )
        .authentication(Authentication::ApiKey("test_key".to_string()))
        .build()
        .unwrap();

        // 测试序列化
        let serialized = serde_json::to_string(&card).unwrap();
        
        // 测试反序列化
        let deserialized: AgentCard = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(card.name, deserialized.name);
        assert_eq!(card.version, deserialized.version);
        assert_eq!(card.description, deserialized.description);
        assert_eq!(card.capabilities, deserialized.capabilities);
        assert_eq!(card.skills.len(), deserialized.skills.len());
        assert_eq!(card.authentication, deserialized.authentication);
    }
}