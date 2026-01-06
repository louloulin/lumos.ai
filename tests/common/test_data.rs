use std::collections::HashMap;

/// 测试数据集
pub struct TestDataSets {
    pub small_documents: Vec<&'static str>,
    pub large_documents: Vec<String>,
    pub multilingual_content: HashMap<String, Vec<&'static str>>,
    pub edge_cases: Vec<&'static str>,
}

impl TestDataSets {
    pub fn load() -> Self {
        Self {
            small_documents: Self::get_small_documents(),
            large_documents: Self::generate_large_documents(),
            multilingual_content: Self::load_multilingual_data(),
            edge_cases: Self::get_edge_cases(),
        }
    }

    fn get_small_documents() -> Vec<&'static str> {
        vec![
            "Python is a high-level programming language.",
            "Machine learning is a subset of artificial intelligence.",
            "Deep learning uses neural networks with multiple layers.",
            "Natural language processing deals with text analysis.",
            "Computer vision enables machines to interpret visual information.",
            "Reinforcement learning learns through trial and error.",
            "Data science combines statistics and programming.",
            "Cloud computing provides on-demand computing resources.",
        ]
    }

    fn generate_large_documents() -> Vec<String> {
        vec![
            "Large document content ".repeat(1000),
            "Another large document with different content ".repeat(800),
            "Third large document for testing purposes ".repeat(1200),
        ]
    }

    fn load_multilingual_data() -> HashMap<String, Vec<&'static str>> {
        let mut data = HashMap::new();

        data.insert(
            "chinese".to_string(),
            vec![
                "人工智能是计算机科学的一个分支。",
                "机器学习是人工智能的一个子领域。",
                "深度学习使用多层神经网络。",
                "自然语言处理处理文本分析。",
            ],
        );

        data.insert(
            "english".to_string(),
            vec![
                "Artificial intelligence is a branch of computer science.",
                "Machine learning is a subfield of artificial intelligence.",
                "Deep learning uses multi-layer neural networks.",
                "Natural language processing handles text analysis.",
            ],
        );

        data.insert(
            "japanese".to_string(),
            vec![
                "人工知能はコンピュータサイエンスの一分野です。",
                "機械学習は人工知能のサブフィールドです。",
                "ディープラーニングは多層ニューラルネットワークを使用します。",
            ],
        );

        data
    }

    fn get_edge_cases() -> Vec<&'static str> {
        vec![
            "",                                             // 空文档
            "a",                                            // 单字符
            "🚀🎯🔥",                                       // 特殊字符
            "   ",                                          // 空白字符
            "\n\n\n",                                       // 换行符
            "A".repeat(10000).leak(),                       // 超长文档
            "Mixed 中文 English 日本語 content",            // 混合语言
            "Special chars: @#$%^&*()_+-=[]{}|;':\",./<>?", // 特殊符号
        ]
    }
}

/// 测试查询数据
pub struct TestQueries;

impl TestQueries {
    /// 获取测试查询集
    pub fn get_test_queries() -> Vec<(&'static str, Vec<usize>)> {
        vec![
            ("programming language", vec![0]),
            ("artificial intelligence", vec![1, 2]),
            ("neural networks", vec![2]),
            ("machine learning", vec![1]),
            ("text analysis", vec![3]),
            ("computer vision", vec![4]),
            ("data science", vec![6]),
            ("cloud computing", vec![7]),
        ]
    }

    /// 获取中文查询集
    pub fn get_chinese_queries() -> Vec<(&'static str, Vec<usize>)> {
        vec![
            ("人工智能", vec![0, 1]),
            ("机器学习", vec![1]),
            ("神经网络", vec![2]),
            ("文本分析", vec![3]),
        ]
    }

    /// 获取复杂查询集
    pub fn get_complex_queries() -> Vec<&'static str> {
        vec![
            "What is the relationship between machine learning and artificial intelligence?",
            "How do neural networks work in deep learning?",
            "Compare supervised and unsupervised learning methods",
            "Explain the applications of natural language processing",
            "What are the challenges in computer vision?",
        ]
    }
}

/// 性能测试数据
pub struct PerformanceTestData;

impl PerformanceTestData {
    /// 获取不同大小的文档集合
    pub fn get_document_sets() -> HashMap<String, Vec<String>> {
        let mut sets = HashMap::new();

        // 小文档集 (100个文档，每个100字符)
        sets.insert(
            "small".to_string(),
            (0..100)
                .map(|i| format!("Small document {} content ", i).repeat(10))
                .collect(),
        );

        // 中等文档集 (50个文档，每个1000字符)
        sets.insert(
            "medium".to_string(),
            (0..50)
                .map(|i| format!("Medium document {} content ", i).repeat(100))
                .collect(),
        );

        // 大文档集 (10个文档，每个10000字符)
        sets.insert(
            "large".to_string(),
            (0..10)
                .map(|i| format!("Large document {} content ", i).repeat(1000))
                .collect(),
        );

        sets
    }

    /// 获取并发测试查询
    pub fn get_concurrent_queries() -> Vec<String> {
        (0..100)
            .map(|i| format!("Concurrent query number {}", i))
            .collect()
    }
}

/// 错误测试数据
pub struct ErrorTestData;

impl ErrorTestData {
    /// 获取无效输入数据
    pub fn get_invalid_inputs() -> Vec<String> {
        vec![
            "".to_string(),   // 空输入
            "\0".to_string(), // 空字符
            "�".to_string(),  // 无效UTF-8
            "x".repeat(1000), // 大输入（减小大小避免内存问题）
        ]
    }

    /// 获取边界条件数据
    pub fn get_boundary_conditions() -> Vec<(String, String)> {
        vec![
            ("empty_string".to_string(), "".to_string()),
            ("single_char".to_string(), "a".to_string()),
            ("max_length".to_string(), "x".repeat(65536)),
            ("unicode_mixed".to_string(), "Hello 世界 🌍".to_string()),
        ]
    }
}
