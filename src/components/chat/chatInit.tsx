import type { UIMessage } from "@tanstack/ai-react"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentWindow } from "@tauri-apps/api/window"
import { type ReactNode, useEffect } from "react"
import { useChatContext } from "@/components/chat/chatContext"


export function ChatInit({ children }: { children: ReactNode }) {
	const { setMessages, sendMessage } = useChatContext()
	useEffect(() => {
		invoke<UIMessage[]>("EVENT_NAMES.get_current_history").then((history) => {
			
			setMessages(history)
		});
		const unlisten = getCurrentWindow().listen<{ translation_prompt: string; selected_text: string }>(
			"EVENT_NAMES.START_CHAT_STREAM",
			(e) => {
				sendMessage({
					content: [
						{ type: 'text', content: e.payload.selected_text },
						{ type: 'text', content: e.payload.translation_prompt },
					],
				})
			})
		return () => { unlisten.then((fn) => fn()) }
	}, [setMessages, sendMessage])

	return <>{children}</>
}
