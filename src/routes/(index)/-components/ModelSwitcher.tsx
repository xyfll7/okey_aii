import { invoke } from "@tauri-apps/api/core";
import { Fragment, useEffect, useMemo, useState } from "react";
import { InputGroupButton } from "#/components/ui/input-group";
import {
	Select,
	SelectContent,
	SelectGroup,
	SelectItem,
	SelectLabel,
	SelectSeparator,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";
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
type Combo = { provider: ProviderId; model: ModelInfo };

const VALUE_SEP = "\u0000";

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

// Encode a provider + model pair into a single select value.
function comboValue(provider: ProviderId, modelId: string): string {
	return `${provider}${VALUE_SEP}${modelId}`;
}

export function ModelSwitcher({ session_id }: { session_id: string }) {
	const [provider, setProvider] = useState<ProviderId | null>(null);
	const [model, setModel] = useState<string>("");
	const [combos, setCombos] = useState<Combo[]>([]);

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

	// Load every provider's model list once, so the select can show all
	// provider + model combinations in a single popup.
	useEffect(() => {
		Promise.all(
			PROVIDERS.map(async (pid) => {
				const list = await invoke<ModelInfo[]>("list_models", {
					provider: pid,
				});
				return list.map((item) => ({ provider: pid, model: item }));
			}),
		)
			.then((groups) => setCombos(groups.flat()))
			.catch((error) => console.error(error));
	}, []);

	const selectedLabel = useMemo(() => {
		if (!provider || !model) return "--";
		return (
			combos.find((c) => c.provider === provider && c.model.id === model)?.model
				.label || model
		);
	}, [combos, provider, model]);

	const onValueChange = async (value: string | null) => {
		if (!value) return;
		const [pid, modelId] = value.split(VALUE_SEP);
		if (!isProviderId(pid)) return;
		try {
			if (pid !== provider) {
				const apiKeys = await invoke<Record<string, string>>("get_api_keys");
				await invoke("switch_provider", {
					session_id,
					provider: pid,
					api_key: apiKeys[pid] ?? null,
				});
				setProvider(pid);
			}
			// Provider switch resets the session model to that provider's default,
			// so the selected model always needs to be applied explicitly.
			if (modelId !== model) {
				await invoke("switch_model", { session_id, model: modelId });
				setModel(modelId);
			}
		} catch (error) {
			console.error(error);
		}
	};

	return (
		<Select
			value={provider && model ? comboValue(provider, model) : null}
			onValueChange={onValueChange}
		>
			<SelectTrigger
				render={
					<InputGroupButton
						size="xs"
						className="h-6! border-transparent! bg-transparent! dark:bg-transparent! hover:bg-muted! dark:hover:bg-muted/50! focus-visible:border-transparent! focus-visible:ring-0!"
					/>
				}
			>
				<SelectValue>{() => selectedLabel}</SelectValue>
			</SelectTrigger>
			<SelectContent side="top" align="start" className="w-fit">
				{PROVIDERS.map((pid, index) => {
					const items = combos.filter((c) => c.provider === pid);
					if (items.length === 0) return null;
					return (
						<Fragment key={pid}>
							{index > 0 && <SelectSeparator />}
							<SelectGroup>
								<SelectLabel>{providerLabel(pid)}</SelectLabel>
								{items.map((c) => (
									<SelectItem
										key={c.model.id}
										value={comboValue(pid, c.model.id)}
									>
										{c.model.label}
									</SelectItem>
								))}
							</SelectGroup>
						</Fragment>
					);
				})}
			</SelectContent>
		</Select>
	);
}
