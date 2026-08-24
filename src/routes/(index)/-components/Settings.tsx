import { invoke } from "@tauri-apps/api/core";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
	DropdownMenuContent,
	DropdownMenuGroup,
	DropdownMenuTrigger,
} from "#/components/ui/dropdown-menu";
import { ScrollArea } from "#/components/ui/scroll-area";
import { setLocale, useLocale } from "#/lib/locale";
import { cn } from "#/lib/utils";
import { useDrawerStack } from "#/routes/(index)/-components/DrawerStack";
import { m } from "@/paraglide/messages.js";

function SettingsContent({ className }: { className?: string }) {
	return (
		<ScrollArea className={cn("h-full", "overflow-hidden", className)}>
			<div className="max-w-screen flex-coh items-start px-2 pr-4">123123</div>
		</ScrollArea>
	);
}

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

export function LanguageSelector() {
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
