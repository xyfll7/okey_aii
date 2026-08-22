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
import { cn } from "#/lib/utils";
import "./MessageNavigator.css";

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
			<HoverCardTrigger delay={70} closeDelay={0}>
				<button
					className="group/tick gap-2 whitespace-nowrap font-medium cursor-pointer focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-60 disabled:cursor-not-allowed transition-colors duration-100 [&_svg]:shrink-0 select-none text-fg-secondary hover:text-fg-primary disabled:hover:bg-transparent border border-transparent  text-xs rounded-full relative flex items-center justify-end w-5 h-3 animate-none hover:bg-transparent"
					type="button"
					aria-label={role === "user" ? "Go to your message" : "Go to response"}
					aria-current={isActive ? "true" : undefined}
					onClick={onClick}
				>
					<div
						className={cn(
							"rounded-full h-px transition-[width,opacity,background-color] duration-150 will-change-[width] bg-fg-tertiary opacity-50 group-hover:opacity-70  group-hover/tick:bg-fg-primary group-hover/tick:opacity-100",
							"overflow-hidden",
							"group-hover/tick:w-3.5",
							isActive
								? "w-2.5 bg-fg-primary! opacity-100!"
								: role === "assistant"
									? "w-2"
									: "w-1",
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

	
	const ambientId =
		(currentAnchorId && msg.some((m) => m.id === currentAnchorId)
			? currentAnchorId
			: msg.find((m) => visibleSet.has(m.id))?.id) ?? msg[0]?.id;

	
	const [navId, setNavId] = useState<string | undefined>(undefined);
	const isNavigating = useRef(false);

	
	const ambientIdRef = useRef(ambientId);
	useEffect(() => {
		ambientIdRef.current = ambientId;
	}, [ambientId]);

	
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
		
		
		window.setTimeout(() => {
			isNavigating.current = false;
		}, 150);
	};

	if (total <= MIN_MESSAGES) return null;

	const clampedActive = Math.min(Math.max(activeIndex, 0), total - 1);
	const canGoPrev = clampedActive > 0;
	const canGoNext = clampedActive < total - 1;

	return (
		<div className={cn("absolute right-3 top-1/2 -translate-y-1/2 z-20")}>
			<div className="group flex flex-col items-end gap-1">
				<Icons.arrowUp01
					className={cn(
						"-me-0.5",
						"size-4 text-fg-secondary hover:text-fg-primary select-none transition-all duration-200 opacity-0! group-hover:opacity-100!  translate-y-1 group-hover:translate-y-0",
						canGoPrev
							? "cursor-pointer"
							: "cursor-default group-hover:opacity-60!",
					)}
					aria-label="Navigate to previous message"
					onClick={() => canGoPrev && scrollToIndex(clampedActive - 1)}
				/>

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

				<Icons.arrowDown01
					className={cn(
						"-me-0.5",
						"size-4 text-fg-secondary hover:text-fg-primary select-none transition-all duration-200 opacity-0! group-hover:opacity-100! -translate-y-1 group-hover:translate-y-0",
						canGoNext
							? "cursor-pointer"
							: "cursor-default group-hover:opacity-60!",
					)}
					aria-label="Navigate to next message"
					onClick={() => canGoNext && scrollToIndex(clampedActive + 1)}
				/>
			</div>
		</div>
	);
};

export default MessageNavigator;
