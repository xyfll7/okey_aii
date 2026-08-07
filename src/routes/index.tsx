import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import {
	Field,
	FieldDescription,
	FieldGroup,
	FieldLabel,
} from "#/components/ui/field";
import {
	InputGroup,
	InputGroupAddon,
	InputGroupButton,
	InputGroupText,
	InputGroupTextarea,
} from "#/components/ui/input-group";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";

export const Route = createFileRoute("/")({ component: Home });

interface Session {
	session_id: string;
	title: string;
	provider: string;
	model: string;
	preset_id: string;
	created_at: number;
}

/// 单条聊天记录(对齐 rig::message::Message 的 serde 序列化)
interface ChatMessage {
	role: "user" | "assistant" | "system" | "tool";
	content: string;
}

function Home() {
	const [sessions, setSessions] = useState<Session[]>([]);
	const [activeTab, setActiveTab] = useState<string>("");

	const refresh = useCallback(() => {
		invoke<Session[]>("list_sessions").then(setSessions).catch(console.error);
	}, []);

	useEffect(() => {
		refresh();
	}, [refresh]);

	// 会话列表变化时，确保激活的 tab 仍然有效，否则回退到第一个
	useEffect(() => {
		if (sessions.length === 0) {
			setActiveTab("");
			return;
		}
		if (!sessions.some((s) => s.session_id === activeTab)) {
			setActiveTab(sessions[0].session_id);
		}
	}, [sessions, activeTab]);

	return (
		<div className="p-8">
			<div className="flex items-center justify-between">
				<h1 className="text-4xl font-bold">Sessions</h1>
				<Button
					onClick={async () => {
						await invoke<string>("create_session");
						refresh();
					}}
				>
					新建会话
				</Button>
			</div>

			{sessions.length === 0 ? (
				<p className="mt-4 text-lg text-gray-500">暂无会话</p>
			) : (
				<Tabs value={activeTab} onValueChange={setActiveTab} className="mt-4">
					<TabsList>
						{sessions.map((s) => (
							<TabsTrigger key={s.session_id} value={s.session_id}>
								{s.title}
							</TabsTrigger>
						))}
					</TabsList>
					{sessions.map((s) => (
						<TabsContent key={s.session_id} value={s.session_id}>
							<div className="rounded border p-3">
								<span className="font-semibold">{s.title}</span>
								<span className="ml-2 text-gray-500">
									{s.provider} / {s.model} / {s.preset_id} / {s.session_id}
								</span>
							</div>
							<SessionView sessionId={s.session_id} />
							<InputGroupBlockEnd sessionId={s.session_id} />
						</TabsContent>
					))}
				</Tabs>
			)}
		</div>
	);
}

