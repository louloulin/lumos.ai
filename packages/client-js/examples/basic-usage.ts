/**
 * Basic usage examples for @lumosai/client-js
 * 
 * Note: This example assumes you have set the LUMOSAI_API_KEY environment variable
 * or you'll need to replace 'your-api-key' with an actual API key.
 */
import { createClient } from '../src';

async function main() {
  // Initialize the client
  const client = createClient({
    apiKey: Bun.env.LUMOSAI_API_KEY || 'your-api-key',
  });

  try {
    // Get all agents
    console.log('Fetching all agents...');
    const agents = await client.getAgents();
    console.log(`Found ${agents.length} agents`);

    if (agents.length > 0) {
      // Use the first agent
      const agent = agents[0];
      const agentDetails = await agent.details();
      console.log(`Using agent: ${agentDetails.id}`);

      // Generate a response
      console.log('\nGenerating response...');
      const response = await agent.generate('What is artificial intelligence?');
      console.log(`Response: ${response.message.content}`);

      // Create a memory thread
      console.log('\nCreating memory thread...');
      const thread = await client.createMemoryThread({
        title: 'AI Discussion',
        agentId: agentDetails.id,
      });
      const threadDetails = await thread.get();
      console.log(`Thread created with ID: ${threadDetails.id}`);

      // Add a message to the thread
      console.log('\nAdding message to thread...');
      await thread.addMessage({
        role: 'user',
        content: 'Remember that I am interested in machine learning.',
      });
      console.log('Message added');

      // Get all messages in the thread
      console.log('\nFetching messages from thread...');
      const messages = await thread.getMessages();
      console.log(`Thread has ${messages.messages.length} messages`);
    }

    // Get all workflows
    console.log('\nFetching all workflows...');
    const workflows = await client.getWorkflows();
    console.log(`Found ${workflows.length} workflows`);

    if (workflows.length > 0) {
      // Use the first workflow
      const workflow = workflows[0];
      const workflowDetails = await workflow.details();
      console.log(`Using workflow: ${workflowDetails.id}`);

      // Start a workflow run
      console.log('\nStarting workflow run...');
      const workflowRun = workflow.createRun();
      const runResult = await workflowRun.start({ 
        triggerData: { key: 'value' } 
      });
      console.log('Workflow run started:', runResult);
    }

  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the example
main(); 