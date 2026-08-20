import { useChatInit } from "#/components/chat/chatInit";
import { ChatProvider } from "#/components/chat/chatProvider";
import { ChatList } from "#/routes/(index)/-components/ChatList";
import { Inputer } from "#/routes/(index)/-components/Inputer";

export function SessionView({ session_id }: { session_id: string }) {
	return <ChatProvider>
		<ChatContent session_id={session_id} />
	</ChatProvider>;
}

function ChatContent({ session_id }: { session_id: string }) {
	const { messages, status } = useChatInit({ session_id });
	const msgs = messages.filter((e) => e.role !== "system");
	const isBusy = status === "submitted" || status === "streaming";
	return (
		<>
			<ChatList msgs={msgs} isBusy={isBusy} />
			<div className="px-2 pb-2">
				<Inputer isBusy={isBusy} />
			</div>
		</>
	);
}
