import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { useChatContext } from "#/components/chat/chatContext";
import { type RigHistoryItem, rigMessageToUIMessage } from "#/lib/rigMessage";

export function useChatInit() {
	const { session_id, setMessages, append, messages, status } =
		useChatContext();
	useEffect(() => {
		function get_history() {
			invoke<RigHistoryItem[]>("get_history", { session_id }).then((history) =>
				setMessages(history.map((e) => rigMessageToUIMessage(e))),
			);
		}
		get_history();
		const unlistenPromise = getCurrentWindow().listen<RigHistoryItem>(
			`on_message_done${session_id}`,
			() => {
				get_history();
				// Only covers the race when a background round finishes exactly at first mount;
				// unbind after the first trigger to avoid pulling full history on every turn.
				unlistenPromise.then((fn) => fn());
			},
		);
		return () => {
			unlistenPromise.then((fn) => fn());
		};
	}, [setMessages, session_id]);
	useEffect(() => {
		const unlisten = getCurrentWindow().listen<RigHistoryItem>(
			`on_message_${session_id}`,
			(e) => {
				console.log("abc===========", session_id, e);
				append(rigMessageToUIMessage(e.payload));
			},
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [append, session_id]);
	return { messages, status };
}
