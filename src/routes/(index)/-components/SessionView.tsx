import { useChatContext } from "#/components/chat/chatContext";
import { useChatInit } from "#/components/chat/chatInit";
import { ChatProvider } from "#/components/chat/chatProvider";
import { cn } from "#/lib/utils";
import { ChatList } from "#/routes/(index)/-components/ChatList";
import { Inputer } from "#/routes/(index)/-components/Inputer";

export function SessionView({ session_id }: { session_id: string }) {
	useChatInit({ session_id });
	const { messages, status } = useChatContext();
	const msgs = messages.filter((e) => e.role !== "system");
	const isBusy = status === "submitted" || status === "streaming";
	return (
		<ChatProvider>
			<div className={cn("relative h-full", "flex flex-col overflow-hidden")}>
				<ChatList msgs={msgs} isBusy={isBusy} />
				<div className="px-2 pb-2">
					<Inputer isBusy={isBusy} />
				</div>
			</div>
		</ChatProvider>
	);
}
