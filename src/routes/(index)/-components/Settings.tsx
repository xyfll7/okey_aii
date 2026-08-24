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
import { m } from "@/paraglide/messages.js";

export function Settings() {
	const { push } = useDrawerStack();
	return (
		<Button
			size={"icon-sm"}
			variant={"ghost"}
			onClick={() => {
				// title/content 用工厂函数形式：语言切换触发层重渲染时重新求值，
				// 弹窗内的 m.*() 文本自动同步更新，且保留弹窗内部组件 state。
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

function SettingsContent({ className }: { className?: string }) {
	const [apiKeys, setApiKeys] = useState<Record<string, string>>({});
	const [currentProvider, setCurrentProvider] = useState<ProviderId>("OpenAI");
	const labels = providerLabels();

	useEffect(() => {
		void invoke<Record<string, string>>("get_api_keys").then((keys) => {
			setApiKeys(keys);
		});
	}, []);

	const resetKey = `${currentProvider}-${apiKeys[currentProvider] ?? ""}`;

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
											{m.common_api_provider()} ({labels[currentProvider]})
										</FieldLabel>
										<ToggleGroup
											size="sm"
											value={[currentProvider]}
											onValueChange={(values) => {
												const value = values[0] as ProviderId | undefined;
												if (value && PROVIDERS.includes(value)) {
													setCurrentProvider(value);
												}
											}}
											variant="outline"
											spacing={2}
											className="flex-wrap w-full"
										>
											{PROVIDERS.map((id) => (
												<ToggleGroupItem
													key={id}
													value={id}
													aria-label={labels[id]}
												>
													{labels[id]}
												</ToggleGroupItem>
											))}
										</ToggleGroup>
									</Field>
									<Field>
										<FieldLabel htmlFor="model-provider-api-key">
											{labels[currentProvider]} {m.common_api_key()}
										</FieldLabel>
										<Input
											key={resetKey}
											defaultValue={apiKeys[currentProvider] ?? ""}
											id="model-provider-api-key"
											placeholder="Enter API Key"
											required
											onBlur={async (e) => {
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
												href={
													{
														OpenAI: "https://platform.openai.com/api-keys",
														Anthropic:
															"https://console.anthropic.com/settings/keys",
														DeepSeek: "https://www.deepseek.com/",
														Qwen: "https://bailian.console.aliyun.com/cn-beijing/#/home",
														Zai: "https://open.bigmodel.cn/login",
													}[currentProvider]
												}
												target="_blank"
												rel="noreferrer"
											>
												{m.common_get_api_key({
													provider: labels[currentProvider],
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

const PROVIDERS = ["OpenAI", "Anthropic", "DeepSeek", "Qwen", "Zai"] as const;

type ProviderId = (typeof PROVIDERS)[number];

function providerLabels(): Record<ProviderId, string> {
	return {
		OpenAI: m.model_providers_OpenAI(),
		Anthropic: m.model_providers_Anthropic(),
		DeepSeek: m.model_providers_DeepSeek(),
		Qwen: m.model_providers_Qwen(),
		Zai: m.model_providers_ZAI(),
	};
}

function LanguageSelector() {
	const currentLocale = useLocale();

	const changeLocale = async (locale: "en" | "zh-CN") => {
		try {
			await invoke("set_current_locale", { locale });
			setLocale(locale);
		} catch (error) {
			console.error(error);
		}
	};

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
								if (checked) void changeLocale(locale);
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
