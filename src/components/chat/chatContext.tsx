import type { useChat } from "@tanstack/ai-react"
import { createContext, useContext } from "react"

export type ChatContextValue = ReturnType<typeof useChat> & { session_id: string }

export const ChatContext = createContext<ChatContextValue | null>(null)

export function useChatContext() {
	const ctx = useContext(ChatContext)
	if (!ctx) {
		throw new Error("useChatContext must be used within a <ChatProvider>")
	}
	return ctx
}
