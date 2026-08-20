import { useEffect, useRef, useState } from "react";
import { useChatContext } from "#/components/chat/chatContext";
import { getMessageText } from "#/components/chat/chatUtils";
import { Icons } from "#/components/icon";
import {
	HoverCard,
	HoverCardContent,
	HoverCardTrigger,
} from "#/components/ui/hover-card";
import {
	useMessageScroller,
	useMessageScrollerVisibility,
} from "#/components/ui/message-scroller";
import { cn } from "@/lib/utils";

const MIN_MESSAGES = 4;
const MAX_MESSAGES = 17;
const NavTick = ({
	isActive,
	role,
	content,
	onClick,
}: {
	isActive: boolean;
	role: string;
	content: string;
	onClick: () => void;
}) => {
	return (
		<HoverCard>
			<HoverCardTrigger>
				<button
					className="group/tick gap-2 whitespace-nowrap font-medium cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-60 disabled:cursor-not-allowed transition-colors duration-100 [&_svg]:shrink-0 select-none text-fg-secondary hover:text-fg-primary disabled:hover:bg-transparent border border-transparent px-2.5 text-xs rounded-full relative flex items-center justify-end w-10 h-3 animate-none hover:bg-transparent"
					type="button"
					aria-label={role === "user" ? "Go to your message" : "Go to response"}
					aria-current={isActive ? "true" : undefined}
					onClick={onClick}
				>
					<div
						className={cn(
							"overflow-hidden",
							"rounded-full h-px transition-[width,opacity,background-color] duration-150 will-change-[width] bg-fg-tertiary opacity-50 group-hover:opacity-70 group-hover/tick:w-4 group-hover/tick:bg-fg-primary group-hover/tick:opacity-100",
							isActive
								? "w-4 bg-fg-primary! opacity-100!"
								: role === "assistant"
									? "w-3"
									: "w-1.5",
						)}
					/>
				</button>
			</HoverCardTrigger>
			<HoverCardContent
				side="left"
				align="center"
				sideOffset={8}
				className="max-w-64 max-h-40 overflow-hidden"
			>
				<p className="text-xs text-fg-secondary leading-relaxed line-clamp-5">
					{content}
				</p>
			</HoverCardContent>
		</HoverCard>
	);
};

const MessageNavigator = () => {
	const { messages } = useChatContext();
	const msg = messages.filter((e) => e.role !== "system").slice(0, MAX_MESSAGES);

	const total = msg.length;

	const { scrollToMessage } = useMessageScroller();
	const { currentAnchorId, visibleMessageIds } = useMessageScrollerVisibility();

	const visibleSet = new Set(visibleMessageIds);

	// Ambient "where am I" id derived from the scroller's own visibility
	// tracking. `currentAnchorId` is the sticky anchor (stays set after it
	// scrolls above the viewport) and only ever points at user messages
	// (because only user messages are scroll anchors); fall back to the
	// topmost visible message when the anchor isn't in our (sliced) window.
	const ambientId =
		(currentAnchorId && msg.some((m) => m.id === currentAnchorId)
			? currentAnchorId
			: msg.find((m) => visibleSet.has(m.id))?.id) ?? msg[0]?.id;

	// Explicit navigation pointer. While the user drives the prev/next
	// controls (or a tick) we own the active index so it actually advances
	// to the message we scrolled to, instead of being re-derived back onto
	// the nearest user anchor after the scroll settles.
	const [navId, setNavId] = useState<string | undefined>(undefined);
	const isNavigating = useRef(false);

	// Mirror the latest ambient id into a ref so the scroll listener below
	// always reads fresh data without re-subscribing on every render.
	const ambientIdRef = useRef(ambientId);
	useEffect(() => {
		ambientIdRef.current = ambientId;
	}, [ambientId]);

	// Sync the navigation pointer back to ambient tracking whenever the user
	// scrolls the viewport on their own (not while we drive a programmatic
	// navigation). State updates here happen inside a scroll event handler,
	// not an effect, so they stay lint-clean. We listen in the capture phase
	// on window because the viewport's scroll events don't bubble.
	useEffect(() => {
		const onScroll = () => {
			if (isNavigating.current) return;
			setNavId(ambientIdRef.current);
		};
		window.addEventListener("scroll", onScroll, true);
		return () => window.removeEventListener("scroll", onScroll, true);
	}, []);

	const activeId = navId ?? ambientId;
	const activeIndex = msg.findIndex((m) => m.id === activeId);

	const scrollToIndex = (index: number) => {
		const target = msg[index];
		if (!target) return;
		isNavigating.current = true;
		setNavId(target.id);
		scrollToMessage(target.id);
		// Release the navigation lock shortly after the (instant) scroll so
		// ambient tracking resumes on the next natural scroll.
		window.setTimeout(() => {
			isNavigating.current = false;
		}, 150);
	};

	if (total <= MIN_MESSAGES) return null;

	const clampedActive = Math.min(Math.max(activeIndex, 0), total - 1);
	const canGoPrev = clampedActive > 0;
	const canGoNext = clampedActive < total - 1;

	return (
		<div className="absolute right-2 top-1/2 -translate-y-1/2 z-20">
			<div className="group flex flex-col items-center gap-1">
				<button
					className="inline-flex items-center justify-center whitespace-nowrap text-sm font-medium leading-[normal] cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-60 disabled:cursor-not-allowed [&_svg]:shrink-0 select-none text-fg-secondary hover:bg-button-ghost-hover hover:text-fg-primary disabled:hover:bg-transparent border border-transparent h-8 gap-1.5 rounded-full overflow-hidden w-8 px-1.5 py-1.5 opacity-0! transition-all duration-200 group-hover:opacity-100! disabled:group-hover:opacity-60! -me-2 translate-y-1 group-hover:translate-y-0"
					type="button"
					aria-label="Navigate to previous message"
					disabled={!canGoPrev}
					onClick={() => scrollToIndex(clampedActive - 1)}
				>
					<Icons.arrowUp01 className="size-4" />
				</button>

				<div className="flex flex-col items-end gap-0">
					{msg.map((item, index) => (
						<NavTick
							key={item.id}
							isActive={index === clampedActive}
							role={item.role}
							content={getMessageText(item)[0]}
							onClick={() => scrollToIndex(index)}
						/>
					))}
				</div>

				<button
					className="inline-flex items-center justify-center whitespace-nowrap text-sm font-medium leading-[normal] cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-60 disabled:cursor-not-allowed [&_svg]:shrink-0 select-none text-fg-secondary hover:bg-button-ghost-hover hover:text-fg-primary disabled:hover:bg-transparent border border-transparent h-8 gap-1.5 rounded-full overflow-hidden w-8 px-1.5 py-1.5 opacity-0! transition-all duration-200 group-hover:opacity-100! disabled:group-hover:opacity-60! -me-2 -translate-y-1 group-hover:translate-y-0"
					type="button"
					aria-label="Navigate to next message"
					disabled={!canGoNext}
					onClick={() => scrollToIndex(clampedActive + 1)}
				>
					<Icons.arrowDown01 className="size-4" />
				</button>
			</div>
		</div>
	);
};

export default MessageNavigator;
