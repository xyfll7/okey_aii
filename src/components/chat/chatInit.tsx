import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { type RigMessage, rigMessageToUIMessage } from "#/lib/rigMessage";
import { useChatContext } from "@/components/chat/chatContext";

export function useChatInit({ session_id }: { session_id: string }) {
	const { setMessages, append } = useChatContext();
	useEffect(() => {
		invoke<RigMessage[]>("get_history", { session_id }).then((history) => {
			setMessages(history.map((e) => rigMessageToUIMessage(e)));
		});
		const unlisten = getCurrentWindow().listen<RigMessage>(
			`on_message_${session_id}`,
			(e) => {
				append(rigMessageToUIMessage(e.payload));
			},
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [setMessages, append, session_id]);
}