/// 单个会话视图:加载历史 + 监听流式事件,渲染聊天气泡。
function SessionView({ sessionId }: { sessionId: string }) {
	const [messages, setMessages] = useState<ChatMessage[]>([]);
	/// 正在生成中的助手回复(流式累积,完成后并入 messages)
	const [streaming, setStreaming] = useState<string>("");
	const bottomRef = useRef<HTMLDivElement>(null);
	console.log("sssssid::",messages)
	// 1) 首次进入加载历史
	const reload = useCallback(() => {
		invoke<ChatMessage[]>("get_history", { sessionId })
			.then(setMessages)
			.catch(console.error);
	}, [sessionId]);

	useEffect(() => {
		reload();
	}, [reload]);

	// 2) 监听该 session 的事件流
	useEffect(() => {
		const event_name = `agui-event:${sessionId}`;
		let unlisten: (() => void) | undefined;

		(async () => {
			unlisten = await listen<{
				type: string;
				delta?: string;
				message?: string;
			}>(event_name, (e) => {
				const payload = e.payload;
				switch (payload.type) {
					case "RUN_STARTED":
						setStreaming("");
						break;
					case "TEXT_MESSAGE_CONTENT":
						if (payload.delta) {
							console.log("pppppp:", payload);
							setStreaming((prev) => prev + (payload.delta ?? ""));
						}
						break;
					case "TEXT_MESSAGE_END":
						// 流结束:把累积的内容并入 messages,清空 streaming
						setStreaming((cur) => {
							if (cur) {
								setMessages((prev) => [
									...prev,
									{ role: "assistant", content: cur },
								]);
							}
							return "";
						});
						// 后端已写入 history,同步一次以兜底(可选)
						break;
					case "RUN_ERROR":
						setStreaming("");
						console.error("RUN_ERROR", payload.message);
						break;
					default:
						break;
				}
			});
		})();

		return () => {
			unlisten?.();
		};
	}, [sessionId]);

	// 3) 自动滚动到底
	// biome-ignore lint/correctness/useExhaustiveDependencies: 副作用仅滚动,依赖用于触发
	useEffect(() => {
		bottomRef.current?.scrollIntoView({ behavior: "smooth" });
	}, [messages, streaming]);

	return (
		<div className="mt-3 h-[50vh] space-y-3 overflow-y-auto rounded border p-3">
			{messages.length === 0 && !streaming && (
				<p className="text-sm text-gray-400">开始新对话…</p>
			)}
			{messages.map((m, i) => (
				<Bubble
					key={`${m.role}-${m.content.slice(0, 8)}-${String(i)}`}
					variant={m.role}
					content={m.content}
				/>
			))}
			{streaming && (
				<Bubble variant="assistant" content={streaming} streaming />
			)}
			<div ref={bottomRef} />
		</div>
	);
}

function Bubble({
	variant,
	content,
	streaming,
}: {
	variant: string;
	content: string;
	streaming?: boolean;
}) {
	const isUser = variant === "user";
	const cc = JSON.stringify(content);
	return (
		<div className={`flex ${isUser ? "justify-end" : "justify-start"}`}>
			<div
				className={`max-w-[80%] whitespace-pre-wrap rounded-lg px-3 py-2 text-sm ${
					isUser
						? "bg-primary text-primary-foreground"
						: "bg-muted text-muted-foreground"
				}${streaming ? " animate-pulse" : ""}`}
			>
				{cc}
			</div>
		</div>
	);
}

/// 输入框 + 发送按钮:调用 send_message,流式结果由 SessionView 监听展示。
export function InputGroupBlockEnd({ sessionId }: { sessionId: string }) {
	const [text, setText] = useState("");
	const [sending, setSending] = useState(false);
	const MAX = 280;

	const send = useCallback(async () => {
		const prompt = text.trim();
		if (!prompt || sending) return;
		setSending(true);
		setText("");
		try {
			await invoke("send_message", { sessionId, prompt });
		} catch (e) {
			console.error(e);
		} finally {
			setSending(false);
		}
	}, [text, sending, sessionId]);

	return (
		<FieldGroup className="max-w-sm">
			<Field>
				<FieldLabel htmlFor="block-end-textarea">Textarea</FieldLabel>
				<InputGroup>
					<InputGroupTextarea
						id="block-end-textarea"
						placeholder="Write a comment..."
						value={text}
						maxLength={MAX}
						onChange={(e) => setText(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === "Enter" && !e.shiftKey) {
								e.preventDefault();
								send();
							}
						}}
						disabled={sending}
					/>
					<InputGroupAddon align="block-end">
						<InputGroupText>
							{text.length}/{MAX}
						</InputGroupText>
						<InputGroupButton
							variant="default"
							size="sm"
							className="ml-auto"
							disabled={sending || !text.trim()}
							onClick={send}
						>
							{sending ? "发送中…" : "Post"}
						</InputGroupButton>
					</InputGroupAddon>
				</InputGroup>
				<FieldDescription>
					Enter 发送,Shift+Enter 换行。流式接收回复。
				</FieldDescription>
			</Field>
		</FieldGroup>
	);
}
