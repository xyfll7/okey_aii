import type { UIMessage } from "@tanstack/ai-react";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type as ostype } from "@tauri-apps/plugin-os";
import { useEffect, useRef, useState } from "react";
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
	return (
		<ChatProvider
			session_id={session_id}
			onChatReady={(append) => {
				// Send the freshly selected text as this session's opening user
				// message once its chat is ready. The backend creates a new
				// session per gesture and passes session_id + user_contents via
				// the event; since session_id changes per session, onChatReady
				// fires once per opened session.
				if (!user_contents?.length) return;
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
			}}
		>
			<BubbleView />
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

/**
 * 每一轮对话的流式输出正常结束后，执行一次 onRoundEnd。
 * 判定条件：不再忙碌（status 非 submitted/streaming）+ 无错误 +
 * 出现了新的 assistant 消息（按消息 id 去重）。
 */
function useChatRoundEnd(options: {
	chat: UIMessage | undefined;
	isBusy: boolean;
	error: unknown;
	onRoundEnd: () => void;
}) {
	const { chat, isBusy, error, onRoundEnd } = options;
	// 用 ref 持有最新回调，回调身份变化不会触发重复判定
	const onRoundEndRef = useRef(onRoundEnd);
	useEffect(() => {
		onRoundEndRef.current = onRoundEnd;
	});

	const lastHandledIdRef = useRef<string | null>(null);
	useEffect(() => {
		if (isBusy) return; // 仍在流式生成中，等这轮结束
		if (error) return; // 出错的一轮不算正常结束
		if (!chat?.id) return; // 还没有 assistant 消息
		if (lastHandledIdRef.current === chat.id) return; // 这一轮已处理过
		lastHandledIdRef.current = chat.id;
		onRoundEndRef.current();
	}, [chat, isBusy, error]);
}

function BubbleView() {
	const { messages, status, error } = useChatContext();
	useChatInit();
	const chat = (() => {
		const item = messages?.at(-1);
		return item?.role === "assistant" ? item : undefined;
	})();
	const chatText = chat ? (getMessageText(chat)[0] ?? "") : "";
	const isBusy = status === "submitted" || status === "streaming";
	// 指向实际渲染 chatText 的节点，用于测量其真实占用的宽度
	const chatTextRef = useRef<HTMLDivElement>(null);
	// 右侧固定按钮区（复制/朗读/展开），用于计算窗口整体所需宽度
	const actionsRef = useRef<HTMLDivElement>(null);

	useChatRoundEnd({
		chat,
		isBusy,
		error,
		onRoundEnd: () => {
			// TODO: 在这里写本轮对话结束后的逻辑（每一轮都会执行且只执行一次）
			// onRoundEnd 在渲染提交之后触发，此时 chatTextRef 已指向渲染出的节点
			const el = chatTextRef.current;
			if (el) {
				// 文本右边缘已包含左侧把手与间距；再加上右侧按钮区与预留间距，
				// 即为让整行内容完整显示所需的最小窗口逻辑宽度
				const actionsWidth = actionsRef.current?.getBoundingClientRect().width ?? 0;
				const desiredWidth = Math.ceil(el.getBoundingClientRect().right + actionsWidth + 10);
				invoke("resize_translate_bubble", { width: desiredWidth }).catch(console.error);
			}
		},
	});

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
							<div ref={chatTextRef}>{chatText}</div>
							{chatText ? (
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
			<div ref={actionsRef} className="flex">
				<Button className={cn("")} size={"icon-sm"} variant={"ghost"}>
					<Copyed text={chatText} />
				</Button>
				<Button
					className={cn("")}
					size={"icon-sm"}
					variant={"ghost"}
					onClick={() => {
						const chat_user = messages?.at(-2);
						if (chat_user) speak(getMessageText(chat_user)[0]);
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
