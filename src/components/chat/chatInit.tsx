import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { useChatContext } from "#/components/chat/chatContext";
import { type RigHistoryItem, rigMessageToUIMessage } from "#/lib/rigMessage";

export function useChatInit({ session_id }: { session_id: string }) {
	const { setMessages, append, messages, status } = useChatContext();
	useEffect(() => {
		invoke<RigHistoryItem[]>("get_history", { session_id }).then((history) => {
			setMessages(history.map((e) => rigMessageToUIMessage(e)));
		});
		const unlisten = getCurrentWindow().listen<RigHistoryItem>(
			`on_message_${session_id}`,
			(e) => append(rigMessageToUIMessage(e.payload)),
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [setMessages, append, session_id]);
	return { messages, status };
}
