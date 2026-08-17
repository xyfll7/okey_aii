// 前端镜像 rig-core 的 `rig::message::Message` serde 序列化结构。
// 对齐 src-tauri/src/ai/state.rs 中 HistoryItem 与 commands.rs 里 get_history 返回的 Vec<HistoryItem>。
//
// 序列化约定 (来自 rig-core 0.41.0):
//   - Message:      #[serde(tag = "role", rename_all = "lowercase")]
//   - UserContent:  #[serde(tag = "type", rename_all = "lowercase")]
//   - AssistantContent: #[serde(untagged)]  (Text 含 `text` 字段, ToolCall 含 `id`+`function` ...)
//   - OneOrMany<T>: 始终序列化为数组 (Vec<T>)
//   - Text:         { text: string } (+ 可能被 provider 展平的额外字段)
//
// HistoryItem 是后端额外包装的一层(见 state.rs):
//   - id:         唯一 id,前端用于渲染 key 与按 id 删除
//   - created_at: 创建时间(毫秒时间戳)
//   - message:    实际的 RigMessage

/** 基础文本内容 (rig::message::Text)。 */
export interface RigText {
	text: string;
	/** Provider 特定的额外字段 (如引用元数据),展平在 text 同层,可选。 */
	[key: string]: unknown;
}

/** 文档/图片/音频/视频的源数据 (rig::message::DocumentSourceKind)。 */
export type RigDocumentSourceKind =
	| { type: "url"; value: string }
	| { type: "base64"; value: string }
	| { type: "fileId"; value: string }
	| { type: "raw"; value: number[] }
	| { type: "string"; value: string }
	| { type: "unknown" };

/** 图片内容 (rig::message::Image)。 */
export interface RigImage {
	data: RigDocumentSourceKind;
	media_type?: string;
	detail?: string;
	[key: string]: unknown;
}

/** 音频内容 (rig::message::Audio)。 */
export interface RigAudio {
	data: RigDocumentSourceKind;
	media_type?: string;
	[key: string]: unknown;
}

/** 视频内容 (rig::message::Video)。 */
export interface RigVideo {
	data: RigDocumentSourceKind;
	media_type?: string;
	[key: string]: unknown;
}

/** 文档内容 (rig::message::Document)。 */
export interface RigDocument {
	data: RigDocumentSourceKind;
	media_type?: string;
	[key: string]: unknown;
}

/** 工具结果内容块 (rig::message::ToolResultContent)。 */
export type RigToolResultContent =
	| (RigText & { type: "text" })
	| (RigImage & { type: "image" })
	| { type: "json"; value: unknown };

/** 工具结果 (rig::message::ToolResult)。 */
export interface RigToolResult {
	id: string;
	call_id?: string;
	content: RigToolResultContent[];
}

/** 用户消息内容 (rig::message::UserContent),按 type 标签区分。 */
export type RigUserContent =
	| (RigText & { type: "text" })
	| (RigToolResult & { type: "tool_result" })
	| (RigImage & { type: "image" })
	| (RigAudio & { type: "audio" })
	| (RigVideo & { type: "video" })
	| (RigDocument & { type: "document" });

/** 助手工具调用函数 (rig::message::ToolFunction)。 */
export interface RigToolFunction {
	name: string;
	arguments: unknown;
}

/** 助手工具调用 (rig::message::ToolCall)。 */
export interface RigToolCall {
	id: string;
	call_id?: string;
	function: RigToolFunction;
	signature?: string;
	additional_params?: unknown;
}

/** 助手结构化推理 (rig::message::Reasoning)。 */
export interface RigReasoning {
	id?: string;
	content: Array<
		| { type: "text"; text: string; signature?: string }
		| { type: "encrypted"; content: string }
		| { type: "redacted"; data: string }
		| { type: "summary"; content: string }
	>;
}

/**
 * 助手消息内容 (rig::message::AssistantContent)。
 * 注意: serde 为 untagged,无 `type` 标签。
 * 通过是否存在 `text` / `function` / `content`(reasoning) 字段来区分。
 */
export type RigAssistantContent =
	| RigText
	| RigToolCall
	| RigReasoning
	| RigImage;

/** 顶层消息 (rig::message::Message),按 role 标签区分。 */
export type RigMessage =
	| { role: "system"; content: string }
	| { role: "user"; content: RigUserContent[] }
	| { role: "assistant"; id?: string; content: RigAssistantContent[] };

/**
 * 后端 `state.rs` 中 `HistoryItem` 的镜像:
 * 包装一条 RigMessage,额外携带 `id`(唯一标识、渲染 key、按 id 删除用)
 * 与 `created_at`(创建时间,毫秒时间戳)。
 * `get_history` 返回的就是 `HistoryItem[]`。
 */
export interface RigHistoryItem {
	id: string;
	created_at: number;
	message: RigMessage;
}

/** 从一条 Message 中提取纯文本,便于 UI 展示。 */
export function rigMessageToText(message: RigMessage): string {
	switch (message.role) {
		case "system":
			return message.content;
		case "user":
			return message.content
				.map((c) =>
					c.type === "text" || c.type === "tool_result"
						? JSON.stringify(c)
						: `[${c.type}]`,
				)
				.join("\n");
		case "assistant":
			return message.content
				.map((c) => {
					if ("text" in c) return c.text;
					if ("function" in c && !("data" in c)) {
						return `[tool_call] ${(c as RigToolCall).function.name}`;
					}
					if ("content" in c && !("data" in c)) {
						return `[reasoning] ${JSON.stringify((c as RigReasoning).content)}`;
					}
					return "[image]";
				})
				.join("\n");
	}
}

import type { UIMessage } from "@tanstack/ai-react";

/**
 * 把一条 HistoryItem 转成 useChat 所需的 UIMessage[]。
 * 使用 HistoryItem 自带的 `id` 作为 UIMessage 的 id(渲染 key 与删除用),
 * `created_at` 可传给 UIMessage 作为 createdAt。content 按类型映射为
 * UIMessage 的 parts(text / image / ...)。
 */
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
						return { type: "text", content: `[${c.type}]` } as const;
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
				// 注意: RigText / RigImage 都带 `[key: string]: unknown` 索引签名,
				// 所以不能用 `"text" in c` 区分文本与图片。需要用正文字段判别。
				if ("function" in c) {
					const tc = c as RigToolCall;
					return {
						type: "tool-call",
						id: tc.id,
						name: tc.function.name,
						arguments: JSON.stringify(tc.function.arguments ?? {}),
						state: "complete" as const,
					} as const;
				}
				if ("content" in c && !("data" in c)) {
					return {
						type: "thinking",
						content: JSON.stringify((c as RigReasoning).content),
					} as const;
				}
				if ("text" in c && !("data" in c)) {
					return { type: "text", content: (c as RigText).text } as const;
				}
				// 其余(图片等):用占位文本,避免丢失消息。
				return { type: "text", content: "[image]" } as const;
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
				`Unexpected RigMessage role: ${JSON.stringify(_exhaustive)}`,
			);
		}
	}
}
