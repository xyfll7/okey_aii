import type { UIMessage } from "@tanstack/ai-react";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type as ostype } from "@tauri-apps/plugin-os";
import { useEffect, useState } from "react";
import Copyed from "#/components/Copyed";
import { useChatContext } from "#/components/chat/chatContext";
import { useChatInit } from "#/components/chat/chatInit";
import { ChatProvider } from "#/components/chat/chatProvider";
import { getMessageText } from "#/components/chat/chatUtils";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { cn, speak } from "#/lib/utils";
import type { Session } from "#/types";
export const Route = createFileRoute("/translate_bubble/")({
	component: RouteComponent,
});
function RouteComponent() {
	const { session_id, user_contents } = useSessionId();
	if (!session_id) {
		return null;
	}
	console.log("sssssss",session_id)
	return (
		<ChatProvider session_id={session_id}>
			<BubbleView user_contents={user_contents} />
		</ChatProvider>
	);
}

function useSessionId() {
	const [session, setSession] = useState<{
		session_id: string;
		user_contents?: string[];
	} | null>(null);

	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const last_session = sessions.at(-1);
				if (last_session?.session_id) {
					setSession((prev) => prev ?? { session_id: last_session.session_id });
				}
			})
			.catch(console.error);
		const unlisten = getCurrentWindow().listen<{
			session_id: string;
			user_contents: string[];
		}>("on_open_session_with_session_id", (e) => {
			const { session_id, user_contents } = e.payload;
			setSession({ session_id, user_contents });
		});
		return () => {
			unlisten.then((fn) => fn());
		};
	}, []);
	return session ?? { session_id: "", user_contents: undefined };
}

function BubbleView({ user_contents }: { user_contents?: string[] }) {
	const { messages, status, session_id, append } = useChatContext();
	useChatInit();

	// Send the freshly selected text as the user message once the session is
	// ready; the backend passes it along with the session id via the event.
	// `append` is referentially stable (useCallback in ChatProvider) and both
	// `session_id`/`user_contents` only change together when a new event
	// arrives, so this effect fires exactly once per opened session.
	useEffect(() => {
		if (!session_id || !user_contents?.length) return;
		console.log("1111111_____")
		const [selected_text, translate_instruction] = user_contents;
		const userMessage: UIMessage = {
			id: `selected-${Date.now()}`,
			role: "user",
			parts: [
				{ type: "text", content: selected_text },
				...(translate_instruction
					? [{ type: "text" as const, content: translate_instruction }]
					: []),
			],
		};
		append(userMessage);
	}, [session_id, user_contents, append]);
	const chat = (() => {
		const item = messages?.at(-1);
		return item?.role === "assistant" ? item : undefined;
	})();
	const isBusy = status === "submitted" || status === "streaming";

	const _ostype = ostype();
	return (
		<div
			data-tauri-drag-region
			className={cn(
				"h-full",
				"p-0.5",
				"bg-background",
				"flex justify-between items-center",
				{ "border rounded-md": ["macos"].includes(_ostype) },
			)}
		>
			<div
				className="flex items-center justify-start w-full  overflow-hidden"
				data-tauri-drag-region
			>
				<div
					className="flex overflow-hidden cursor-grab  active:cursor-grabbing"
					data-tauri-drag-region
				>
					<Button
						className={cn(
							"hover:text-current",
							"hover:bg-transparent dark:hover:bg-transparent cursor-grab ",
							"active:translate-y-0!",
						)}
						size={"icon-sm"}
						variant={"ghost"}
						onClick={() => {}}
						data-tauri-drag-region
					>
						<Icons.gripVertical
							strokeWidth={3}
							className="cursor-grab  active:cursor-grabbing"
							data-tauri-drag-region
						/>
					</Button>
				</div>
				<div className="flex overflow-hidden text-nowrap flex-1 ">
					{isBusy ? (
						<div className="shimmer text-muted-foreground">..!@#$%^&*()_+</div>
					) : (
						<>
							<div>{chat && getMessageText(chat).join("")}</div>
							{chat && getMessageText(chat).join("") ? (
								<span
									className="truncate text-transparent selection:bg-transparent cursor-grab hover:cursor-grabbing"
									data-tauri-drag-region
								>
									.........................
								</span>
							) : (
								""
							)}
						</>
					)}
				</div>
			</div>
			<div className="flex">
				<Button className={cn("")} size={"icon-sm"} variant={"ghost"}>
					<Copyed text={chat ? getMessageText(chat).join("") : ""} />
				</Button>
				<Button
					className={cn("")}
					size={"icon-sm"}
					variant={"ghost"}
					onClick={() => {
						const chat_user = messages?.at(-2);
						if (chat_user) speak(getMessageText(chat_user).join(""));
					}}
				>
					<Icons.volumeHigh />
				</Button>
				<Button
					className={cn("")}
					size={"icon-sm"}
					variant={"ghost"}
					onClick={async () => {
						await invoke("open_window_index");
					}}
				>
					<Icons.arrowExpand />
				</Button>
			</div>
		</div>
	);
}
