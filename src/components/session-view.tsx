import { ChatList } from "#/routes/translate/-components/chatList";
import { useDrawerLayerId } from "@/components/drawer-stack";
import { ChatInit } from "./chat/chatInit";
import { ChatProvider } from "./chat/chatProvider";

export function SessionView({ session_id }: { session_id: string }) {
	const id = useDrawerLayerId();
	return (
		<div>
			<ChatProvider>
				<ChatInit session_id={session_id}>
					<ChatList />
				</ChatInit>
			</ChatProvider>
		</div>
	);
}
