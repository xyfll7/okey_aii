import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { Item, ItemGroup } from "#/components/ui/item";
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

	const remove = async (session_id: string) => {
		try {
			await invoke("delete_session", { session_id });
			setItems((prev) => prev.filter((s) => s.session_id !== session_id));
		} catch (err) {
			console.error(err);
		}
	};

	return (
		<ScrollArea className={cn("h-full", "overflow-hidden", className)}>
			<div className="max-w-screen flex-coh items-start px-2 pr-4">
				<ItemGroup className="gap-1">
					{items.map((s) => (
						<Item
							key={s.session_id}
							variant="muted"
							role="listitem"
							className="py-0 px-0.5 w-full flex-nowrap overflow-hidden "
							onClick={async () => {
								try {
									await invoke("open_session", {
										session_id: s.session_id,
									});
								} catch (err) {
									console.error(err);
								}
							}}
							render={
								<a
									href="/"
									onClick={(e) => {
										e.preventDefault();
									}}
									className="flex justify-between overflow-hidden  bg-transparent"
								>
									<Button className={"flex-1 min-w-0 shrink"} variant={"ghost"} disabled>
										<div className="truncate w-full min-w-0 text-start">{s.title}</div>
									</Button>
									<Button
										className="opacity-0 pointer-events-none transition-opacity group-hover/item:opacity-100 group-hover/item:pointer-events-auto"
										size={"icon-xs"}
										variant={"secondary"}
										aria-label={`删除 ${s.title}`}
										onClick={(e) => {
											e.stopPropagation();
											e.preventDefault();
											remove(s.session_id);
										}}
									>
										<Icons.delete />
									</Button>
								</a>
							}
						></Item>
					))}
				</ItemGroup>
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
					content: <HistorySessionsContent />,
				});
			}}
		>
			<Icons.list />
		</Button>
	);
}
