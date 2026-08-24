import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "#/components/ui/dropdown-menu";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";

type LanguageOption = { label: string; value: string };

export default function LanguageSelector() {
	const [localLanguage, setLocalLanguage] = useState<string>("zh-CN");
	const [targetLanguage, setTargetLanguage] = useState<string>("en");
	const [options, setOptions] = useState<LanguageOption[]>([]);
	const [selfExplaining, setSelfExplaining] = useState(false);

	const localLanguageLabel =
		options.find((item) => item.value === localLanguage)?.label || localLanguage;
	const targetLanguageLabel =
		options.find((item) => item.value === targetLanguage)?.label || targetLanguage;

	useEffect(() => {
		(async () => {
			try {
				const local = await invoke<string>("get_local_language");
				const target = await invoke<string>("get_target_language");
				const selfExplainingModel = await invoke<boolean>("get_self_explaining_model");
				const remoteOptions = await invoke<[string, string][]>("get_language_options");
				if (Array.isArray(remoteOptions)) {
					setOptions(remoteOptions.map(([value, label]) => ({ value, label })));
				}
				if (local) setLocalLanguage(local);
				if (target) setTargetLanguage(target);
				setSelfExplaining(!!selfExplainingModel);
			} catch {
				// ignore
			}
		})();
	}, []);

	const setLocalLanguageAndPersist = async (value: string) => {
		try {
			await invoke("set_local_language", { language: value });
			setLocalLanguage(value);
		} catch (error) {
			console.error(error);
		}
	};

	const setTargetLanguageAndPersist = async (value: string) => {
		try {
			await invoke("set_target_language", { language: value });
			setTargetLanguage(value);
		} catch (error) {
			console.error(error);
		}
	};

	const toggleSelfExplaining = async () => {
		const enabled = !selfExplaining;
		try {
			await invoke("set_self_explaining_model", { enabled });
			setSelfExplaining(enabled);
		} catch (error) {
			console.error(error);
		}
	};

	return (
		<div className="px-2 pb-2 flex  flex-wrap">
			<Tooltip>
				<TooltipTrigger
					render={
						<Button size="icon-xs" variant="ghost">
							<Icons.question />
						</Button>
					}
				/>
				<TooltipContent>
					<div>
						{m.translate_language_selector_tooltip_line1({ localLanguage: localLanguageLabel })}
					</div>
					<div>
						{m.translate_language_selector_tooltip_line2({ localLanguage: localLanguageLabel, targetLanguage: targetLanguageLabel })}
					</div>
				</TooltipContent>
			</Tooltip>

			<DropdownMenu>
				<DropdownMenuTrigger
					disabled={selfExplaining}
					render={
						<Button size="xs" variant="ghost">
							{options.find((item) => item.value === localLanguage)?.label}
						</Button>
					}
				/>
				<DropdownMenuContent side="top" align="start">
					{options.map((item) => (
						<DropdownMenuItem
							key={item.value}
							onClick={() => void setLocalLanguageAndPersist(item.value)}
						>
							{item.label}
						</DropdownMenuItem>
					))}
				</DropdownMenuContent>
			</DropdownMenu>
			<Button size="icon-xs" variant="ghost" disabled>
				<Icons.exchange />
			</Button>
			<DropdownMenu>
				<DropdownMenuTrigger
					disabled={selfExplaining}
					render={
						<Button size="xs" variant="ghost">
							{options.find((item) => item.value === targetLanguage)?.label}
						</Button>
					}
				/>
				<DropdownMenuContent side="top" align="start">
					{options.map((item) => (
						<DropdownMenuItem
							key={item.value}
							onClick={() => void setTargetLanguageAndPersist(item.value)}
						>
							{item.label}
						</DropdownMenuItem>
					))}
				</DropdownMenuContent>
			</DropdownMenu>
			<Button
				size="xs"
				variant="ghost"
				className={cn(
					selfExplaining ? "" : "opacity-50",
					"hover:text-inherit",
				)}
				onClick={() => void toggleSelfExplaining()}
			>
				{selfExplaining ? m.translate_language_selector_self_explaining_on() : m.translate_language_selector_self_explaining_off()}
			</Button>
		</div>
	);
}
