import { type MouseEvent, useRef } from "react";
import { useChatInit } from "#/components/chat/chatInit";
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
import { cn } from "#/lib/utils";
import { useSelected } from "#/store";
import { MessageBubble } from "./MessageBubble";
import MessageNavigator from "./MessageNavigator";
import { SelectionFloatingButton } from "./SelectionFloatingButton";

function handleChatSelection(
	e: MouseEvent<HTMLElement>,
	callback: (e: string) => void,
) {
	const selection = window.getSelection();
	const text = selection?.toString().trim();
	if (!text || !selection || selection.rangeCount === 0) return;
	const range = selection.getRangeAt(0);
	if (e.currentTarget.contains(range.commonAncestorContainer)) {
		callback(text);
	}
}

export function ChatList({ session_id }: { session_id: string }) {
	const chatListRef = useRef<HTMLDivElement>(null);
	const { setText } = useSelected();
	const { messages, status } = useChatInit({ session_id });
	const msgs = messages.filter((e) => e.role !== "system");
	const isBusy = status === "submitted" || status === "streaming";
	return (
		<MessageScrollerProvider defaultScrollPosition="last-anchor">
			<SelectionFloatingButton containerRef={chatListRef} />
			<MessageScroller
				className="relative"
				onMouseUp={(e) => handleChatSelection(e, (text) => setText(text))}
			>
				<MessageNavigator />
				<MessageScrollerViewport
					ref={chatListRef}
					className={cn(
						"[scrollbar-color:color-mix(in_oklch,var(--foreground)_17%,transparent)_transparent]",
						"scroll-fade-t",
					)}
				>
					<MessageScrollerContent
						aria-busy={isBusy}
						data-chat-container
						className="p-4"
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
				<MessageScrollerButton
					className="start-s-1/2 rounded-full"
					variant="secondary"
					size="icon-sm"
				/>
				{msgs.length === 0 && (
					<Empty
						className="absolute inset-0"
						onMouseUp={(e) => handleChatSelection(e, (text) => setText(text))}
					>
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
				)}
			</MessageScroller>
		</MessageScrollerProvider>
	);
}
