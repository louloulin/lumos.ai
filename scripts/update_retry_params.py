#!/usr/bin/env python3
"""
批量更新测试中的重试参数
将所有 500ms 延迟改为 1000ms，3 次重试改为 5 次，1000ms 初始延迟改为 2000ms
"""

import re
import sys

def update_retry_params(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 替换模式 1: tokio::time::sleep(Duration::from_millis(500))
    content = re.sub(
        r'tokio::time::sleep\(Duration::from_millis\(500\)\)',
        'tokio::time::sleep(Duration::from_millis(1000))',
        content
    )
    
    # 替换模式 2: retry_with_backoff 的参数 (3, 1000) -> (5, 2000)
    content = re.sub(
        r'retry_with_backoff\(\s*\|\| async \{ agent\.generate\(&messages, &options\)\.await \},\s*3,\s*1000,',
        'retry_with_backoff(\n            || async { agent.generate(&messages, &options).await },\n            5,\n            2000,',
        content
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
    
    print(f"✅ Updated: {file_path}")

if __name__ == "__main__":
    files = [
        "lumosai_core/src/agent/week1_agent_tests.rs",
        "lumosai_core/src/agent/plan4_api_tests.rs",
        "lumosai_core/src/agent/operators.rs",
    ]
    
    for file_path in files:
        try:
            update_retry_params(file_path)
        except Exception as e:
            print(f"❌ Error updating {file_path}: {e}", file=sys.stderr)
    
    print("\n🎉 All files updated!")

