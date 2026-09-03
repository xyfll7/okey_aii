import type { UIMessage } from "@tanstack/ai-react";
import { Markdown } from "@tanstack/markdown/react";
import { useChatContext } from "#/components/chat/chatContext";
import { getMessageText } from "#/components/chat/chatUtils";
import { Icons } from "#/components/icon";
import {
	Bubble,
	BubbleContent,
	BubbleGroup,
	BubbleReactions,
} from "#/components/ui/bubble";
import { Button } from "#/components/ui/button";
import {
	Message,
	MessageContent,
	MessageFooter,
} from "#/components/ui/message";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";

export function MessageBubble({ message }: { message: UIMessage }) {
	const { messages, error, reload, status } = useChatContext();
	const parts = getMessageText(message);
	// `useChat` keeps a single `error` that always describes the latest failed
	// turn, so only the trailing user message is allowed to surface it.
	const isFailedTurn =
		!!error &&
		status !== "submitted" &&
		status !== "streaming" &&
		messages.filter((e) => e.role === "user").at(-1)?.id === message.id;
	return (
		<Message align={message.role === "user" ? "end" : "start"}>
			<MessageContent>
				<BubbleGroup className="w-full">
					{message.role === "user" && (
						<>
							<Bubble variant={"outline"} align="end">
								<BubbleContent>{parts[0]}</BubbleContent>
								<BubbleReactions
									className={cn("sr-only")}
									align="start"
									role="img"
									aria-label="Reaction: thumbs up"
								>
									<span>👍</span>
								</BubbleReactions>
							</Bubble>
							{parts
								.slice(1)
								.filter(Boolean)
								.map((text, index) => (
									<MessageFooter key={`${message.id}-extra-${String(index)}`}>
										{text}
									</MessageFooter>
								))}
							{isFailedTurn && (
								<>
									<MessageFooter className="gap-2">
										<span className="font-normal text-destructive">
											{m.chat_send_failed()}
										</span>
										<Button
											variant="ghost"
											size="icon-xs"
											title={m.chat_retry()}
											aria-label={m.chat_retry()}
											onClick={() => {
												void reload();
											}}
										>
											<Icons.refresh />
										</Button>
									</MessageFooter>
									<MessageFooter className="gap-2">
										<span className="font-normal text-destructive">
											{error.message}
										</span>
									</MessageFooter>
								</>
							)}
						</>
					)}
					{message.role === "assistant" && (
						<>
							{/* <MessageHeader>{"123213"}</MessageHeader> */}
							<Bubble variant="ghost">
								<BubbleContent>
									<Markdown>{parts.join("")}</Markdown>
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
						</>
					)}
				</BubbleGroup>
			</MessageContent>
		</Message>
	);
}
