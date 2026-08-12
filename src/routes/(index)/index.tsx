import { createFileRoute } from "@tanstack/react-router";
import { useDrawerStack } from "#/components/drawer-stack";
import { SessionView } from "#/components/session-view";
import { Button } from "#/components/ui/button";
export const Route = createFileRoute("/(index)/")({ component: Home });

function Home() {
	const { push } = useDrawerStack();
	return (
		<div className="p-8">
			<Button
				onClick={() => {
					push({
						id: "string",
						title: "React.ReactNode",
						description: "React.ReactNode",
						showSwipeHandle: true,
						content: <SessionView session_id="123" />,
					});
				}}
			>
				add
			</Button>
		</div>
	);
}
