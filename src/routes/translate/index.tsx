import { createFileRoute } from "@tanstack/react-router";
import { SessionView } from "#/components/session-view";
import { useDrawerStack } from "@/components/drawer-stack";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/translate/")({
	component: RouteComponent,
});

function RouteComponent() {
	const { push } = useDrawerStack();

	return (
		<Button
			onClick={() =>
				push({
					title: "Settings",
					showSwipeHandle: true,
					content: <SessionView />,
				})
			}
		>
			Open Settings
		</Button>
	);
}
