import React, { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

interface ToolNodeData {
  name: string;
  description?: string;
  config?: {
    toolName: string;
    params: Record<string, any>;
  };
  inputs?: Array<{ id: string; name: string; type: string }>;
  outputs?: Array<{ id: string; name: string; type: string }>;
}

export const ToolNode = memo(({ data, selected }: NodeProps<ToolNodeData>) => {
  return (
    <div className={`px-4 py-2 shadow-md rounded-md bg-orange-50 border-2 ${selected ? 'border-orange-500' : 'border-orange-200'}`}>
      <div className="flex flex-col">
        <div className="text-xs font-bold text-orange-500">工具</div>
        <div className="text-sm font-medium">{data.name}</div>
        {data.description && (
          <div className="text-xs text-gray-500 mt-1">{data.description}</div>
        )}
        {data.config?.toolName && (
          <div className="text-xs mt-1 bg-orange-100 p-1 rounded">
            工具: {data.config.toolName}
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
          className="w-2 h-2 bg-orange-500"
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
          className="w-2 h-2 bg-orange-500"
          style={{ top: 10 + idx * 10 }}
        />
      ))}
    </div>
  );
}); 