import { useState } from "react";
import { useChatContext } from "#/components/chat/chatContext";
import { Icons } from "#/components/icon";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "#/components/ui/dropdown-menu";
import {
	InputGroup,
	InputGroupAddon,
	InputGroupButton,
	InputGroupTextarea,
} from "#/components/ui/input-group";
import { cn } from "#/lib/utils";
import { useSelected } from "@/store";
import { SelectedText } from "./SelectedText";

export function Inputer({ className }: { className?: string }) {
	const [value, setValue] = useState("");
	const selected = useSelected();
	const { append, status, stop } = useChatContext();
	const isBusy = status === "submitted" || status === "streaming";
	const handleSend = async () => {
		if (isBusy) {
			stop();
			return;
		}
		const content = value.trim();
		if (!content) return;
		append({
			id: crypto.randomUUID(),
			role: "user",
			createdAt: new Date(),
			parts: [{ type: "text", content }],
		});
		setValue("");
	};
	return (
		<InputGroup
			className={cn(
				className,
				"rounded-xl",
				"has-[[data-slot=input-group-control]:focus-visible]:border-ring/70 has-[[data-slot=input-group-control]:focus-visible]:ring-ring/7",
			)}
		>
			{selected.text && (
				<InputGroupAddon align="block-start">
					<SelectedText onChat={(e) => {}} />
				</InputGroupAddon>
			)}
			<InputGroupTextarea
				placeholder={"m.translate_input_placeholder()"}
				value={value}
				onChange={(e) => setValue(e.target.value)}
				onKeyDown={(e) => {
					if (e.key === "Enter" && !e.shiftKey) {
						e.preventDefault();
						handleSend();
					}
				}}
			/>
			<InputGroupAddon align="block-end">
				<DropdownMenu>
					<DropdownMenuTrigger
						render={<InputGroupButton variant="ghost" size="icon-xs" />}
					/>
					<DropdownMenuContent side="top" align="start">
						<DropdownMenuItem>123123</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>

				<InputGroupButton
					variant="default"
					className="rounded-full ml-auto cursor-pointer"
					size="icon-xs"
					onClick={handleSend}
				>
					{isBusy ? <Icons.stop /> : <Icons.arrowUp />}
					<span className="sr-only">{isBusy ? "abort" : "send"}</span>
				</InputGroupButton>
			</InputGroupAddon>
		</InputGroup>
	);
}
