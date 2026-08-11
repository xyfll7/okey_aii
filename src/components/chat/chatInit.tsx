import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type ReactNode, useEffect } from "react";
import { type RigMessage, rigMessageToUIMessage } from "#/lib/rigMessage";
import type { Session } from "#/routes/(index)";
import { useChatContext } from "@/components/chat/chatContext";
// import { useDrawerStack } from "../drawer-stack";
// import { SessionView } from "../session-view";
export function ChatInit({ children }: { children: ReactNode }) {
	const { setMessages, append } = useChatContext();
	// const { push } = useDrawerStack();
	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const session = sessions.at(0);
				invoke<RigMessage[]>("get_history", {
					session_id: session?.session_id,
				}).then((history) => {
					setMessages(history.map((e) => rigMessageToUIMessage(e)));
				});
			})
			.catch(console.error);
		const unlisten = getCurrentWindow().listen<RigMessage>(
			"on_message",
			(e) => {
				append(rigMessageToUIMessage(e.payload));
			},
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [setMessages, append]);

	return <>{children}</>;
}
