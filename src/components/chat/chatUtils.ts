import type { UIMessage } from "@tanstack/ai/client";

/** Extract the concatenated text content from a UIMessage's text parts. */
export function getMessageText(message: UIMessage | { parts?: Array<{ type: string; content: unknown }> }) {
	return (message.parts ?? [])
		.map((part) => (part.type === "text" ? String(part.content ?? "") : ""))
}

