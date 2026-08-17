import { Inputer } from "#/components/Inputer";
import { cn } from "#/lib/utils";
import { ChatList } from "#/routes/(index)/-components/chatList";
import { ChatProvider } from "./chat/chatProvider";

export function SessionView({ session_id }: { session_id: string }) {
	return (
		<ChatProvider>
			<div className={cn("relative h-full", "flex flex-col overflow-hidden")}>
				<ChatList session_id={session_id} />
				<div className="px-2 pb-2">
					<Inputer />
				</div>
			</div>
		</ChatProvider>
	);
}
