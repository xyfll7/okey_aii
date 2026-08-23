import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { ScrollArea } from "#/components/ui/scroll-area";
import { cn } from "#/lib/utils";
import { useDrawerStack } from "#/routes/(index)/-components/DrawerStack";

function SettingsContent({ className }: { className?: string }) {
	return (
		<ScrollArea className={cn("h-full", "overflow-hidden", className)}>
			<div className="max-w-screen flex-coh items-start px-2 pr-4">
					123123
			</div>
		</ScrollArea>
	);
}

export function Settings() {
	const { push } = useDrawerStack();
	return (
		<Button
			size={"icon-sm"}
			variant={"ghost"}
			onClick={() => {
				push({
					title: "Settings",
					content: <SettingsContent />,
				});
			}}
		>
			<Icons.settings />
		</Button>
	);
}
