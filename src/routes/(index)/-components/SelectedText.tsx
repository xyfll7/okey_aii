import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import Copyed from "#/components/Copyed";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import type { PromptTag } from "#/lib/types";
import { cn, speak } from "#/lib/utils";
import { useSelected } from "@/store";
import { PromptTags } from "./PromptTags";

export function SelectedText({ onChat }: { onChat: (e: PromptTag) => void }) {
	const { text, setText } = useSelected();
	const { tags, add, update, remove } = usePromptTags();
	console.log("ttttt:", tags);
	if (!text) return "";
	return (
		<div className="w-full">
			<div className="w-full flex items-center mb-1">
				<div className="max-w-full truncate overflow-hidden">
					<span className={cn("mr-1")}>{text}</span>
				</div>
				{text?.trim() && (
					<Button size={"icon-sm"} variant={"ghost"}>
						<Copyed key={text} text={text} />
					</Button>
				)}
				{text?.trim() && (
					<Button
						size={"icon-sm"}
						variant={"ghost"}
						onClick={() => {
							if (!text) return;
							speak(text);
						}}
					>
						<Icons.volumeHigh />
					</Button>
				)}
				{text?.trim() && (
					<Button
						size={"icon-sm"}
						variant={"ghost"}
						onClick={() => {
							setText("");
						}}
					>
						<Icons.x />
					</Button>
				)}
			</div>
			{text?.trim() && (
				<div className="flex flex-wrap">
					{tags.map((tag) => (
						<Button
							className="mr-1 mb-1"
							size={"xs"}
							variant={"outline"}
							key={tag.id}
							onClick={() => onChat(tag)}
						>
							{tag.label}
						</Button>
					))}
					<PromptTags
						prompts={tags}
						onDelete={(id) => remove(id)}
						onAdd={(label, content) => add(label, content)}
						onEdit={(id, label, content) => update({ id, label, content })}
					/>
				</div>
			)}
		</div>
	);
}

export function usePromptTags() {
	const [tags, setTags] = useState<PromptTag[]>([]);

	const refresh = useCallback(async () => {
		setTags(await invoke<PromptTag[]>("get_prompt_tags"));
	}, []);

	useEffect(() => {
		refresh().catch((err) => console.error("Failed to load prompt tags:", err));
	}, [refresh]);

	const add = useCallback(async (label: string, content: string) => {
		setTags(await invoke<PromptTag[]>("add_prompt_tag", { label, content }));
	}, []);

	const update = useCallback(async (tag: PromptTag) => {
		setTags(
			await invoke<PromptTag[]>("update_prompt_tag", {
				id: tag.id,
				label: tag.label ?? "",
				content: tag.content ?? "",
			}),
		);
	}, []);

	const remove = useCallback(async (id: number) => {
		setTags(await invoke<PromptTag[]>("delete_prompt_tag", { id }));
	}, []);

	return { tags, refresh, add, update, remove };
}
