import { Thread } from '@/components/assistant-ui/thread';
import { LomusaiNetworkRuntimeProvider } from '@/services/network-runtime-provider';
import { ChatProps } from '@/types';
import { ToolFallback } from './tool-fallback';

export const NetworkChat = ({ agentId, memory }: ChatProps) => {
  return (
    <LomusaiNetworkRuntimeProvider agentId={agentId} memory={memory}>
      <Thread memory={memory} ToolFallback={ToolFallback} />
    </LomusaiNetworkRuntimeProvider>
  );
};
