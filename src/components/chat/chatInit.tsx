import type { UIMessage } from "@tanstack/ai-react";
import { invoke } from "@tauri-apps/api/core";
import { type ReactNode, useEffect } from "react";
import type { Session } from "#/routes/(index)";
import { useChatContext } from "@/components/chat/chatContext";
import { useDrawerStack } from "../drawer-stack";
import { SessionView } from "../session-view";

export function ChatInit({ children }: { children: ReactNode }) {
	const { setMessages, sendMessage } = useChatContext();
	const { push } = useDrawerStack();
	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const session = sessions.at(0);
				invoke<UIMessage[]>("get_history", {
					session_id: session?.session_id,
				}).then((history) => {
					setMessages(history);
				});
			})
			.catch(console.error);

		push({
			title: "Settings",
			showSwipeHandle: true,
			content: <SessionView />,
		});
		// const unlisten = getCurrentWindow().listen<{
		// 	translation_prompt: string;
		// 	selected_text: string;
		// }>("EVENT_NAMES.START_CHAT_STREAM", (e) => {
		// 	sendMessage({
		// 		content: [
		// 			{ type: "text", content: e.payload.selected_text },
		// 			{ type: "text", content: e.payload.translation_prompt },
		// 		],
		// 	});
		// });
		// return () => {
		// 	unlisten.then((fn) => fn());
		// };
	}, [setMessages, push]);

	return <>{children}</>;
}
