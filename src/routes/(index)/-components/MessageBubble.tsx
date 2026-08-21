import type { UIMessage } from "@tanstack/ai-react";
import { Markdown } from "@tanstack/markdown/react";
import { getMessageText } from "#/components/chat/chatUtils";
import {
	Bubble,
	BubbleContent,
	BubbleGroup,
	BubbleReactions,
} from "#/components/ui/bubble";
import {
	Message,
	MessageContent,
	MessageFooter,
} from "#/components/ui/message";
import { cn } from "#/lib/utils";

export function MessageBubble({ message }: { message: UIMessage }) {
	return (
		<Message align={message.role === "user" ? "end" : "start"}>
			<MessageContent>
				<BubbleGroup className="w-full">
					{message.role === "user" && (
						<>
							<Bubble variant={"outline"} align="end">
								<BubbleContent>{getMessageText(message)[0]}</BubbleContent>
								<BubbleReactions
									className={cn("sr-only")}
									align="start"
									role="img"
									aria-label="Reaction: thumbs up"
								>
									<span>👍</span>
								</BubbleReactions>
							</Bubble>
							{getMessageText(message)[1] && (
								<MessageFooter>{getMessageText(message)[1]}</MessageFooter>
							)}
						</>
					)}
					{message.role === "assistant" && (
						<>
							{/* <MessageHeader>{"123213"}</MessageHeader> */}
							<Bubble variant="ghost">
								<BubbleContent>
									<Markdown>{getMessageText(message).join("")}</Markdown>
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
