import type { UIMessage } from "@tanstack/ai/client";
import { useChat } from "@tanstack/ai-react";
import { invoke } from "@tauri-apps/api/core";
import type { ReactNode } from "react";
import { type RigHistoryItem, rigMessageToUIMessage } from "#/lib/rigMessage";
import { speak } from "#/lib/utils";
import { AutoSpeakState } from "@/lib/types";
import { buildPromptHistoryItem, chatAdapter } from "./chatAdapter";
import { ChatContext } from "./chatContext";
import { getMessageText } from "./chatUtils";

function autoSpeak(message: UIMessage) {
	invoke<AutoSpeakState>("get_auto_speak").then((res) => {
		const selectedText = getMessageText(message)[0];
		const isSingleWord = selectedText.trim().split(/\s+/).length === 1;
		if (
			(res === AutoSpeakState.Single && isSingleWord) ||
			(res === AutoSpeakState.All && selectedText.trim().length > 0)
		) {
			speak(selectedText);
		}
	});
}

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
		// Assemble the prompt template in the frontend as early as possible (language detection + user language config),
		// instead of assembling it in the backend send_message step where the earlier timing would be missed.
		try {
			const item = buildPromptHistoryItem(arg);
			const assembled = await invoke<RigHistoryItem>("assemble_prompt", {
				item,
			});
			const message = rigMessageToUIMessage(assembled);

			autoSpeak(message);

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
