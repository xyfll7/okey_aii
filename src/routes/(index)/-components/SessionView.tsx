import { ChatProvider } from "#/components/chat/chatProvider";
import { ChatList } from "#/routes/(index)/-components/ChatList";
import { Inputer } from "#/routes/(index)/-components/Inputer";
import { SelectedContext } from "@/store";

export function SessionView({ session_id }: { session_id: string }) {
	return (
		<SelectedContext.Provider value={{ text: "" }}>
			<ChatProvider session_id={session_id}>
				<ChatList session_id={session_id} />
				<div className="px-2 pb-2">
					<Inputer />
				</div>
			</ChatProvider>
		</SelectedContext.Provider>
	);
}
