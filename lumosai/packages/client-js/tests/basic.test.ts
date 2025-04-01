import { expect, test, describe } from "bun:test";
import { createClient } from "../src";

describe("Basic functionality", () => {
  test("createClient should return a client instance", () => {
    const client = createClient({ apiKey: "test-api-key" });
    expect(client).toBeDefined();
  });
}); 