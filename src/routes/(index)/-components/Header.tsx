import { type as ostype } from "@tauri-apps/plugin-os";
import type React from "react";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";

function CreateNewSession() {
	return (
		<Button size={"icon-sm"} variant={"ghost"} onClick={async () => {}}>
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

export function Header(props: React.ComponentProps<"div">) {
	const _ostype = ostype();
	const [hotkey, setHotkey] = useState<string>("");
	useEffect(() => {}, []);

	if (["macos"].includes(_ostype)) {
		return (
			<div
				className={cn("flex items-center justify-end", props.className)}
				data-tauri-drag-region
			>
				<CreateNewSession />

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

				<CreateNewSession />
			</div>
			<div className=" flex">
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
