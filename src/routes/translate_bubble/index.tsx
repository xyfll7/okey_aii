import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { type as ostype } from "@tauri-apps/plugin-os";
import { useEffect, useState } from "react";
import Copyed from "#/components/Copyed";
import { useChatContext } from "#/components/chat/chatContext";
import { useChatInit } from "#/components/chat/chatInit";
import { ChatProvider } from "#/components/chat/chatProvider";
import { getMessageText } from "#/components/chat/chatUtils";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import type { Session } from "#/types";
import { cn, speak } from "@/lib/utils";
export const Route = createFileRoute("/translate_bubble/")({
	component: RouteComponent,
});
function RouteComponent() {
	const session_id = useSessionId();
	if (!session_id) {
		return null;
	}
	return (
		<ChatProvider session_id={session_id}>
			<BubbleView session_id={session_id}></BubbleView>
		</ChatProvider>
	);
}

function useSessionId() {
	const [session_id, setSession_id] = useState("");

	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const last_session = sessions.at(-1);
				last_session?.session_id && setSession_id(last_session?.session_id);
			})
			.catch(console.error);
	}, []);
	return session_id;
}

function BubbleView({ session_id }: { session_id: string }) {
	useChatInit({ session_id });
	const { messages, status } = useChatContext();
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
