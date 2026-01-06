#!/bin/bash
echo "=== 差距分析报告 ==="
echo ""

echo "【1. 待实现/不完整功能】"
echo ""
echo "RAG 系统："
echo "- Vector Search: $(grep -r "vector_search\|similarity_search" lumosai_rag/src/ 2>/dev/null | wc -l) 实现"
echo "- Hybrid Search: $(grep -r "HybridSearch\|hybrid_search" lumosai_rag/src/ 2>/dev/null | wc -l) 实现"
echo "- Reranking: $(grep -r "rerank\|Reranker" lumosai_rag/src/ 2>/dev/null | wc -l) 实现"

echo ""
echo "流式处理："
echo "- Agent Stream: $(grep -r "impl.*Stream" lumosai_core/src/agent/streaming.rs 2>/dev/null | wc -l) 实现"
echo "- Tool Stream: $(grep -r "stream" lumosai_core/src/tool/*.rs 2>/dev/null | wc -l) 提及"

echo ""
echo "结构化输出："
echo "- Structured Output Impl: $(grep -r "impl.*StructuredOutput" lumosai_core/src/agent/ 2>/dev/null | wc -l) 实现"
echo "- JSON Schema: $(grep -r "JsonSchema\|json_schema" lumosai_core/src/ 2>/dev/null | wc -l) 使用"

echo ""
echo "【2. 部署支持】"
echo "- Dockerfile: $(ls Dockerfile 2>/dev/null && echo "✅" || echo "❌")"
echo "- docker-compose.yml: $(ls docker-compose*.yml 2>/dev/null | wc -l) 个"
echo "- Kubernetes manifests: $(find . -name "*.yaml" -o -name "*.yml" | grep -i "k8s\|kubernetes" 2>/dev/null | wc -l) 个"
echo "- Helm charts: $(ls -d helm 2>/dev/null && echo "✅" || echo "❌")"

echo ""
echo "【3. 性能基准】"
echo "- Benchmark files: $(find . -path "*/benches/*.rs" 2>/dev/null | wc -l) 个"
echo "- Performance tests: $(grep -r "#\[bench\]" . 2>/dev/null | wc -l) 个"

echo ""
echo "【4. 示例和教程】"
echo "- Example files: $(find lumosai_examples/examples -name "*.rs" 2>/dev/null | wc -l) 个"
echo "- Tutorial docs: $(find docs -name "*tutorial*" -o -name "*guide*" 2>/dev/null | wc -l) 个"
echo "- Real-world examples: $(find docs/examples -type d 2>/dev/null | wc -l) 个"

echo ""
echo "【5. 企业功能完整度】"
for dir in lumosai_auth lumosai_enterprise lumosai_security; do
    if [ -d "$dir/src" ]; then
        echo "- $dir: $(find $dir/src -name "*.rs" 2>/dev/null | wc -l) 文件"
    fi
done

echo ""
echo "【6. 集成测试】"
echo "- Integration tests: $(find . -path "*/tests/*integration*.rs" 2>/dev/null | wc -l) 个"
echo "- E2E tests: $(find . -path "*/tests/*e2e*.rs" -o -path "*/tests/*end_to_end*.rs" 2>/dev/null | wc -l) 个"

echo ""
echo "【7. CI/CD】"
echo "- GitHub Actions: $(ls .github/workflows/*.yml 2>/dev/null | wc -l) 个"
echo "- GitLab CI: $(ls .gitlab-ci.yml 2>/dev/null && echo "✅" || echo "❌")"
