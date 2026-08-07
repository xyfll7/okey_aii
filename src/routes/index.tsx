import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

export const Route = createFileRoute("/")({ component: Home });

interface Session {
	session_id: string;
	title: string;
	provider: string;
	model: string;
	preset_id: string;
	created_at: number;
}

function Home() {
	const [sessions, setSessions] = useState<Session[]>([]);

	const refresh = useCallback(() => {
		invoke<Session[]>("list_sessions")
			.then(setSessions)
			.catch(console.error);
	}, []);

	useEffect(() => {
		refresh();
	}, [refresh]);

	const handleNewSession = async () => {
		await invoke<string>("open_session");
		refresh();
	};

	return (
		<div className="p-8">
			<button
				type="button"
				onClick={handleNewSession}
				className="rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
			>
				新建会话
			</button>
			<h1 className="text-4xl font-bold">Sessions</h1>
			{sessions.length === 0 ? (
				<p className="mt-4 text-lg text-gray-500">暂无会话</p>
			) : (
				<ul className="mt-4 space-y-2">
					{sessions.map((s) => (
						<li key={s.session_id} className="rounded border p-3">
							<span className="font-semibold">{s.title}</span>
							<span className="ml-2 text-gray-500">
								{s.provider} / {s.model} / {s.preset_id} / {s.session_id}
							</span>
						</li>
					))}
				</ul>
			)}
		</div>
	);
}
