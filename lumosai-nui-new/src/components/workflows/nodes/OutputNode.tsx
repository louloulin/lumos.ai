import React, { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

interface OutputNodeData {
  name: string;
  inputs?: Array<{ id: string; name: string; type: string }>;
}

export const OutputNode = memo(({ data, selected }: NodeProps<OutputNodeData>) => {
  return (
    <div className={`px-4 py-2 shadow-md rounded-md bg-green-50 border-2 ${selected ? 'border-green-500' : 'border-green-200'}`}>
      <div className="flex items-center">
        <div className="ml-2">
          <div className="text-xs font-bold text-green-500">输出</div>
          <div className="text-sm font-medium">{data.name}</div>
        </div>
      </div>

      {/* 输入处理 */}
      {data.inputs?.map((input) => (
        <Handle
          key={input.id}
          type="target"
          position={Position.Left}
          id={input.id}
          className="w-2 h-2 bg-green-500"
        />
      ))}
    </div>
  );
}); 