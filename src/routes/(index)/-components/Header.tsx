import { invoke } from "@tauri-apps/api/core";
import { type as ostype } from "@tauri-apps/plugin-os";
import type React from "react";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";
import { HistorySessions } from "#/routes/(index)/-components/HistorySessions";
import AutoSpeakVolume from "./AutoSpeakVolume";
import { Settings } from "./Settings";

function CreateNewSession({
	onNewSession,
}: {
	onNewSession?: (session_id: string) => void;
}) {
	return (
		<Button
			size={"icon-sm"}
			variant={"ghost"}
			onClick={async () => {
				try {
					const session_id = await invoke<string | null>("new_session");
					// 当前会话无历史数据 → 什么也不做
					if (!session_id) return;
					// 更新主视图 session_id，SessionView 因 key 变化而重载
					onNewSession?.(session_id);
				} catch (err) {
					console.error(err);
				}
			}}
		>
			<Icons.chat />
		</Button>
	);
}

function PinWindow({ className }: { className?: string }) {
	const [pinned, setPinned] = useState(false);

	useEffect(() => {
		invoke<boolean>("get_pin_index_window")
			.then(setPinned)
			.catch(console.error);
	}, []);

	return (
		<Button
			size="icon-sm"
			variant="ghost"
			className={cn(className)}
			title={pinned ? "取消置顶" : "置顶窗口"}
			onClick={async () => {
				try {
					const applied = await invoke<boolean>("set_pin_index_window", {
						pinned: !pinned,
					});
					setPinned(applied);
				} catch (err) {
					console.error(err);
				}
			}}
		>
			<Icons.pin
				className={cn(pinned && "text-green-300")}
			/>
		</Button>
	);
}

export function Header(
	props: React.ComponentProps<"div"> & {
		onNewSession?: (sessionId: string) => void;
	},
) {
	const _ostype = ostype();

	if (["macos"].includes(_ostype)) {
		return (
			<div
				className={cn("flex items-center justify-end", props.className)}
				data-tauri-drag-region
			>
				<CreateNewSession onNewSession={props.onNewSession} />
				<HistorySessions />
				<Settings />
				<AutoSpeakVolume />
				<PinWindow className="mr-1" />
			</div>
		);
	}
	return (
		<div
			className={cn("flex items-center justify-between", props.className)}
			data-tauri-drag-region
		>
			<div className="flex items-center">
				<PinWindow />
				<AutoSpeakVolume />

				<HistorySessions />
				<CreateNewSession onNewSession={props.onNewSession} />
			</div>
			<div className=" flex">
				<Settings />
				<Button
					className="ml-1"
					size={"icon-sm"}
					variant={"ghost"}
					onClick={() => {}}
				>
					<Icons.x />
				</Button>
			</div>
		</div>
	);
}
