import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { useDrawerStack } from "#/components/drawer-stack";
import { SessionView } from "#/components/session-view";
import type { Session } from "#/types";
export const Route = createFileRoute("/(index)/")({ component: Home });

function Home() {
	useBackendEvent();
	const session_id = useSessionId();
	return session_id && <SessionView session_id={session_id} />
}

function useSessionId() {
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
						showSwipeHandle: true,
						content: <SessionView session_id={session.session_id} />,
					});
				}
			})
			.catch(console.error);
	}, [push]);
	return session_id;
}

function useBackendEvent() {
	const { push } = useDrawerStack();
	useEffect(() => {
		const unlisten = getCurrentWindow().listen<string>(
			"on_create_session",
			(e) => {
				push({
					id: e.payload,
					showSwipeHandle: true,
					content: <SessionView session_id={e.payload} />,
				});
			},
		);
		return () => {
			unlisten.then((fn) => fn());
		};
	}, [push]);
}
