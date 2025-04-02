import React, { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

interface LLMNodeData {
  name: string;
  description?: string;
  config?: {
    model: string;
    maxTokens: number;
    temperature: number;
  };
  inputs?: Array<{ id: string; name: string; type: string }>;
  outputs?: Array<{ id: string; name: string; type: string }>;
}

export const LLMNode = memo(({ data, selected }: NodeProps<LLMNodeData>) => {
  return (
    <div className={`px-4 py-2 shadow-md rounded-md bg-purple-50 border-2 ${selected ? 'border-purple-500' : 'border-purple-200'}`}>
      <div className="flex flex-col">
        <div className="text-xs font-bold text-purple-500">LLM</div>
        <div className="text-sm font-medium">{data.name}</div>
        {data.description && (
          <div className="text-xs text-gray-500 mt-1">{data.description}</div>
        )}
        {data.config && (
          <div className="text-xs mt-1 bg-purple-100 p-1 rounded">
            <div>模型: {data.config.model}</div>
            <div>温度: {data.config.temperature}</div>
          </div>
        )}
      </div>

      {/* 输入处理 */}
      {data.inputs?.map((input, idx) => (
        <Handle
          key={input.id}
          type="target"
          position={Position.Left}
          id={input.id}
          className="w-2 h-2 bg-purple-500"
          style={{ top: 10 + idx * 10 }}
        />
      ))}

      {/* 输出处理 */}
      {data.outputs?.map((output, idx) => (
        <Handle
          key={output.id}
          type="source"
          position={Position.Right}
          id={output.id}
          className="w-2 h-2 bg-purple-500"
          style={{ top: 10 + idx * 10 }}
        />
      ))}
    </div>
  );
}); 