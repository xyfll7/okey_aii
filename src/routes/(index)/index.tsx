import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { useDrawerStack } from "#/components/drawer-stack";
import { SessionView } from "#/components/session-view";
import type { RigMessage } from "#/lib/rigMessage";
import type { Session } from "#/types";
export const Route = createFileRoute("/(index)/")({ component: Home });

function Home() {
	useBackendEvent();
	const session_id = useSessionId();
	return (
		<div className="p-8">
			{session_id && <SessionView session_id={session_id} />}
		</div>
	);
}

function useSessionId() {
	const [session_id, setSession_id] = useState("");
	const { push } = useDrawerStack();
	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const [session, ...rest_sessions] = sessions;
				session?.session_id && setSession_id(session?.session_id);
				rest_sessions.map((e) => {
					return push({
						id: e.session_id,
						showSwipeHandle: true,
						content: <SessionView session_id={e.session_id} />,
					});
				});
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
