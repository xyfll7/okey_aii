import { ChatProvider } from "#/components/chat/chatProvider";
import { SelectedProvider } from "#/store";
import { ChatList } from "./ChatList";
import { Inputer } from "./Inputer";
import LanguageSelector from "./LanguageSelector";

export function SessionView({ session_id }: { session_id: string }) {
	return (
		<SelectedProvider>
			<ChatProvider session_id={session_id}>
				<div className="h-full flex flex-col overflow-hidden">
					<ChatList />
					<div className="px-2 pb-2">
						<Inputer session_id={session_id} />
					</div>
					<LanguageSelector />
				</div>
			</ChatProvider>
		</SelectedProvider>
	);
}
