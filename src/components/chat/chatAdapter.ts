// mockAdapter.ts

import {
    EventType,
    type ModelMessage,
    type StreamChunk,
    type UIMessage,
} from "@tanstack/ai/client";
import { type ConnectionAdapter, stream } from "@tanstack/ai-react";
import { Channel, invoke } from "@tauri-apps/api/core";


type StreamEvent =
    | { event: "chunk"; data: { content: string } }
    | { event: "done"; data?: unknown }
    | { event: "error"; data: { message: string } };




interface UserTurn {
    raw: string;
    prompt: string;
}

export function extractLastUserTurn(
    messages: Array<UIMessage> | Array<ModelMessage>,
): UserTurn {
    for (let i = messages.length - 1; i >= 0; i--) {
        const m = messages[i];
        if (m.role !== "user") continue;

        let parts: Array<{ type: string; content: string }> = [];
        if ("content" in m && Array.isArray(m.content)) {
            parts = m.content.map((part) =>
                "text" in part
                    ? { type: "text", content: String(part.text) }
                    : (part as { type: string; content: string }),
            );
        } else if ("parts" in m && Array.isArray((m as UIMessage).parts)) {
            parts = (m as UIMessage).parts as Array<{ type: string; content: string }>;
        }

        const prompt = parts.length ? parts[parts.length - 1].content : "";
        const raw = parts.length > 1 ? parts[0].content : "";
        return { raw, prompt };
    }
    return { raw: "", prompt: "" };
}


interface ChatStreamState {
    finished: boolean;
    errored: boolean;
    aborted: boolean;
}


function startChatStream(
    turn: UserTurn,
    queue: StreamEvent[],
    state: ChatStreamState,
    notify: () => void,
): void {
    const channel = new Channel<StreamEvent>();

    channel.onmessage = (message) => {
        queue.push(message);

        if (message.event === "done" || message.event === "error") state.finished = true;
        notify();
    };
    void invoke("EVENT_NAMES.chat_stream", {
        raw: turn.raw,
        prompt: turn.prompt,
        on_event: channel,
    }).catch((err) => {
        if (state.aborted) return;
        state.finished = true;
        state.errored = true;
        queue.push({ event: "error", data: { message: String(err) } });
        notify();
    });
}

export function chatAdapter(): ConnectionAdapter {
    return stream(async function* (messages, _data, abortSignal) {
        const runId = `run-${Date.now()}`;
        const threadId = `thread-${Date.now()}`;
        const messageId = `msg-${Date.now()}`;
        const model = "backend-model";
        const now = () => Date.now();
        const userTurn = extractLastUserTurn(messages);
        const userText = userTurn.prompt;
   
      
        const queue: StreamEvent[] = [];
        const state: ChatStreamState = { finished: false, errored: false, aborted: false };
        let accumulated = "";
        let resolveNext: (() => void) | null = null;
        const notify = () => {
            resolveNext?.();
            resolveNext = null;
        };

        const onAbort = () => {
            state.aborted = true;
            state.finished = true;
            notify();
        };
        abortSignal?.addEventListener("abort", onAbort);
        try {
            yield {
                type: EventType.RUN_STARTED,
                runId,
                threadId,
                model,
                timestamp: now(),
            } satisfies StreamChunk;
            startChatStream(userTurn, queue, state, notify);
            yield {
                type: EventType.TEXT_MESSAGE_START,
                messageId,
                role: "assistant",
                model,
                timestamp: now(),
            } satisfies StreamChunk;


            while (!state.finished || queue.length > 0) {
                if (state.aborted || abortSignal?.aborted) {
                    void invoke("EVENT_NAMES.abort_chat_stream").catch(() => { });
                    return;
                }
                if (queue.length === 0) {
                    await new Promise<void>((resolve) => {
                        resolveNext = resolve;
                    });
                    continue;
                }
                const message = queue.shift();
                switch (message?.event) {
                    case "chunk": {
                        const delta = message.data?.content ?? "";
                        accumulated += delta;
                        yield {
                            type: EventType.TEXT_MESSAGE_CONTENT,
                            messageId,
                            delta,
                            model,
                            timestamp: now(),
                        } satisfies StreamChunk;
                        break;
                    }
                    case "error": {
                        state.errored = true;

                        yield {
                            type: EventType.TEXT_MESSAGE_END,
                            messageId,
                            model,
                            timestamp: now(),
                        } satisfies StreamChunk;
                        yield {
                            type: EventType.RUN_ERROR,
                            runId,
                            model,
                            timestamp: now(),
                            message: message.data?.message ?? "stream error",
                        } satisfies StreamChunk;
                        break;
                    }
                    default:
                        break;
                }
            }

            if (state.errored) return;

            yield {
                type: EventType.TEXT_MESSAGE_END,
                messageId,
                model,
                timestamp: now(),
            } satisfies StreamChunk;

            yield {
                type: EventType.RUN_FINISHED,
                runId,
                threadId,
                model,
                timestamp: now(),
                finishReason: "stop",
                usage: {
                    promptTokens: userText.length,
                    completionTokens: accumulated.length,
                    totalTokens: userText.length + accumulated.length,
                },
            } satisfies StreamChunk;
        } finally {
            abortSignal?.removeEventListener("abort", onAbort);
        }
    });
}
