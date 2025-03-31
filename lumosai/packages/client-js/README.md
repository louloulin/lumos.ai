# @lomusai/client-js

JavaScript client for the LumosAI API.

## Installation

```bash
npm install @lomusai/client-js
# or
yarn add @lomusai/client-js
# or
pnpm add @lomusai/client-js
```

## Basic Usage

```typescript
import { createClient } from '@lomusai/client-js';

// Create a client instance
const client = createClient({
  apiKey: 'your-api-key',
  // Optional: custom base URL
  // baseUrl: 'https://custom-api-endpoint.lumosai.com'
});

// Using an agent
const agent = client.getAgent('agent-id');

// Generate a response
const response = await agent.generate('What is the capital of France?');
console.log(response.message.content);

// Stream a response
const stream = await agent.stream('Tell me a story about robots');
stream.processDataStream({
  onTextPart: (text) => {
    process.stdout.write(text);
  },
  onErrorPart: (error) => {
    console.error(error);
  }
});

// Working with memory
const thread = await client.createMemoryThread({
  title: 'New Conversation',
  agentId: 'agent-id'
});

// Add a message to the thread
await thread.addMessage({
  role: 'user',
  content: 'Remember that my favorite color is blue'
});

// Start a workflow
const workflow = client.getWorkflow('workflow-id');
const { start } = workflow.createRun();
await start({ triggerData: { name: 'workflow-input' } });
```

## API Reference

### Client

- `createClient(config)` - Create a new client instance
- `client.getAgent(agentId)` - Get an agent by ID
- `client.getAgents()` - Get all available agents
- `client.getWorkflow(workflowId)` - Get a workflow by ID
- `client.getWorkflows()` - Get all available workflows
- `client.getMemoryThread(threadId, agentId?)` - Get a memory thread
- `client.createMemoryThread(params)` - Create a new memory thread
- `client.getMemoryThreads(params)` - Get all memory threads
- `client.saveMessageToMemory(params)` - Save messages to memory
- `client.getMemoryStatus(agentId?)` - Get memory status

### Agent

- `agent.details()` - Get agent details
- `agent.generate(input, options?)` - Generate a response
- `agent.stream(input, options?)` - Stream a response
- `agent.getTool(toolId)` - Get information about a tool
- `agent.evals()` - Get agent evaluations
- `agent.liveEvals()` - Get live evaluations

### Memory

- `thread.get()` - Get thread details
- `thread.update(params)` - Update thread properties
- `thread.delete()` - Delete a thread
- `thread.getMessages(params?)` - Get messages from the thread
- `thread.addMessage(message)` - Add a message to the thread

### Workflow

- `workflow.details()` - Get workflow details
- `workflow.createRun()` - Create a new run of the workflow
- `workflow.getRuns(params?)` - Get all runs of this workflow
- `workflowRun.start(params)` - Start the workflow run
- `workflowRun.details()` - Get run details
- `workflowRun.cancel()` - Cancel the run

## License

MIT 