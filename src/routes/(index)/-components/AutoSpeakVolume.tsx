import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Icons } from "@/components/icon";
import { Button } from "@/components/ui/button";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import {
	AutoSpeakState,
	type AutoSpeakState as AutoSpeakStateValue,
} from "@/lib/types";

const NEXT: Record<AutoSpeakStateValue, AutoSpeakStateValue> = {
	[AutoSpeakState.Off]: AutoSpeakState.Single,
	[AutoSpeakState.Single]: AutoSpeakState.All,
	[AutoSpeakState.All]: AutoSpeakState.Off,
};

const AutoSpeakVolume = ({ className }: { className?: string }) => {
	const [state, setState] = useState<AutoSpeakStateValue>(
		AutoSpeakState.Single,
	);

	useEffect(() => {
		invoke<AutoSpeakStateValue>("get_auto_speak")
			.then(setState)
			.catch(console.error);
	}, []);

	const icon = {
		[AutoSpeakState.Off]: <Icons.volumeOff className={className} />,
		[AutoSpeakState.Single]: <Icons.volumeLow className={className} />,
		[AutoSpeakState.All]: <Icons.volumeHigh className={className} />,
	}[state];

	return (
		<Tooltip>
			<TooltipTrigger
				render={
					<Button
						size="icon-sm"
						variant="ghost"
						onClick={async () => {
							const next = NEXT[state];
							try {
								const applied = await invoke<AutoSpeakStateValue>(
									"set_auto_speak",
									{ auto_speak: next },
								);
								setState(applied);
							} catch (err) {
								console.error(err);
							}
						}}
					>
						{icon}
					</Button>
				}
			/>
			<TooltipContent>
				{
					{
						[AutoSpeakState.Off]: "m.auto_speak_off()",
						[AutoSpeakState.Single]: "m.auto_speak_single()",
						[AutoSpeakState.All]: "m.auto_speak_all()",
					}[state]
				}
			</TooltipContent>
		</Tooltip>
	);
};

export default AutoSpeakVolume;
