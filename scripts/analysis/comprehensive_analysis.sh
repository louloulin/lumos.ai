#!/bin/bash
echo "=== LumosAI 核心功能完整性分析 ==="
echo ""

echo "1. Agent 核心功能检查："
echo "   - Builder API: $(grep -r "AgentBuilder" lumosai_core/src/agent/ | wc -l) 处"
echo "   - Tool Integration: $(grep -r "add_tool\|register_tool" lumosai_core/src/agent/ | wc -l) 处"
echo "   - Memory Management: $(grep -r "WorkingMemory\|SemanticMemory" lumosai_core/src/agent/ | wc -l) 处"
echo "   - Streaming Support: $(grep -r "generate_stream\|StreamExt" lumosai_core/src/agent/ | wc -l) 处"

echo ""
echo "2. Multi-Agent 功能："
echo "   - Collaboration: $(ls lumosai_core/src/agent/collaboration.rs 2>/dev/null && echo "✅" || echo "❌")"
echo "   - DAG Orchestration: $(ls lumosai_core/src/agent/dag_orchestration.rs 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Chain: $(ls lumosai_core/src/agent/chain.rs 2>/dev/null && echo "✅" || echo "❌")"

echo ""
echo "3. LLM 提供商支持："
grep -r "impl LlmProvider for" lumosai_core/src/llm/ 2>/dev/null | grep -o "impl LlmProvider for [A-Za-z]*" | cut -d' ' -f4 | sort -u

echo ""
echo "4. 工具系统："
echo "   - Tool Trait: $(grep -c "trait Tool" lumosai_core/src/tool/tool.rs 2>/dev/null)"
echo "   - Tool Builder: $(grep -c "ToolBuilder" lumosai_core/src/tool/builder.rs 2>/dev/null)"
echo "   - Tool Registry: $(grep -c "ToolRegistry" lumosai_core/src/tool/registry.rs 2>/dev/null)"

echo ""
echo "5. Workflow 功能："
echo "   - DAG Workflow: $(ls lumosai_core/src/workflow/dag_workflow.rs 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Enhanced Workflow: $(ls lumosai_core/src/workflow/enhanced.rs 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Execution Engine: $(ls lumosai_core/src/workflow/execution_engine.rs 2>/dev/null && echo "✅" || echo "❌")"

echo ""
echo "6. Memory 系统："
ls -1 lumosai_core/src/memory/*.rs 2>/dev/null | xargs -I {} basename {} .rs | grep -v "mod\|tests"

echo ""
echo "7. RAG 功能："
echo "   - Document Processing: $(grep -r "ChunkingStrategy\|DocumentProcessor" lumosai_rag/src/ 2>/dev/null | wc -l) 处"
echo "   - Vector Search: $(grep -r "vector_search\|similarity_search" lumosai_rag/src/ 2>/dev/null | wc -l) 处"
echo "   - Retrieval: $(grep -r "retrieve\|Retriever" lumosai_rag/src/ 2>/dev/null | wc -l) 处"

echo ""
echo "8. 企业级功能："
echo "   - Auth: $(ls -d lumosai_auth 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Security: $(ls -d lumosai_security 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Telemetry: $(ls -d lumosai_telemetry 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Enterprise: $(ls -d lumosai_enterprise 2>/dev/null && echo "✅" || echo "❌")"

echo ""
echo "9. 集成能力："
echo "   - Python Bindings: $(ls -d lumosai_bindings/python 2>/dev/null && echo "✅" || echo "❌")"
echo "   - JS Bindings: $(ls -d lumosai_bindings/npm 2>/dev/null && echo "✅" || echo "❌")"
echo "   - MCP: $(ls -d lumosai_mcp 2>/dev/null && echo "✅" || echo "❌")"
echo "   - Cloud: $(ls -d lumosai_cloud 2>/dev/null && echo "✅" || echo "❌")"

echo ""
echo "10. UI/UX："
echo "   - UI Package: $(ls -d lumosai_ui 2>/dev/null && echo "✅" || echo "❌")"
echo "   - CLI: $(ls -d lumosai_cli 2>/dev/null && echo "✅" || echo "❌")"
