//! 混合检索演示程序 - Hybrid Retrieval
//!
//! 演示如何使用混合检索结合向量检索和关键词检索

use lumosai_rag::{
    retriever::{
        BM25Config, BM25Retriever, HybridRetriever, HybridSearchConfig, RerankStrategy, Retriever,
    },
    types::{Document, Metadata, RetrievalOptions, RetrievalRequest, ScoredDocument},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 混合检索演示 - Hybrid Retrieval\n");
    println!("{}", "=".repeat(80));
    println!();

    // 演示 1: 创建混合检索器
    demo_create_hybrid_retriever().await?;

    // 演示 2: 加权求和融合
    demo_weighted_sum_fusion().await?;

    // 演示 3: 倒数排名融合 (RRF)
    demo_reciprocal_rank_fusion().await?;

    // 演示 4: 凸组合融合
    demo_convex_combination_fusion().await?;

    // 总结
    println!("\n{}", "=".repeat(80));
    println!("✅ 混合检索演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 功能总结:");
    println!("  1. 向量检索: 基于语义相似度的检索");
    println!("  2. 关键词检索: 基于 BM25 的精确匹配");
    println!("  3. 混合融合: 结合两种检索方法的优势");
    println!("  4. 多种融合策略: WeightedSum, RRF, ConvexCombination, RankBased");
    println!();
    println!("💡 技术优势:");
    println!("  - 召回率提升: 结合语义和关键词检索");
    println!("  - 精确度提升: 多种融合策略优化排序");
    println!("  - 灵活配置: 可调整权重和融合策略");
    println!("  - 性能优化: 并行执行两种检索");
    println!();
    println!("🚀 对标 LlamaIndex:");
    println!("  ✅ 混合检索（Vector + BM25）");
    println!("  ✅ 倒数排名融合 (RRF)");
    println!("  ✅ 加权融合策略");
    println!("  ✅ 可配置的融合参数");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// 演示 1: 创建混合检索器
async fn demo_create_hybrid_retriever() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 演示 1: 创建混合检索器");
    println!("{}", "-".repeat(80));

    // 创建配置
    let config = HybridSearchConfig {
        vector_weight: 0.7,
        keyword_weight: 0.3,
        min_score_threshold: 0.1,
        max_candidates_per_method: 100,
        rerank_strategy: RerankStrategy::WeightedSum,
    };

    println!("✅ 混合检索配置:");
    println!("  - 向量检索权重: {}", config.vector_weight);
    println!("  - 关键词检索权重: {}", config.keyword_weight);
    println!("  - 最小分数阈值: {}", config.min_score_threshold);
    println!("  - 每种方法最大候选数: {}", config.max_candidates_per_method);
    println!("  - 融合策略: WeightedSum");

    println!("\n💡 混合检索原理:");
    println!("  1. 向量检索: 使用嵌入向量进行语义相似度匹配");
    println!("  2. 关键词检索: 使用 BM25 算法进行精确词匹配");
    println!("  3. 结果融合: 使用融合策略合并两种检索结果");
    println!("  4. 分数归一化: 统一不同检索方法的分数范围");
    println!("  5. 排序输出: 按最终分数降序返回结果");

    println!();
    Ok(())
}

/// 演示 2: 加权求和融合
async fn demo_weighted_sum_fusion() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚖️  演示 2: 加权求和融合 (Weighted Sum)");
    println!("{}", "-".repeat(80));

    println!("📐 加权求和公式:");
    println!("  final_score = α × vector_score + β × keyword_score");
    println!("  其中: α = 0.7 (向量权重), β = 0.3 (关键词权重)");
    println!();

    // 模拟检索结果
    let vector_results = vec![
        ("doc1", 0.95, "LumosAI is a Rust-based AI framework..."),
        ("doc2", 0.85, "Rust programming language for AI..."),
        ("doc3", 0.75, "AI framework comparison..."),
    ];

    let keyword_results = vec![
        ("doc1", 0.80, "LumosAI is a Rust-based AI framework..."),
        ("doc4", 0.70, "Rust systems programming..."),
        ("doc2", 0.60, "Rust programming language for AI..."),
    ];

    println!("📊 向量检索结果:");
    for (id, score, content) in &vector_results {
        println!("  {} [分数: {:.2}] {}", id, score, &content[..40.min(content.len())]);
    }

    println!("\n📊 关键词检索结果:");
    for (id, score, content) in &keyword_results {
        println!("  {} [分数: {:.2}] {}", id, score, &content[..40.min(content.len())]);
    }

    // 计算融合分数
    let mut combined_scores: HashMap<&str, f32> = HashMap::new();
    
    for (id, score, _) in &vector_results {
        combined_scores.insert(id, score * 0.7);
    }
    
    for (id, score, _) in &keyword_results {
        let existing = combined_scores.get(id).unwrap_or(&0.0);
        combined_scores.insert(id, existing + score * 0.3);
    }

    let mut results: Vec<_> = combined_scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n🎯 融合后的结果:");
    for (id, score) in results {
        println!("  {} [最终分数: {:.2}]", id, score);
    }

    println!("\n💡 加权求和优势:");
    println!("  - 简单直观: 易于理解和调整");
    println!("  - 灵活配置: 可根据场景调整权重");
    println!("  - 计算高效: 线性时间复杂度");

    println!();
    Ok(())
}

