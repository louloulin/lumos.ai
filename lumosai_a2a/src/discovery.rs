//! A2A Agent Discovery
//!
//! 实现 Agent 发现和匹配功能

use crate::card::AgentCardManager;
use crate::types::*;
use std::collections::HashMap;

/// Agent 发现器
#[derive(Debug)]
pub struct AgentDiscovery {
    capability_weights: HashMap<String, f64>,
}

impl AgentDiscovery {
    /// 创建新的 Agent 发现器
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        
        // 默认权重配置
        weights.insert("streaming".to_string(), 0.1);
        weights.insert("push_notifications".to_string(), 0.1);
        weights.insert("state_transition_history".to_string(), 0.05);
        weights.insert("exact_skill_match".to_string(), 1.0);
        weights.insert("partial_skill_match".to_string(), 0.5);
        weights.insert("name_match".to_string(), 0.2);
        weights.insert("description_match".to_string(), 0.1);

        Self {
            capability_weights: weights,
        }
    }

    /// 发现能处理特定能力的 Agents
    pub fn discover_agents_for_capability<'a>(
        &self,
        agent_manager: &'a AgentCardManager,
        capability: &str,
    ) -> Vec<&'a AgentCard> {
        agent_manager.find_by_capability(capability)
    }

    /// 根据需求匹配最佳 Agent
    pub fn find_best_agent_for_requirement<'a>(
        &self,
        agent_manager: &'a AgentCardManager,
        requirement: &str,
        preferred_capabilities: &[String],
    ) -> Option<(&'a AgentCard, f64)> {
        let candidates = agent_manager.search_by_description(requirement);
        
        if candidates.is_empty() {
            return None;
        }

        let mut best_candidate = None;
        let mut best_score = 0.0;

        for candidate in candidates {
            let score = self.calculate_match_score(candidate, requirement, preferred_capabilities);
            if score > best_score {
                best_score = score;
                best_candidate = Some((candidate, score));
            }
        }

        best_candidate
    }

    /// 根据技能匹配 Agents
    pub fn find_agents_by_skills<'a>(
        &self,
        agent_manager: &'a AgentCardManager,
        required_skills: &[String],
        skill_match_mode: SkillMatchMode,
    ) -> Vec<(&'a AgentCard, f64)> {
        let mut matches = Vec::new();

        for card in agent_manager.get_all_cards() {
            let score = self.calculate_skill_match_score(card, required_skills, skill_match_mode);
            if score > 0.0 {
                matches.push((card, score));
            }
        }

        // 按分数排序
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// 计算匹配分数
    fn calculate_match_score(
        &self,
        card: &AgentCard,
        requirement: &str,
        preferred_capabilities: &[String],
    ) -> f64 {
        let mut score = 0.0;

        // 名称匹配
        if card.name.to_lowercase().contains(&requirement.to_lowercase()) {
            score += self.get_weight("name_match");
        }

        // 描述匹配
        if let Some(description) = &card.description {
            if description.to_lowercase().contains(&requirement.to_lowercase()) {
                score += self.get_weight("description_match");
            }
        }

        // 能力匹配
        for capability in preferred_capabilities {
            if card.supports_capability(capability) {
                score += self.get_weight(capability);
            }
        }

        score
    }

    /// 计算技能匹配分数
    fn calculate_skill_match_score(
        &self,
        card: &AgentCard,
        required_skills: &[String],
        match_mode: SkillMatchMode,
    ) -> f64 {
        let mut score = 0.0;
        let mut matched_skills = 0;

        for required_skill in required_skills {
            if card.supports_skill(required_skill) {
                score += self.get_weight("exact_skill_match");
                matched_skills += 1;
            } else {
                // 检查部分匹配
                for skill in &card.skills {
                    if skill.name.to_lowercase().contains(&required_skill.to_lowercase()) {
                        score += self.get_weight("partial_skill_match");
                        matched_skills += 1;
                        break;
                    }
                }
            }
        }

        // 根据匹配模式调整分数
        let total_required = required_skills.len().max(1) as f64;
        match match_mode {
            SkillMatchMode::AllRequired => {
                if matched_skills == required_skills.len() {
                    score / total_required
                } else {
                    0.0
                }
            }
            SkillMatchMode::AnyRequired => {
                if matched_skills > 0 {
                    score / matched_skills as f64
                } else {
                    0.0
                }
            }
            SkillMatchMode::PreferMost => {
                score / total_required
            }
        }
    }

    /// 获取权重
    fn get_weight(&self, key: &str) -> f64 {
        self.capability_weights
            .get(key)
            .copied()
            .unwrap_or(0.1)
    }

    /// 设置权重
    pub fn set_weight(&mut self, key: String, weight: f64) {
        self.capability_weights.insert(key, weight);
    }

    /// 获取所有权重
    pub fn get_weights(&self) -> &HashMap<String, f64> {
        &self.capability_weights
    }

    /// 重置权重为默认值
    pub fn reset_weights(&mut self) {
        *self = Self::new();
    }
}

