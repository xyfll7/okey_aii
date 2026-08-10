import { useDrawerLayerId, useDrawerStack } from "@/components/drawer-stack";
import { Button } from "@/components/ui/button";

export function SessionView() {
	const { push, pop } = useDrawerStack();
	const id = useDrawerLayerId();
	return (
		<div className="flex flex-col gap-3">
			<Button
				variant="outline"
				onClick={() =>
					push({
						title: "SessionView",
						showSwipeHandle: true,
						content: <SessionView />,
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