/// 演示 3: 倒数排名融合 (RRF)
async fn demo_reciprocal_rank_fusion() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔢 演示 3: 倒数排名融合 (Reciprocal Rank Fusion)");
    println!("{}", "-".repeat(80));

    println!("📐 RRF 公式:");
    println!("  RRF_score(d) = Σ 1 / (k + rank(d))");
    println!("  其中: k = 60 (常数), rank(d) 是文档在列表中的排名");
    println!();

    // 模拟排名结果
    let vector_ranks = vec![
        ("doc1", 1),
        ("doc2", 2),
        ("doc3", 3),
    ];

    let keyword_ranks = vec![
        ("doc1", 1),
        ("doc4", 2),
        ("doc2", 3),
    ];

    println!("📊 向量检索排名:");
    for (id, rank) in &vector_ranks {
        println!("  排名 {}: {}", rank, id);
    }

    println!("\n📊 关键词检索排名:");
    for (id, rank) in &keyword_ranks {
        println!("  排名 {}: {}", rank, id);
    }

    // 计算 RRF 分数
    let k = 60.0;
    let mut rrf_scores: HashMap<&str, f32> = HashMap::new();
    
    for (id, rank) in &vector_ranks {
        let score = 1.0 / (k + *rank as f32);
        rrf_scores.insert(id, score);
    }
    
    for (id, rank) in &keyword_ranks {
        let score = 1.0 / (k + *rank as f32);
        let existing = rrf_scores.get(id).unwrap_or(&0.0);
        rrf_scores.insert(id, existing + score);
    }

    let mut results: Vec<_> = rrf_scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n🎯 RRF 融合后的结果:");
    for (id, score) in results {
        println!("  {} [RRF 分数: {:.4}]", id, score);
    }

    println!("\n💡 RRF 优势:");
    println!("  - 排名优先: 关注文档排名而非绝对分数");
    println!("  - 鲁棒性强: 对分数尺度不敏感");
    println!("  - 广泛应用: 搜索引擎常用融合方法");
    println!("  - 无需归一化: 自动处理不同分数范围");

    println!();
    Ok(())
}

/// 演示 4: 凸组合融合
async fn demo_convex_combination_fusion() -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 演示 4: 凸组合融合 (Convex Combination)");
    println!("{}", "-".repeat(80));

    println!("📐 凸组合公式:");
    println!("  final_score = α × norm(vector_score) + (1-α) × norm(keyword_score)");
    println!("  其中: α = 0.7, norm() 是归一化函数");
    println!();

    // 模拟检索结果
    let vector_results = vec![
        ("doc1", 0.95),
        ("doc2", 0.85),
        ("doc3", 0.75),
    ];

    let keyword_results = vec![
        ("doc1", 8.5),  // BM25 分数范围不同
        ("doc4", 7.2),
        ("doc2", 6.8),
    ];

    println!("📊 向量检索结果（归一化前）:");
    for (id, score) in &vector_results {
        println!("  {} [分数: {:.2}]", id, score);
    }

    println!("\n📊 关键词检索结果（归一化前）:");
    for (id, score) in &keyword_results {
        println!("  {} [分数: {:.2}]", id, score);
    }

    // 归一化
    let max_vector = vector_results.iter().map(|(_, s)| s).fold(0.0f32, |a, &b| a.max(b));
    let max_keyword = keyword_results.iter().map(|(_, s)| s).fold(0.0f32, |a, &b| a.max(b));

    let normalized_vector: Vec<_> = vector_results.iter()
        .map(|(id, score)| (*id, score / max_vector))
        .collect();

    let normalized_keyword: Vec<_> = keyword_results.iter()
        .map(|(id, score)| (*id, score / max_keyword))
        .collect();

    println!("\n📊 归一化后的向量检索:");
    for (id, score) in &normalized_vector {
        println!("  {} [归一化分数: {:.2}]", id, score);
    }

    println!("\n📊 归一化后的关键词检索:");
    for (id, score) in &normalized_keyword {
        println!("  {} [归一化分数: {:.2}]", id, score);
    }

    // 凸组合
    let alpha = 0.7;
    let mut combined_scores: HashMap<&str, f32> = HashMap::new();
    
    for (id, score) in &normalized_vector {
        combined_scores.insert(id, alpha * score);
    }
    
    for (id, score) in &normalized_keyword {
        let existing = combined_scores.get(id).unwrap_or(&0.0);
        combined_scores.insert(id, existing + (1.0 - alpha) * score);
    }

    let mut results: Vec<_> = combined_scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n🎯 凸组合融合后的结果:");
    for (id, score) in results {
        println!("  {} [最终分数: {:.2}]", id, score);
    }

    println!("\n💡 凸组合优势:");
    println!("  - 分数归一化: 统一不同检索方法的分数范围");
    println!("  - 数学严谨: 保证融合分数在 [0, 1] 范围内");
    println!("  - 可解释性强: 清晰的权重分配");
    println!("  - 适应性好: 适用于分数范围差异大的场景");

    println!();
    Ok(())
}

