import React, { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

interface ConditionNodeData {
  name: string;
  description?: string;
  inputs?: Array<{ id: string; name: string; type: string }>;
  outputs?: Array<{ id: string; name: string; type: string }>;
}

export const ConditionNode = memo(({ data, selected }: NodeProps<ConditionNodeData>) => {
  return (
    <div className={`px-4 py-2 shadow-md rounded-md bg-yellow-50 border-2 ${selected ? 'border-yellow-500' : 'border-yellow-200'}`}>
      <div className="flex flex-col">
        <div className="text-xs font-bold text-yellow-600">条件</div>
        <div className="text-sm font-medium">{data.name}</div>
        {data.description && (
          <div className="text-xs text-gray-500 mt-1">{data.description}</div>
        )}
      </div>

      {/* 输入处理 */}
      {data.inputs?.map((input, idx) => (
        <Handle
          key={input.id}
          type="target"
          position={Position.Left}
          id={input.id}
          className="w-2 h-2 bg-yellow-600"
          style={{ top: 10 + idx * 10 }}
        />
      ))}

      {/* 输出处理 - 根据输出名称显示不同文本 */}
      {data.outputs?.map((output, idx) => (
        <React.Fragment key={output.id}>
          <Handle
            type="source"
            position={Position.Right}
            id={output.id}
            className="w-2 h-2 bg-yellow-600"
            style={{ top: 10 + idx * 10 }}
          />
          <div 
            className="absolute text-xs font-semibold"
            style={{ 
              right: -5, 
              top: 4 + idx * 10, 
              transform: 'translateX(100%)',
              color: output.name.includes('真') ? 'green' : 'red'
            }}
          >
            {output.name}
          </div>
        </React.Fragment>
      ))}
    </div>
  );
}); 