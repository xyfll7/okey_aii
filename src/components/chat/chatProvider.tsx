import type { UIMessage } from "@tanstack/ai/client";
import { useChat } from "@tanstack/ai-react";
import type { ReactNode } from "react";
import { chatAdapter } from "./chatAdapter";
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
	const append = (arg: UIMessage) => {

		return originalAppend(arg);
	};

	return (
		<ChatContext.Provider
			value={{ ...chat, append: append as typeof originalAppend }}
		>
			{children}
		</ChatContext.Provider>
	);
}
