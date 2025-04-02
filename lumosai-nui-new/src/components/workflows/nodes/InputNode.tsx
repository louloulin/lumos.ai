import React, { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';

interface InputNodeData {
  name: string;
  outputs?: Array<{ id: string; name: string; type: string }>;
}

export const InputNode = memo(({ data, selected }: NodeProps<InputNodeData>) => {
  return (
    <div className={`px-4 py-2 shadow-md rounded-md bg-blue-50 border-2 ${selected ? 'border-blue-500' : 'border-blue-200'}`}>
      <div className="flex items-center">
        <div className="ml-2">
          <div className="text-xs font-bold text-blue-500">输入</div>
          <div className="text-sm font-medium">{data.name}</div>
        </div>
      </div>

      {/* 输出处理 */}
      {data.outputs?.map((output) => (
        <Handle
          key={output.id}
          type="source"
          position={Position.Right}
          id={output.id}
          className="w-2 h-2 bg-blue-500"
        />
      ))}
    </div>
  );
}); 