import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { useChatContext } from "#/components/chat/chatContext";
import { Icons } from "#/components/icon";
import {
	InputGroup,
	InputGroupAddon,
	InputGroupButton,
	InputGroupTextarea,
} from "#/components/ui/input-group";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";
import { useSelected } from "@/store";
import { useDrawerStack } from "./DrawerStack";
import { ModelSwitcher } from "./ModelSwitcher";
import { SelectedText } from "./SelectedText";
import { SessionView } from "./SessionView";

export function Inputer({
	className,
	session_id,
}: {
	className?: string;
	session_id: string;
}) {
	const [value, setValue] = useState("");
	const selected = useSelected();
	const { append, status, stop } = useChatContext();
	const { push } = useDrawerStack();
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
					<SelectedText
						onChatNew={async (tag) => {
							const [new_session_id] = await invoke<[string]>("create_session");
							push({
								id: new_session_id,
								content: () => (
									<SessionView
										session_id={new_session_id}
										onChatReady={(append) => {
											// Auto-send the selected text + tag as the new
											// session's opening message once its chat is ready.
											const text = selected.text.trim();
											if (!text) return;
											append({
												id: crypto.randomUUID(),
												role: "user",
												createdAt: new Date(),
												parts: [
													{ type: "text", content: text },
													{ type: "text", content: tag.content ?? "" },
												],
											});
										}}
									/>
								),
							});
						}}
						onChat={(tag) => {
							const text = selected.text.trim();
							if (!text) return;
							append({
								id: crypto.randomUUID(),
								role: "user",
								createdAt: new Date(),
								parts: [
									{ type: "text", content: text },
									{ type: "text", content: tag.content ?? "" },
								],
							});
						}}
					/>
				</InputGroupAddon>
			)}
			<InputGroupTextarea
				placeholder={m.translate_input_placeholder()}
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
				<ModelSwitcher session_id={session_id} />
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
