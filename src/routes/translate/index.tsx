import { createFileRoute } from "@tanstack/react-router";
import { ChatList } from "./-components/chatList";

export const Route = createFileRoute("/translate/")({
	component: RouteComponent,
});

function RouteComponent() {

	return (
		<ChatList></ChatList>
	);
}