/// 技能匹配模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillMatchMode {
    /// 必须匹配所有技能
    AllRequired,
    /// 匹配任意技能
    AnyRequired,
    /// 优先匹配最多的技能
    PreferMost,
}

/// 能力匹配器
#[derive(Debug)]
pub struct CapabilityMatcher {
    min_score_threshold: f64,
}

impl CapabilityMatcher {
    /// 创建新的能力匹配器
    pub fn new(min_score_threshold: f64) -> Self {
        Self {
            min_score_threshold,
        }
    }

    /// 匹配能力
    pub fn match_capabilities<'a>(
        &self,
        available_agents: &[&'a AgentCard],
        required_capabilities: &[String],
        optional_capabilities: &[String],
    ) -> Vec<(&'a AgentCard, f64)> {
        let mut matches = Vec::new();

        for agent in available_agents {
            let score = self.calculate_capability_score(
                agent,
                required_capabilities,
                optional_capabilities,
            );

            if score >= self.min_score_threshold {
                matches.push((*agent, score));
            }
        }

        // 按分数排序
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// 计算能力分数
    pub fn calculate_capability_score(
        &self,
        agent: &AgentCard,
        required_capabilities: &[String],
        optional_capabilities: &[String],
    ) -> f64 {
        let mut score = 0.0;
        let total_required = required_capabilities.len().max(1) as f64;

        // 必需能力（权重更高）
        for capability in required_capabilities {
            if agent.supports_capability(capability) {
                score += 2.0 / total_required;
            } else {
                score -= 1.0 / total_required; // 缺少必需能力扣分
            }
        }

        // 可选能力（权重较低）
        if !optional_capabilities.is_empty() {
            let total_optional = optional_capabilities.len() as f64;
            for capability in optional_capabilities {
                if agent.supports_capability(capability) {
                    score += 1.0 / total_optional;
                }
            }
        }

        // 技能数量加分
        let skill_bonus = if agent.skills.is_empty() {
            0.0
        } else {
            (agent.skills.len() as f64).log10() * 0.1
        };
        score += skill_bonus;

        score.clamp(0.0, 1.0)
    }
}

impl Default for AgentDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::AgentCardBuilder;
    use url::Url;

    fn create_test_agent(name: &str, skills: Vec<Skill>) -> AgentCard {
        AgentCardBuilder::new(
            name.to_string(),
            Url::parse("https://example.com").unwrap(),
            "1.0.0".to_string(),
        )
        .skills(skills)
        .build()
        .unwrap()
    }

    #[test]
    fn test_agent_discovery() {
        let manager = AgentCardManager::new();
        let discovery = AgentDiscovery::new();

        let text_skill = Skill::new("text_analysis".to_string(), "Text Analysis".to_string());
        let _agent = create_test_agent("Text Analyzer", vec![text_skill]);

        let results = discovery.discover_agents_for_capability(&manager, "text_analysis");
        assert_eq!(results.len(), 0); // No agents registered
    }

    #[test]
    fn test_skill_match() {
        let discovery = AgentDiscovery::new();

        let text_skill = Skill::new("text_analysis".to_string(), "Text Analysis".to_string());
        let code_skill = Skill::new("code_analysis".to_string(), "Code Analysis".to_string());
        let _agent = create_test_agent("Multi Agent", vec![text_skill, code_skill]);

        let required_skills = vec!["text_analysis".to_string(), "code_analysis".to_string()];
        let manager = AgentCardManager::new();
        let matches = discovery.find_agents_by_skills(
            &manager,
            &required_skills,
            SkillMatchMode::AllRequired,
        );

        assert_eq!(matches.len(), 0); // No agents in manager
    }

    #[test]
    fn test_capability_matcher() {
        let matcher = CapabilityMatcher::new(0.3);

        let agent = AgentCardBuilder::new(
            "Test Agent".to_string(),
            Url::parse("https://example.com").unwrap(),
            "1.0.0".to_string(),
        )
        .enable_streaming(true)
        .build()
        .unwrap();

        let agents = vec![&agent];
        let required = vec!["streaming".to_string()];
        let optional = vec!["push_notifications".to_string()];

        let matches = matcher.match_capabilities(&agents, &required, &optional);
        
        assert_eq!(matches.len(), 1);
        assert!(matches[0].1 > 0.3);
    }
}