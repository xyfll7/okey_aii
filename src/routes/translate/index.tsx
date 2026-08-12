import { createFileRoute } from "@tanstack/react-router";
import { SessionView } from "#/components/session-view";

export const Route = createFileRoute("/translate/")({
	component: RouteComponent,
});

function RouteComponent() {
	return (
		<SessionView session_id="1234"/>
	);
}
