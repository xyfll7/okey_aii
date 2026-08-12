import { ChatList } from "#/routes/(index)/-components/chatList";
import { ChatProvider } from "./chat/chatProvider";

export function SessionView({ session_id }: { session_id: string }) {
	return (
		<ChatProvider>
			<ChatList session_id={session_id}/>
		</ChatProvider>
	);
}
