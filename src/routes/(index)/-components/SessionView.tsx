import { ChatProvider } from "#/components/chat/chatProvider";
import { cn } from "#/lib/utils";
import { ChatList } from "#/routes/(index)/-components/ChatList";
import { Inputer } from "#/routes/(index)/-components/Inputer";
import { SelectedContext } from "@/store";

export function SessionView({ session_id }: { session_id: string }) {
	return (
		<SelectedContext.Provider value={{ text: "11" }}>
			<ChatProvider session_id={session_id}>
				<div className={cn("relative h-full", "flex flex-col overflow-hidden")}>
					<ChatList session_id={session_id} />
					<div className="px-2 pb-2">
						<Inputer />
					</div>
				</div>
			</ChatProvider>
		</SelectedContext.Provider>
	);
}
