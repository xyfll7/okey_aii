import type { ChatAppend } from "#/components/chat/chatContext";
import { ChatProvider } from "#/components/chat/chatProvider";
import { SelectedProvider } from "#/store";
import { ChatList } from "./ChatList";
import { Inputer } from "./Inputer";
import LanguageSelector from "./LanguageSelector";

export function SessionView({
	session_id,
	onChatReady,
}: {
	session_id: string;
	/** Fired once per session when its chat has initialized and `append` can be safely called. */
	onChatReady?: (append: ChatAppend) => void;
}) {
	return (
		<SelectedProvider>
			<ChatProvider session_id={session_id} onChatReady={onChatReady}>
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
