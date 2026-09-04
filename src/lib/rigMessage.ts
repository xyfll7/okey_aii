export interface RigText {
	text: string;
	
	additional_params?: unknown;
}


export type RigDocumentSourceKind =
	| { type: "url"; value: string }
	| { type: "base64"; value: string }
	| { type: "fileId"; value: string }
	| { type: "raw"; value: number[] }
	| { type: "string"; value: string }
	| { type: "unknown" };


export interface RigImage {
	data: RigDocumentSourceKind;
	media_type?: string;
	detail?: string;
	additional_params?: unknown;
}


export interface RigAudio {
	data: RigDocumentSourceKind;
	media_type?: string;
	additional_params?: unknown;
}


export interface RigVideo {
	data: RigDocumentSourceKind;
	media_type?: string;
	additional_params?: unknown;
}


export interface RigDocument {
	data: RigDocumentSourceKind;
	media_type?: string;
	additional_params?: unknown;
}


export type RigToolResultContent =
	| (RigText & { type: "text" })
	| (RigImage & { type: "image" })
	| { type: "json"; value: unknown };


export interface RigToolResult {
	
	call: string;
	
	provider?: { call_id: string; item_id?: string };
	
	name: string;
	content: RigToolResultContent[];
}


export type RigUserContent =
	| (RigText & { type: "text" })
	| (RigToolResult & { type: "tool_result" })
	| (RigImage & { type: "image" })
	| (RigAudio & { type: "audio" })
	| (RigVideo & { type: "video" })
	| (RigDocument & { type: "document" });


export interface RigToolFunction {
	name: string;
	arguments: unknown;
}


export interface RigToolCall {
	type: "tool_call";
	
	id: string;
	
	provider?: { call_id: string; item_id?: string };
	function: RigToolFunction;
	signature?: string;
	additional_params?: unknown;
}


export interface RigReasoning {
	id?: string;
	content: Array<
		| { type: "text"; content: { text: string; signature?: string } }
		| { type: "encrypted"; content: string }
		| { type: "redacted"; content: { data: string } }
		| { type: "summary"; content: string }
	>;
}


export type RigAssistantContent =
	| (RigText & { type: "text" })
	| RigToolCall
	| (RigReasoning & { type: "reasoning" })
	| (RigImage & { type: "image" });


export type RigMessage =
	| { role: "system"; content: string }
	| { role: "user"; content: RigUserContent[] }
	| { role: "assistant"; id?: string; content: RigAssistantContent[] };


export interface RigHistoryItem {
	id: string;
	created_at: number;
	message: RigMessage;
}


export function rigMessageToText(message: RigMessage): string {
	switch (message.role) {
		case "system":
			return message.content;
		case "user":
			return message.content
				.map((c) =>
					c.type === "text" || c.type === "tool_result"
						? JSON.stringify(c)
						: `[$$${c.type}]`,
				)
				.join("\n");
		case "assistant":
			return message.content
				.map((c) => {
					switch (c.type) {
						case "text":
							return c.text;
						case "tool_call":
							return `[tool_call] $$${c.function.name}`;
						case "reasoning":
							return `[reasoning] $$${reasoningToText(c)}`;
						case "image":
							return "[image]";
						default:
							return `[$$${(c as { type: string }).type}]`;
					}
				})
				.join("\n");
	}
}

import type { UIMessage } from "@tanstack/ai-react";

/** Flattens a rig reasoning payload into readable plain text. */
function reasoningToText(reasoning: RigReasoning): string {
	const lines = (reasoning.content ?? []).map((block) => {
		switch (block.type) {
			case "text":
				return block.content.text;
			case "summary":
				return block.content;
			case "redacted":
				return block.content.data;
			case "encrypted":
				return "";
			default:
				return "";
		}
	});
	return lines.filter((s) => s?.trim()).join("\n");
}


export function rigMessageToUIMessage(item: RigHistoryItem): UIMessage {
	const m = item.message;
	switch (m.role) {
		case "system":
			return {
				id: item.id,
				role: "system",
				createdAt: new Date(item.created_at),
				parts: [{ type: "text", content: m.content }],
			};
		case "user": {
			const parts: UIMessage["parts"] = m.content.map((c) => {
				switch (c.type) {
					case "text":
						return { type: "text", content: c.text } as const;
					case "image":
						return {
							type: "image",
							source: { type: "url" as const, value: "" },
						} as const;
					case "tool_result":
						return { type: "text", content: JSON.stringify(c) } as const;
					default:
						return { type: "text", content: `[$$${c.type}]` } as const;
				}
			});
			return {
				id: item.id,
				role: "user",
				createdAt: new Date(item.created_at),
				parts,
			};
		}
		case "assistant": {
			const parts: UIMessage["parts"] = m.content.map((c) => {
				
				switch (c.type) {
					case "tool_call":
						return {
							type: "tool-call",
							id: c.id,
							name: c.function.name,
							arguments: JSON.stringify(c.function.arguments ?? {}),
							state: "complete" as const,
						} as const;
					case "reasoning":
						return {
							type: "thinking",
							content: reasoningToText(c),
						} as const;
					case "text":
						return { type: "text", content: c.text } as const;
					case "image":
						
						return { type: "text", content: "[image]" } as const;
					default:
						
						return {
							type: "text",
							content: `[$$${(c as { type: string }).type}]`,
						} as const;
				}
			});
			return {
				id: item.id,
				role: "assistant",
				createdAt: new Date(item.created_at),
				parts,
			};
		}
		default: {
			const _exhaustive: never = m;
			throw new Error(
				`Unexpected RigMessage role: $$${JSON.stringify(_exhaustive)}`,
			);
		}
	}
}
