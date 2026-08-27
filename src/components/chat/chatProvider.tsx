import type { UIMessage } from "@tanstack/ai/client";
import { useChat } from "@tanstack/ai-react";
import { invoke } from "@tauri-apps/api/core";
import type { ReactNode } from "react";
import { type RigHistoryItem, rigMessageToUIMessage } from "#/lib/rigMessage";
import { buildPromptHistoryItem, chatAdapter } from "./chatAdapter";
import { ChatContext } from "./chatContext";

export function ChatProvider({
	session_id,
	children,
}: {
	session_id: string;
	children: ReactNode;
}) {
	const chat = useChat({
		initialMessages: [],
		connection: chatAdapter(session_id),
	});

	const originalAppend = chat.append;
	const append = async (arg: UIMessage) => {
		// 尽早在前端组装 prompt 模板（语言检测 + 用户语言配置），
		// 避免在 send_message 后端环节才组装而错过更早的时机。
		try {
			const item = buildPromptHistoryItem(arg);
			const assembled = await invoke<RigHistoryItem>("assemble_prompt", {
				item,
			});
			const message = rigMessageToUIMessage(assembled);
			originalAppend(message);
		} catch (err) {
			console.error("assemble_prompt_item failed:", err);
		}
	};

	return (
		<ChatContext.Provider
			value={{ ...chat, append: append as typeof originalAppend }}
		>
			{children}
		</ChatContext.Provider>
	);
}
