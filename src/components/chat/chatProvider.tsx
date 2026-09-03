import type { UIMessage } from "@tanstack/ai/client";
import { useChat } from "@tanstack/ai-react";
import { invoke } from "@tauri-apps/api/core";
import { type ReactNode, useCallback, useEffect, useRef } from "react";
import { type RigHistoryItem, rigMessageToUIMessage } from "#/lib/rigMessage";
import { speak } from "#/lib/utils";
import { AutoSpeakState } from "@/lib/types";
import { buildPromptHistoryItem, chatAdapter } from "./chatAdapter";
import { type ChatAppend, ChatContext } from "./chatContext";
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
	onChatReady,
}: {
	session_id: string;
	children: ReactNode;
	onChatReady?: (append: ChatAppend) => void;
}) {
	const chat = useChat({
		threadId: session_id,
		initialMessages: [],
		connection: chatAdapter(session_id),
	});

	// Keep a stable append reference so consumers (e.g. effects depending on
	// it) don't re-run on every provider re-render.
	const originalAppendRef = useRef(chat.append);
	originalAppendRef.current = chat.append;
	const append = useCallback(async (arg: UIMessage) => {
		// Assemble the prompt template in the frontend as early as possible (language detection + user language config),
		// instead of assembling it in the backend send_message step where the earlier timing would be missed.
		try {
			const item = buildPromptHistoryItem(arg);
			const assembled = await invoke<RigHistoryItem>("assemble_prompt", {
				item,
			});
			const message = rigMessageToUIMessage(assembled);

			autoSpeak(message);

			originalAppendRef.current(message);
		} catch (err) {
			console.error("assemble_prompt_item failed:", err);
		}
	}, []);

	// Optional ready hook: fire once per session, right after that session's
	// chat has been initialized and `append` is safe to call. session_id is
	// stable for the whole lifetime of a session, so a change always means a
	// new session — whether it arrives as a fresh <ChatProvider> mount or as a
	// new session_id prop on the same provider.
	const onChatReadyRef = useRef(onChatReady);
	onChatReadyRef.current = onChatReady;
	const readySessionRef = useRef<string | null>(null);

	useEffect(() => {
		if (readySessionRef.current === session_id) return;
		readySessionRef.current = session_id;
		onChatReadyRef.current?.(append as ChatAppend);
	}, [session_id, append]);

	return (
		<ChatContext.Provider
			value={{
				...chat,
				session_id,
				append: append as typeof chat.append,
			}}
		>
			{children}
		</ChatContext.Provider>
	);
}
