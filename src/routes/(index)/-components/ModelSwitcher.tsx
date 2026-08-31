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
import { useLocale } from "#/lib/locale";
import type { ModelInfo, ProviderInfo, Session } from "#/types";

// The authoritative provider list is fetched at runtime via `list_providers`;
// each provider carries its localized label resolved by the backend's own i18n,
// so the frontend keeps no provider → label mapping of its own.
type ProviderWithModels = ProviderInfo & { models: ModelInfo[] };

const VALUE_SEP = "\u0000";

// Encode a provider + model pair into a single select value.
function comboValue(provider: string, modelId: string): string {
	return `${provider}${VALUE_SEP}${modelId}`;
}

export function ModelSwitcher({ session_id }: { session_id: string }) {
	const locale = useLocale();
	const [provider, setProvider] = useState<string | null>(null);
	const [model, setModel] = useState<string>("");
	const [providers, setProviders] = useState<ProviderWithModels[]>([]);

	// Load the provider list from the backend together with each provider's
	// models, then the current session's provider/model once the list is known.
	// Re-fetches when the locale changes so the provider labels stay in the
	// active language.
	// biome-ignore lint/correctness/useExhaustiveDependencies: useLocale() re-renders on language change; the fetch must re-run to refresh backend-resolved labels.
	useEffect(() => {
		let cancelled = false;
		(async () => {
			try {
				const pids = await invoke<ProviderInfo[]>("list_providers");
				if (cancelled) return;
				const groups = await Promise.all(
					pids.map(async (p) => {
						try {
							const models = await invoke<ModelInfo[]>("list_models", {
								provider: p.id,
							});
							return { ...p, models };
						} catch {
							// A provider whose models can't be listed (missing key,
							// unreachable endpoint) simply exposes nothing.
							return { ...p, models: [] as ModelInfo[] };
						}
					}),
				);
				if (cancelled) return;
				// Drop providers with no usable model instead of rendering an
				// empty group in the popup.
				setProviders(groups.filter((g) => g.models.length > 0));
				const sessions = await invoke<Session[]>("list_sessions");
				const session = sessions.find((s) => s.session_id === session_id);
				if (cancelled) return;
				if (session && groups.some((p) => p.id === session.provider.id)) {
					setProvider(session.provider.id);
					setModel(session.model);
				}
			} catch (error) {
				console.error(error);
			}
		})();
		return () => {
			cancelled = true;
		};
	}, [session_id, locale]);

	const selectedLabel = useMemo(() => {
		if (!provider || !model) return "--";
		return (
			providers
				.find((p) => p.id === provider)
				?.models.find((m) => m.id === model)?.label || model
		);
	}, [providers, provider, model]);

	const onValueChange = async (value: string | null) => {
		if (!value) return;
		const [pid, modelId] = value.split(VALUE_SEP);
		if (!providers.some((p) => p.id === pid)) return;
		try {
			// A model only exists within a provider, so both halves go over in
			// one command; applying them separately would leave the session
			// holding a model from the previous provider in between.
			const apiKeys = pid !== provider
				? await invoke<Record<string, string>>("get_api_keys")
				: null;
			await invoke("switch_combo", {
				session_id,
				provider: pid,
				model: modelId,
				api_key: apiKeys?.[pid] ?? null,
			});
			setProvider(pid);
			setModel(modelId);
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
				{providers.map((p, index) => (
					<Fragment key={p.id}>
						{index > 0 && <SelectSeparator />}
						<SelectGroup>
							<SelectLabel>{p.label}</SelectLabel>
							{p.models.map((m) => (
								<SelectItem key={m.id} value={comboValue(p.id, m.id)}>
									{m.label}
								</SelectItem>
							))}
						</SelectGroup>
					</Fragment>
				))}
			</SelectContent>
		</Select>
	);
}
