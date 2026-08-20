import { invoke } from "@tauri-apps/api/core";
import Copyed from "@/components/Copyed";
import { Icons } from "@/components/icon";
import { Button } from "@/components/ui/button";
import type { PromptTag } from "@/lib/types";
import { cn, speak } from "@/lib/utils";
import {  useSelected } from "@/store";
import { PromptTags } from "./PromptTags";

export function SelectedText({ onChat }: { onChat: (e: PromptTag) => void }) {
	const selected = useSelected();








	if (!selected.text) return "";
	return (
		<div className="w-full">
			<div className="w-full flex items-center mb-1">
				<div className="max-w-full truncate overflow-hidden">
					<span className={cn("mr-1")}>{selected.text}</span>
				</div>
				{selected.text?.trim() && (
					<Button size={"icon-sm"} variant={"ghost"}>
						<Copyed key={selected.text} text={selected.text} />
					</Button>
				)}
				{selected.text?.trim() && (
					<Button
						size={"icon-sm"}
						variant={"ghost"}
						onClick={() => {
							if (!selected.text) return;
							speak(selected.text);
						}}
					>
						<Icons.volumeHigh />
					</Button>
				)}
				{selected.text?.trim() && (
					<Button
						size={"icon-sm"}
						variant={"ghost"}
						onClick={() => {
							// selected.setState(() => ({ text: "" }));
						}}
					>
						<Icons.x />
					</Button>
				)}
			</div>
			{selected.text?.trim() && (
				<div className="flex flex-wrap">
					{[].slice(3).map((e) => (
						<Button
							className="mr-1 mb-1"
							size={"xs"}
							variant={"outline"}
							key={""}
							disabled={true}
							onClick={() => {
							
							}}
						>
							{"e.label"}
						</Button>
					))}
					<PromptTags
						prompts={[]}
						onDelete={()=>{}}
						onAdd={()=>{}}
						onEdit={()=>{}}
					/>
				</div>
			)}
		</div>
	);
}
