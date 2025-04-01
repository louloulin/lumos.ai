import { describe, expect, it, mock, spyOn } from "bun:test";
import { createClient, LumosAIClient } from "../src";

describe("LumosAIClient", () => {
  // 模拟 fetch 请求
  const originalFetch = globalThis.fetch;
  
  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it("should create a client instance", () => {
    const client = createClient({ apiKey: "test-key" });
    expect(client).toBeInstanceOf(LumosAIClient);
  });

  it("should use the default API URL if not specified", () => {
    const client = createClient({ apiKey: "test-key" });
    // @ts-ignore - 使用私有属性进行测试
    expect(client.baseUrl).toBe("https://api.lumosai.com");
  });

  it("should use a custom API URL if specified", () => {
    const client = createClient({ 
      apiKey: "test-key",
      baseUrl: "https://custom.api.lumosai.com" 
    });
    // @ts-ignore - 使用私有属性进行测试
    expect(client.baseUrl).toBe("https://custom.api.lumosai.com");
  });

  it("should include API key in headers", () => {
    const client = createClient({ apiKey: "test-key" });
    // @ts-ignore - 使用私有属性进行测试
    expect(client.headers["Authorization"]).toBe("Bearer test-key");
  });

  it("should get an agent by ID", () => {
    const client = createClient({ apiKey: "test-key" });
    const agent = client.getAgent("agent-id");
    expect(agent).toBeDefined();
    // @ts-ignore - 使用私有属性进行测试
    expect(agent.agentId).toBe("agent-id");
  });

  it("should get all agents", async () => {
    // 模拟成功的响应
    globalThis.fetch = mock(() => {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({
          agents: [
            { id: "agent-1", name: "Agent 1" },
            { id: "agent-2", name: "Agent 2" }
          ]
        })
      });
    });

    const client = createClient({ apiKey: "test-key" });
    const agents = await client.getAgents();
    
    expect(agents.length).toBe(2);
    // @ts-ignore - 使用私有属性进行测试
    expect(agents[0].agentId).toBe("agent-1");
    // @ts-ignore - 使用私有属性进行测试
    expect(agents[1].agentId).toBe("agent-2");
  });

  it("should handle errors when getting agents", async () => {
    // 模拟失败的响应
    globalThis.fetch = mock(() => {
      return Promise.resolve({
        ok: false,
        statusText: "Unauthorized"
      });
    });

    const client = createClient({ apiKey: "test-key" });
    
    await expect(async () => {
      await client.getAgents();
    }).toThrow("Failed to get agents: Unauthorized");
  });

  it("should create a memory thread", async () => {
    // 模拟成功的响应
    globalThis.fetch = mock(() => {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({
          id: "thread-id",
          title: "Test Thread"
        })
      });
    });

    const client = createClient({ apiKey: "test-key" });
    const thread = await client.createMemoryThread({
      title: "Test Thread",
      agentId: "agent-id"
    });
    
    // @ts-ignore - 使用私有属性进行测试
    expect(thread.threadId).toBe("thread-id");
  });

  it("should get a workflow by ID", () => {
    const client = createClient({ apiKey: "test-key" });
    const workflow = client.getWorkflow("workflow-id");
    expect(workflow).toBeDefined();
    // @ts-ignore - 使用私有属性进行测试
    expect(workflow.workflowId).toBe("workflow-id");
  });
}); 