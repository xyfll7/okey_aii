import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { useChatContext } from "#/components/chat/chatContext";
import { type RigHistoryItem, rigMessageToUIMessage } from "#/lib/rigMessage";

export function useChatInit({ session_id }: { session_id: string }) {
	const { setMessages, append, messages, status } = useChatContext();
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
				// 仅用于兜底首屏挂载时后台那一轮恰好结束的竞态，
				// 首次触发后即解绑，避免后续每轮对话都全量拉取历史。
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
			(e) => append(rigMessageToUIMessage(e.payload)),
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [append, session_id]);
	return { messages, status };
}
