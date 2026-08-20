import { useChat } from "@tanstack/ai-react"
import type { ReactNode } from "react"
import { chatAdapter } from "./chatAdapter"
import { ChatContext } from "./chatContext"

export function ChatProvider({ session_id, children }: { session_id: string; children: ReactNode }) {

    const chat = useChat({
        initialMessages: [],
        connection: chatAdapter(session_id),
    })


    return (
        <ChatContext.Provider value={chat}>{children}</ChatContext.Provider>
    )
}
