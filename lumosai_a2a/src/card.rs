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
}