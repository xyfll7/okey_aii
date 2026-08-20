import { type MouseEvent, useRef } from "react";
import type { ChatContextValue } from "#/components/chat/chatContext";
import { getMessageText } from "#/components/chat/chatUtils";
import { Icons } from "#/components/icon";
import {
	Empty,
	EmptyDescription,
	EmptyHeader,
	EmptyMedia,
	EmptyTitle,
} from "#/components/ui/empty";
import { Marker, MarkerContent } from "#/components/ui/marker";
import {
	MessageScroller,
	MessageScrollerButton,
	MessageScrollerContent,
	MessageScrollerItem,
	MessageScrollerProvider,
	MessageScrollerViewport,
} from "#/components/ui/message-scroller";
import { cn } from "@/lib/utils";
import { MessageBubble } from "./MessageBubble";

function handleChatSelection(e: MouseEvent<HTMLElement>) {
	const selection = window.getSelection();
	const text = selection?.toString().trim();
	if (!text || !selection || selection.rangeCount === 0) return;
	const range = selection.getRangeAt(0);
	if (e.currentTarget.contains(range.commonAncestorContainer)) {
	}
}

export function ChatList({ msgs, isBusy }: { msgs: ChatContextValue["messages"]; isBusy: boolean }) {
	const chatListRef = useRef<HTMLDivElement>(null);
	return (
		<MessageScrollerProvider>
			{msgs.length === 0 ? (
				<Empty className="h-full" onMouseUp={handleChatSelection}>
					<EmptyHeader>
						<EmptyMedia variant="icon">
							<Icons.chat />
						</EmptyMedia>
						<EmptyTitle>{"m.translate_empty_title()"}</EmptyTitle>
						<EmptyDescription>
							{"m.translate_empty_description()"}
						</EmptyDescription>
					</EmptyHeader>
				</Empty>
			) : (
				<MessageScroller className="" onMouseUp={handleChatSelection}>
					<MessageScrollerViewport ref={chatListRef} className="scrollbar-area">
						<MessageScrollerContent
							aria-busy={isBusy}
							data-chat-container
							className="p-4 scroll-fade "
						>
							{msgs.map((item, index) => (
								<MessageScrollerItem
									className="[content-visibility:visible!]"
									key={item.id}
									messageId={item.id}
									scrollAnchor={item.role === "user"}
								>
									<MessageBubble message={item} />
									{msgs.length - 1 === index && (
										<Marker
											role="banner"
											className={cn(
												isBusy && !getMessageText(item).join("").length
													? ""
													: "sr-only",
											)}
										>
											<MarkerContent className="shimmer">
												<span className="font-medium">
													{"m.translate_loading()"}
												</span>
												...
											</MarkerContent>
										</Marker>
									)}
								</MessageScrollerItem>
							))}
						</MessageScrollerContent>
					</MessageScrollerViewport>
					<MessageScrollerButton className="start-s-1/2" />
				</MessageScroller>
			)}
		</MessageScrollerProvider>
	);
}
