import { EventType, type StreamChunk } from "@tanstack/ai/client";
import {
	type ConnectionAdapter,
	stream,
	type UIMessage,
} from "@tanstack/ai-react";
import { Channel, invoke } from "@tauri-apps/api/core";
import type {
	RigHistoryItem,
	RigMessage,
	RigUserContent,
} from "#/lib/rigMessage";

type StreamEvent =
	| { event: "chunk"; data: { content: string } }
	| { event: "done"; data?: unknown }
	| { event: "error"; data: { message: string } };


type ChatEventWire =
	| { type: "TextDelta"; data: string }
	| { type: "ToolCall"; data: { name: string; arguments: unknown } }
	| { type: "ToolCallDelta"; data: string }
	| { type: "Reasoning"; data: string }
	| { type: "Done"; data: null };

interface ChatStreamState {
	finished: boolean;
	errored: boolean;
	aborted: boolean;
}


function buildPromptHistoryItem(userMessage: UIMessage): RigHistoryItem {
	const content: RigUserContent[] = (userMessage.parts ?? [])
		.filter(
			(p): p is Extract<UIMessage["parts"][number], { type: "text" }> =>
				p.type === "text",
		)
		.map((p) => ({ type: "text", text: p.content }));
	const message: RigMessage = {
		role: "user",
		content,
	};
	return {
		id: userMessage.id ?? `prompt-$$${Date.now()}`,
		created_at: Date.now(),
		message,
	};
}

function startChatStream(
	queue: StreamEvent[],
	state: ChatStreamState,
	notify: () => void,
	session_id: string,
	userMessage: UIMessage,
): void {
	const prompt = buildPromptHistoryItem(userMessage);
	const channel = new Channel<ChatEventWire>();

	channel.onmessage = (message) => {
		switch (message.type) {
			case "TextDelta":
				if (message.data) {
					queue.push({ event: "chunk", data: { content: message.data } });
				}
				break;
			case "Done":
				queue.push({ event: "done" });
				state.finished = true;
				break;
			default:
				
				break;
		}
		notify();
	};
	void (async () => {
		try {
			await invoke("send_message", {
				on_event: channel,
				prompt,
				session_id,
			});
		} catch (err) {
			if (state.aborted) return;
			state.finished = true;
			state.errored = true;
			queue.push({ event: "error", data: { message: String(err) } });
			notify();
		}
	})();
}

export function chatAdapter(session_id: string): ConnectionAdapter {
	return stream(async function* (messages, _, abortSignal) {
		const runId = `run-$$${Date.now()}`;
		const threadId = `thread-$$${Date.now()}`;
		const messageId = `msg-$$${Date.now()}`;
		const model = "backend-model";
		const now = () => Date.now();
		const message = messages.at(-1);
		if (message?.role !== "user") {
			return;
		}
		const queue: StreamEvent[] = [];
		const state: ChatStreamState = {
			finished: false,
			errored: false,
			aborted: false,
		};
		let accumulated = "";
		let resolveNext: (() => void) | null = null;
		const notify = () => {
			resolveNext?.();
			resolveNext = null;
		};

		
		const onAbort = () => {
			state.aborted = true;
			state.finished = true;
			void invoke("stop_generation", { session_id })
				.catch((err) => {
					
					
					console.warn("stop_generation:", String(err));
				});
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
			startChatStream(queue, state, notify, session_id, message as UIMessage);
			yield {
				type: EventType.TEXT_MESSAGE_START,
				messageId,
				role: "assistant",
				model,
				timestamp: now(),
			} satisfies StreamChunk;

			while (!state.finished || queue.length > 0) {
				if (state.aborted || abortSignal?.aborted) {
					
					
					return;
				}
				if (queue.length === 0) {
					await new Promise<void>((resolve) => {
						resolveNext = resolve;
					});
					continue;
				}
				const event = queue.shift();
				switch (event?.event) {
					case "chunk": {
						const delta = event.data?.content ?? "";
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
							message: event.data?.message ?? "stream error",
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
					promptTokens: 0,
					completionTokens: accumulated.length,
					totalTokens: 0,
				},
			} satisfies StreamChunk;
		} finally {
			abortSignal?.removeEventListener("abort", onAbort);
		}
	});
}
