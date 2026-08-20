import { Inputer } from "#/components/Inputer";
import { cn } from "#/lib/utils";
import { ChatList } from "#/routes/(index)/-components/chatList";
import { useChatContext } from "../../../components/chat/chatContext";
import { useChatInit } from "../../../components/chat/chatInit";
import { ChatProvider } from "../../../components/chat/chatProvider";

export function SessionView({ session_id }: { session_id: string }) {
	useChatInit({ session_id });
	const { messages, status } = useChatContext();
	const msg = messages.filter((e) => e.role !== "system");
	const isBusy = status === "submitted" || status === "streaming";
	return (
		<ChatProvider>
			<div className={cn("relative h-full", "flex flex-col overflow-hidden")}>
				<ChatList msg={msg} isBusy={isBusy} />
				<div className="px-2 pb-2">
					<Inputer />
				</div>
			</div>
		</ChatProvider>
	);
}
