import type { UIMessage } from "@tanstack/ai/client";

/** Extract the concatenated text content from a UIMessage's text parts. */
export function getMessageText(message: UIMessage) {
	return message.parts
		.map((part) => (part.type === "text" ? part.content : ""))
}

