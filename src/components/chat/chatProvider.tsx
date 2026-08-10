import { useChat } from "@tanstack/ai-react"
import type { ReactNode } from "react"
import { chatAdapter } from "./chatAdapter"
import { ChatContext } from "./chatContext"

export function ChatProvider({ children }: { children: ReactNode }) {

    const chat = useChat({
        initialMessages: [],
        connection: chatAdapter(),
    })


    return (
        <ChatContext.Provider value={chat}>{children}</ChatContext.Provider>
    )
}


