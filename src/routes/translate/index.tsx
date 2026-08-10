import { createFileRoute } from "@tanstack/react-router";
import { useDrawerLayerId, useDrawerStack } from "@/components/drawer-stack";
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
					showSwipeHandle:true,
					content: <SettingsPane />,
				})
			}
		>
			Open Settings
		</Button>
	);
}

function SettingsPane() {
	const { push, pop } = useDrawerStack();
	const id = useDrawerLayerId();
	return (
		<div className="flex flex-col gap-3">
			<Button
				variant="outline"
				onClick={() =>
					push({
						title: "Profile",
						showSwipeHandle: true,
						content: <SettingsPane />,
					})
				}
			>
				Edit Profile →
			</Button>
			<Button variant="ghost" onClick={pop}>
				Close {id}
			</Button>
		</div>
	);
}
