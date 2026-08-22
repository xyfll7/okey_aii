import type { UIMessage } from "@tanstack/ai/client";


export function getMessageText(message: UIMessage | { parts?: Array<{ type: string; content: unknown }> }) {
	return (message.parts ?? [])
		.map((part) => (part.type === "text" ? String(part.content ?? "") : ""))
}
