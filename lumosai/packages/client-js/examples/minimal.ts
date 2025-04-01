/**
 * Minimal example for using @lumosai/client-js
 */
import { createClient } from '../src';

// Create a client instance with your API key
const client = createClient({
  apiKey: Bun.env.LUMOSAI_API_KEY || 'your-api-key-here',
});

// Example: Using an agent
async function exampleAgentUsage() {
  try {
    // Get a specific agent by ID
    const agent = client.getAgent('agent-id');
    
    // Generate a response from the agent
    const response = await agent.generate('Hello, can you help me with a question?');
    console.log('Agent response:', response.message.content);
    
    // Or stream a response for real-time interaction
    const stream = await agent.stream('Tell me about artificial intelligence');
    await stream.processDataStream({
      onTextPart: (text) => {
        console.log('Agent is saying:', text);
      },
      onErrorPart: (error) => {
        console.error('Error:', error);
      }
    });
  } catch (error) {
    console.error('Error using agent:', error);
  }
}

// Example: Working with memory
async function exampleMemoryUsage() {
  try {
    // Create a new memory thread
    const thread = await client.createMemoryThread({
      title: 'Customer Support Conversation',
      agentId: 'support-agent-id',
    });
    console.log('Created thread:', await thread.get());
    
    // Add a message to the thread
    await thread.addMessage({
      role: 'user',
      content: 'I need help troubleshooting my device.',
    });
    
    // Get all messages in the thread
    const messages = await thread.getMessages();
    console.log('Thread messages:', messages);
  } catch (error) {
    console.error('Error using memory:', error);
  }
}

// Example: Working with workflows
async function exampleWorkflowUsage() {
  try {
    // Get a specific workflow by ID
    const workflow = client.getWorkflow('workflow-id');
    
    // Create and start a new workflow run
    const workflowRun = workflow.createRun();
    const result = await workflowRun.start({
      triggerData: {
        customerName: 'John Doe',
        requestType: 'Information Request',
      }
    });
    console.log('Workflow started:', result);
    
    // Get details about the run
    const runDetails = await workflowRun.details();
    console.log('Run details:', runDetails);
  } catch (error) {
    console.error('Error using workflow:', error);
  }
}

// Run examples (uncomment to use)
// exampleAgentUsage();
// exampleMemoryUsage();
// exampleWorkflowUsage(); 