import { useChatContext } from "#/components/chat/chatContext";
import { Icons } from "#/components/icon";
import {
	Bubble,
	BubbleContent,
	BubbleGroup,
	BubbleReactions,
} from "#/components/ui/bubble";
import { Button } from "#/components/ui/button";
import { Message, MessageContent } from "#/components/ui/message";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";

/**
 * The failure of the latest turn, rendered as its own row at the end of the thread.
 *
 * `useChat` holds a single `error` for the whole client and it always describes
 * the turn that just failed, so there is no message to match it against — it just
 * asks "is there an error and are we idle" and shows or hides itself.
 */
export function ErrorBubble() {
	const { error, reload, status } = useChatContext();
	if (!error || status === "submitted" || status === "streaming") return null;

	return (
		<Message align="start">
			<MessageContent>
				<BubbleGroup className="w-full">
					<Bubble variant="destructive" className="max-w-full">
						<BubbleContent>
							<div role="alert" className="space-y-2 text-sm">
								<p className="flex items-center gap-2 font-medium">
									<Icons.alert className="size-4 shrink-0" />
									{m.chat_error_title()}
								</p>
								<p className="text-xs opacity-80">{error.message}</p>
								{/* Safe unconditionally: reaching this means a turn ran, so a
								    user message exists — and `reload()` no-ops anyway. */}
								<Button
									size="xs"
									variant="outline"
									className="gap-1"
									onClick={() => {
										void reload();
									}}
								>
									<Icons.refresh />
									{m.chat_retry()}
								</Button>
							</div>
						</BubbleContent>
						<BubbleReactions
							className={cn("sr-only", "translate-y-4/4")}
							align="start"
							role="img"
							aria-label="Reaction: thumbs up"
						>
							<span>👍</span>
						</BubbleReactions>
					</Bubble>
				</BubbleGroup>
			</MessageContent>
		</Message>
	);
}
