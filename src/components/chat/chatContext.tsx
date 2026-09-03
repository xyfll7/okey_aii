import type { useChat } from "@tanstack/ai-react"
import { createContext, useContext } from "react"

export type ChatContextValue = ReturnType<typeof useChat> & { session_id: string }

/** The app-level `append` exposed by <ChatProvider>, e.g. for one-shot auto-send after the chat becomes ready. */
export type ChatAppend = ChatContextValue["append"]

export const ChatContext = createContext<ChatContextValue | null>(null)

export function useChatContext() {
	const ctx = useContext(ChatContext)
	if (!ctx) {
		throw new Error("useChatContext must be used within a <ChatProvider>")
	}
	return ctx
}
