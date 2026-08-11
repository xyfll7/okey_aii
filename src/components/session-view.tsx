import { ChatList } from "#/routes/translate/-components/chatList";
import { useDrawerLayerId } from "@/components/drawer-stack";

export function SessionView() {
	const id = useDrawerLayerId();
	return <div>
		<ChatList></ChatList>
	</div>
}
