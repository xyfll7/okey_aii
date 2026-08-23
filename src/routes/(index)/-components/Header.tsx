import { invoke } from "@tauri-apps/api/core";
import { type as ostype } from "@tauri-apps/plugin-os";
import type React from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import { HistorySessions } from "#/routes/(index)/-components/HistorySessions";
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
	return (
		<Button
			size="icon-sm"
			variant="ghost"
			className={cn(className)}
			onClick={async () => {}}
		>
			<Icons.pin className={"text-green-300 dark:text-green-20"} />
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
				<Tooltip>
					<TooltipTrigger
						render={
							<Button size="icon-sm" variant="ghost">
								111
							</Button>
						}
					/>
					<TooltipContent></TooltipContent>
				</Tooltip>
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
				<Tooltip>
					<TooltipTrigger
						render={<Button size="icon-sm" variant="ghost"></Button>}
					/>
					<TooltipContent>123</TooltipContent>
				</Tooltip>

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
