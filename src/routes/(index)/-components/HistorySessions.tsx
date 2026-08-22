import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { ScrollArea } from "#/components/ui/scroll-area";
import { cn } from "#/lib/utils";
import { useDrawerStack } from "#/routes/(index)/-components/DrawerStack";
import type { Session } from "#/types";

function HistorySessionsContent({ className }: { className?: string }) {
	const [items, setItems] = useState<Session[]>([]);
	useEffect(() => {
		invoke<Session[]>("list_history_sessions")
			.then(setItems)
			.catch(console.error);
	}, []);
	return (
		<ScrollArea className={cn("h-full", "overflow-hidden", className)}>
			<div className="max-w-screen flex-coh items-start px-2">
				{items.map((s) => (
					<Button
						className="w-full cursor-pointer"
						key={s.session_id}
						variant={"ghost"}
						onClick={async () => {
							try {
								await invoke("open_session", {
									session_id: s.session_id,
								});
							} catch (err) {
								console.error(err);
							}
						}}
					>
						<span className="truncate w-full text-start">{s.title}</span>
					</Button>
				))}
			</div>
		</ScrollArea>
	);
}

export function HistorySessions() {
	const { push } = useDrawerStack();
	return (
		<Button
			size={"icon-sm"}
			variant={"ghost"}
			onClick={() => {
				push({
					title: "History",
					showSwipeHandle: true,
					content: <HistorySessionsContent />,
				});
			}}
		>
			<Icons.list />
		</Button>
	);
}
