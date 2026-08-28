import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuGroup,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "#/components/ui/dropdown-menu";
import { m } from "#/paraglide/messages";
import type { Session } from "#/types";

type ProviderId = "OpenAI" | "Anthropic" | "DeepSeek" | "Qwen" | "Zai";

const PROVIDERS: ProviderId[] = [
	"OpenAI",
	"Anthropic",
	"DeepSeek",
	"Qwen",
	"Zai",
];

type ModelInfo = { id: string; label: string };

function providerLabel(id: ProviderId): string {
	switch (id) {
		case "OpenAI":
			return m.model_providers_OpenAI();
		case "Anthropic":
			return m.model_providers_Anthropic();
		case "DeepSeek":
			return m.model_providers_DeepSeek();
		case "Qwen":
			return m.model_providers_Qwen();
		case "Zai":
			return m.model_providers_ZAI();
	}
}

function isProviderId(value: string): value is ProviderId {
	return (PROVIDERS as string[]).includes(value);
}

export function ModelSwitcher({ session_id }: { session_id: string }) {
	const [provider, setProvider] = useState<ProviderId | null>(null);
	const [model, setModel] = useState<string>("");
	const [models, setModels] = useState<ModelInfo[]>([]);

	// Load the current session's provider/model.
	useEffect(() => {
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const session = sessions.find((s) => s.session_id === session_id);
				if (session && isProviderId(session.provider)) {
					setProvider(session.provider);
					setModel(session.model);
				}
			})
			.catch((error) => console.error(error));
	}, [session_id]);

	// Load the model list whenever the provider changes.
	useEffect(() => {
		if (!provider) return;
		invoke<ModelInfo[]>("list_models", { provider })
			.then((list) => {
				setModels(list);
				// If the current model doesn't belong to this provider (e.g. right after
				// switching providers), fall back to that provider's default model.
				setModel((prev) =>
					list.some((item) => item.id === prev) ? prev : (list[0]?.id ?? ""),
				);
			})
			.catch((error) => console.error(error));
	}, [provider]);

	const currentModelLabel = useMemo(
		() => models.find((item) => item.id === model)?.label || model || "--",
		[models, model],
	);

	const onSelectProvider = async (next: ProviderId) => {
		if (next === provider) return;
		try {
			const apiKeys = await invoke<Record<string, string>>("get_api_keys");
			await invoke("switch_provider", {
				session_id,
				provider: next,
				api_key: apiKeys[next] ?? null,
			});
			setProvider(next);
			// Provider switch resets the session model to that provider's default,
			// which is the first item in the freshly loaded model list.
		} catch (error) {
			console.error(error);
		}
	};

	const onSelectModel = async (next: string) => {
		if (next === model || !provider) return;
		try {
			await invoke("switch_model", { session_id, model: next });
			setModel(next);
		} catch (error) {
			console.error(error);
		}
	};

	return (
		<div className="flex items-center gap-1">
			<DropdownMenu>
				<DropdownMenuTrigger
					render={
						<Button size="xs" variant="ghost">
							{provider ? providerLabel(provider) : "--"}
						</Button>
					}
				/>
				<DropdownMenuContent side="top" align="start">
					<DropdownMenuGroup>
						<DropdownMenuLabel>{m.common_api_provider()}</DropdownMenuLabel>
					</DropdownMenuGroup>
					{PROVIDERS.map((id) => (
						<DropdownMenuItem
							key={id}
							onClick={() => onSelectProvider(id)}
						>
							{providerLabel(id)}
						</DropdownMenuItem>
					))}
				</DropdownMenuContent>
			</DropdownMenu>

			{models.length > 0 && (
				<DropdownMenu>
					<DropdownMenuTrigger
						render={
							<Button size="xs" variant="ghost">
								{currentModelLabel}
							</Button>
						}
					/>
					<DropdownMenuContent side="top" align="start">
						<DropdownMenuGroup>
							<DropdownMenuLabel>
								{provider ? providerLabel(provider) : "--"}
							</DropdownMenuLabel>
						</DropdownMenuGroup>
						<DropdownMenuSeparator />
						{models.map((item) => (
							<DropdownMenuItem
								key={item.id}
								onClick={() => onSelectModel(item.id)}
							>
								{item.label}
							</DropdownMenuItem>
						))}
					</DropdownMenuContent>
				</DropdownMenu>
			)}
		</div>
	);
}
