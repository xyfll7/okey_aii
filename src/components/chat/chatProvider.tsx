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

// Compare a single message part, treating two missing parts as equal so
// single-part messages are still matched correctly.
function isSamePart(
	a: UIMessage["parts"][number] | undefined,
	b: UIMessage["parts"][number] | undefined,
): boolean {
	if (!a && !b) return true;
	return a?.type === "text" && b?.type === "text" && a.content === b.content;
}

// A newly assembled message is a duplicate when the last "user" message in the
// thread already matches it on both parts[0] and parts[1].
function isDuplicateUserMessage(
	messages: UIMessage[],
	message: UIMessage,
): boolean {
	const lastUserMessage = messages.filter((m) => m.role === "user").at(-1);
	if (!lastUserMessage) return false;
	return (
		isSamePart(lastUserMessage.parts[0], message.parts[0]) &&
		isSamePart(lastUserMessage.parts[1], message.parts[1])
	);
}

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
	const append = useCallback(
		async (arg: UIMessage) => {
			// Assemble the prompt template in the frontend as early as possible (language detection + user language config),
			// instead of assembling it in the backend send_message step where the earlier timing would be missed.
			try {
				const item = buildPromptHistoryItem(arg);
				const assembled = await invoke<RigHistoryItem>("assemble_prompt", {
					item,
				});
				const message = rigMessageToUIMessage(assembled);

				autoSpeak(message);

				// Skip appending when the last "user" message in the thread is
				// already identical to the one being assembled (i.e. it was appended
				// earlier), while still letting the auto-speak action run.
				if (isDuplicateUserMessage(chat.messages, message)) return;

				originalAppendRef.current(message);
			} catch (err) {
				console.error("assemble_prompt_item failed:", err);
			}
		},
		[chat],
	);

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
