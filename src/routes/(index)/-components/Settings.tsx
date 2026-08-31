import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
	DropdownMenuContent,
	DropdownMenuGroup,
	DropdownMenuTrigger,
} from "#/components/ui/dropdown-menu";
import {
	Field,
	FieldDescription,
	FieldGroup,
	FieldLabel,
	FieldSet,
} from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { ScrollArea } from "#/components/ui/scroll-area";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { setLocale, useLocale } from "#/lib/locale";
import { cn } from "#/lib/utils";
import { useDrawerStack } from "#/routes/(index)/-components/DrawerStack";
import type { ProviderInfo } from "#/types";
import { m } from "@/paraglide/messages.js";

export function Settings() {
	const { push } = useDrawerStack();
	return (
		<Button
			size={"icon-sm"}
			variant={"ghost"}
			onClick={() => {
				push({
					title: () => (
						<div className="flex justify-between">
							{m.common_settings()}
							<LanguageSelector />
						</div>
					),
					content: () => <SettingsContent />,
				});
			}}
		>
			<Icons.settings />
		</Button>
	);
}

// The authoritative provider list comes from the backend via `list_providers`;
// each provider carries its localized label and API-key page URL, so no
// provider → label/link mapping is kept on the frontend.
function SettingsContent({ className }: { className?: string }) {
	const locale = useLocale();
	const [apiKeys, setApiKeys] = useState<Record<string, string>>({});
	const [providers, setProviders] = useState<ProviderInfo[]>([]);
	const [currentProvider, setCurrentProvider] = useState<string | null>(null);

	useEffect(() => {
		void locale
		let cancelled = false;
		(async () => {
			try {
				const [pids, keys] = await Promise.all([
					invoke<ProviderInfo[]>("list_providers"),
					invoke<Record<string, string>>("get_api_keys"),
				]);
				if (cancelled) return;
				setProviders(pids);
				setApiKeys(keys);
				// Keep the previously selected provider if it still exists, so a
				// locale-triggered re-fetch does not reset the user's selection.
				setCurrentProvider((prev) => {
					if (prev && pids.some((p) => p.id === prev)) return prev;
					return pids[0]?.id ?? null;
				});
			} catch (error) {
				console.error(error);
			}
		})();
		return () => {
			cancelled = true;
		};
	}, [locale]);

	const current = providers.find((p) => p.id === currentProvider);
	const resetKey = `${currentProvider ?? ""}-${apiKeys[currentProvider ?? ""] ?? ""}`;

	return (
		<ScrollArea className={cn("h-full", "overflow-hidden", className)}>
			<div className="max-w-screen flex-coh items-start px-2 pr-4 pt-1">
				<Card className="px-2.5 w-full">
					<form>
						<FieldGroup>
							<FieldSet>
								<FieldGroup>
									<Field>
										<FieldLabel>
											{m.common_api_provider()} (
											{current?.label ?? ""})
										</FieldLabel>
										<ToggleGroup
											size="sm"
											value={currentProvider ? [currentProvider] : []}
											onValueChange={(values) => {
												const value = values[0];
												if (value && providers.some((p) => p.id === value)) {
													setCurrentProvider(value);
												}
											}}
											variant="outline"
											spacing={2}
											className="flex-wrap w-full"
										>
											{providers.map((p) => (
												<ToggleGroupItem
													key={p.id}
													value={p.id}
													aria-label={p.label}
												>
													{p.label}
												</ToggleGroupItem>
											))}
										</ToggleGroup>
									</Field>
									<Field>
										<FieldLabel htmlFor="model-provider-api-key">
											{current?.label ?? ""} {m.common_api_key()}
										</FieldLabel>
										<Input
											key={resetKey}
											defaultValue={apiKeys[currentProvider ?? ""] ?? ""}
											id="model-provider-api-key"
											placeholder="Enter API Key"
											required
											onBlur={async (e) => {
												if (!currentProvider) return;
												const value = e.target.value.trim();
												if (value !== (apiKeys[currentProvider] ?? "")) {
													try {
														await invoke("set_api_key", {
															provider: currentProvider,
															api_key: value,
														});
														setApiKeys((prev) => ({
															...prev,
															[currentProvider]: value,
														}));
													} catch (error) {
														console.error(error);
													}
												}
											}}
										/>
										<FieldDescription>
											{m.common_stored_locally()}{" "}
											<a
												href={current?.api_key_url}
												target="_blank"
												rel="noreferrer"
											>
												{m.common_get_api_key({
													provider: current?.label ?? "",
												})}
											</a>
										</FieldDescription>
									</Field>
								</FieldGroup>
							</FieldSet>
						</FieldGroup>
					</form>
				</Card>
			</div>
		</ScrollArea>
	);
}

function LanguageSelector() {
	const currentLocale = useLocale();

	return (
		<DropdownMenu>
			<DropdownMenuTrigger
				render={
					<Button size={"icon-sm"} variant={"ghost"}>
						<Icons.languages />
					</Button>
				}
			/>
			<DropdownMenuContent className="w-40">
				<DropdownMenuGroup>
					{(
						[
							["en", m.languages_en(), "English"],
							["zh-CN", m.languages_zh_cn(), "中文"],
						] as const
					).map(([locale, label, displayLabel]) => (
						<DropdownMenuCheckboxItem
							className="flex flex-col items-start"
							key={locale}
							checked={currentLocale === locale}
							onCheckedChange={(checked) => {
								if (checked) {
									invoke("set_current_locale", { locale })
										.then(() => setLocale(locale))
										.catch((error) => console.error(error));
								}
							}}
						>
							<span className="text-nowrap">{displayLabel}</span>
							<span className="text-xs text-muted-foreground">{label}</span>
						</DropdownMenuCheckboxItem>
					))}
				</DropdownMenuGroup>
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
