#!/bin/bash
echo "=== LumosAI 深度功能分析 ==="
echo ""

echo "【Agent 模块】"
ls -1 lumosai_core/src/agent/*.rs 2>/dev/null | xargs -I {} basename {} .rs

echo ""
echo "【LLM 提供商】"
ls -1 lumosai_core/src/llm/providers/ 2>/dev/null | head -20

echo ""
echo "【工具系统】"
ls -1 lumosai_core/src/tool/*.rs 2>/dev/null | xargs -I {} basename {} .rs

echo ""
echo "【Workflow 模块】"
ls -1 lumosai_core/src/workflow/*.rs 2>/dev/null | xargs -I {} basename {} .rs

echo ""
echo "【Memory 模块】"
ls -1 lumosai_core/src/memory/*.rs 2>/dev/null | xargs -I {} basename {} .rs

echo ""
echo "【RAG 模块】"
ls -1 lumosai_rag/src/*.rs 2>/dev/null | xargs -I {} basename {} .rs | head -10

echo ""
echo "【向量数据库支持】"
ls -d lumosai_vector/*/ 2>/dev/null | xargs -I {} basename {}
