import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type as ostype } from "@tauri-apps/plugin-os";
import { useEffect, useState } from "react";
import { cn } from "#/lib/utils";
import { useLocale } from "#/lib/locale";
import { useDrawerStack } from "#/routes/(index)/-components/DrawerStack";
import { SessionView } from "#/routes/(index)/-components/SessionView";
import type { Session } from "#/types";
import { Header } from "./-components/Header";
export const Route = createFileRoute("/(index)/")({ component: Home });

function Home() {
	useCreateSessionEvent();
	// 语言切换时强制本组件（及 Header、SessionView 等主内容）重渲染。
	// 必须在此订阅：上游 <Outlet /> 是 React.memo 且不订阅 locale store，
	// 仅靠 RootComponent 重渲染会被 memo bail-out 拦截，无法传到这里。
	useLocale();
	const [session_id, setSession_id] = useSessionId();
	const _ostype = ostype();
	useEffect(() => {
		getCurrentWindow().emit("on_page_index_loaded").catch(console.error);
	}, []);
	return (
		<div
			className={cn(
				{ "border rounded-xl": ["linux"].includes(_ostype) },
				"bg-background",
				"h-full",
				"flex flex-col overflow-hidden",
			)}
			data-tauri-drag-region
		>
			<Header className="p-1" onNewSession={setSession_id} />
			<div className={cn("relative h-full", "flex flex-col overflow-hidden")}>
				{session_id && <SessionView key={session_id} session_id={session_id} />}
			</div>
		</div>
	);
}

function useSessionId(): [string, (id: string) => void] {
	const [session_id, setSession_id] = useState("");
	const { push } = useDrawerStack();
	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const [first_session, ...rest_sessions] = sessions;
				first_session?.session_id && setSession_id(first_session?.session_id);
				for (const session of rest_sessions) {
					push({
						id: session.session_id,
						content: () => <SessionView session_id={session.session_id} />,
					});
				}
			})
			.catch(console.error);
	}, [push]);
	return [session_id, setSession_id];
}

function useCreateSessionEvent() {
	const { push } = useDrawerStack();
	useEffect(() => {
		const unlisten = getCurrentWindow().listen<string>(
			"on_open_session_with_session_id",
			(e) => {
				push({
					id: e.payload,
					content: () => <SessionView session_id={e.payload} />,
				});
			},
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [push]);
}
