import { Thread } from '@/components/assistant-ui/thread';

import { LomusaiRuntimeProvider } from '@/services/mastra-runtime-provider';
import { ChatProps } from '@/types';

export const AgentChat = ({
  agentId,
  agentName,
  threadId,
  initialMessages,
  memory,
  baseUrl,
  refreshThreadList,
}: ChatProps) => {
  return (
    <LomusaiRuntimeProvider
      agentId={agentId}
      agentName={agentName}
      threadId={threadId}
      initialMessages={initialMessages}
      memory={memory}
      baseUrl={baseUrl}
      refreshThreadList={refreshThreadList}
    >
      <Thread memory={memory} />
    </LomusaiRuntimeProvider>
  );
};
