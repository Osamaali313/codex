import { describe, expect, it } from "@jest/globals";

import { CodexExec } from "../src/exec";
import { Thread } from "../src/thread";

function fakeExec(lines: string[]): CodexExec {
  return {
    run: async function* () {
      for (const line of lines) {
        yield line;
      }
    },
  } as unknown as CodexExec;
}

describe("Thread.run error handling", () => {
  it("throws when the stream emits a fatal error event", async () => {
    const exec = fakeExec([
      JSON.stringify({ type: "thread.started", thread_id: "thread_1" }),
      JSON.stringify({ type: "error", message: "stream blew up" }),
    ]);
    const thread = new Thread(exec, {}, {});

    await expect(thread.run("hello")).rejects.toThrow("stream blew up");
  });

  it("returns the completed turn when no error is emitted", async () => {
    const exec = fakeExec([
      JSON.stringify({ type: "thread.started", thread_id: "thread_1" }),
      JSON.stringify({
        type: "item.completed",
        item: { id: "item_1", type: "agent_message", text: "Hi!" },
      }),
      JSON.stringify({
        type: "turn.completed",
        usage: {
          input_tokens: 1,
          cached_input_tokens: 0,
          output_tokens: 1,
          reasoning_output_tokens: 0,
        },
      }),
    ]);
    const thread = new Thread(exec, {}, {});

    const result = await thread.run("hello");
    expect(result.finalResponse).toBe("Hi!");
    expect(result.items).toHaveLength(1);
  });
});
