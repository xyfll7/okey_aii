import { Markdown } from "@tanstack/markdown/react";
import { useEffect, useRef, useState } from "react";
import { Icons } from "#/components/icon";
import { MarkerContent } from "#/components/ui/marker";
import { MessageHeader } from "#/components/ui/message";
import { ScrollArea } from "#/components/ui/scroll-area";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";

/**
 * Collapsible panel showing the model's reasoning/thinking process for an
 * assistant message. Rendered above the answer bubble.
 *
 * While `isThinkingActive` (thinking is still streaming in) the panel is kept
 * expanded automatically; once the thinking finishes it collapses so the
 * answer takes over. The header can always be clicked to toggle manually.
 */
export function ThinkingBlock({
	content,
	isThinkingActive = false,
}: {
	content: string;
	className?: string;
	isThinkingActive?: boolean;
}) {
	const [open, setOpen] = useState(isThinkingActive);
	const wasActiveRef = useRef(isThinkingActive);

	// Keep the panel open while reasoning is streaming in; collapse it the
	// moment the streaming finishes.
	useEffect(() => {
		const wasActive = wasActiveRef.current;
		wasActiveRef.current = isThinkingActive;
		if (isThinkingActive) {
			setOpen(true);
		} else if (wasActive) {
			setOpen(false);
		}
	}, [isThinkingActive]);

	if (!content?.trim()) return null;
	return (
		<div className="min-w-0">
			<MessageHeader
				role="banner"
				onClick={() => setOpen((prev) => !prev)}
				className={cn("cursor-pointer select-none")}
				aria-expanded={open}
			>
				<MarkerContent className="flex gap-1">
					<span
						className={cn("text-start", isThinkingActive && "shimmer")}
					>
						{m.translate_thinking_section()}
					</span>
					<Icons.arrowRight01
						className={cn(
							"size-3.5",
							"transition-transform duration-200",
							open && "rotate-90",
						)}
					/>
				</MarkerContent>
			</MessageHeader>
			{/* Keep the panel mounted even when collapsed: unmounting would
			    re-parse the markdown on every toggle, which flashes the page.
			    `hidden` (display:none) collapses it instantly — no animation,
			    and since the box is fully removed from layout it can't add a
			    phantom gap inside the parent's flex spacing either. */}
			<div className={cn(!open && "hidden")}>
				<ScrollArea className="mt-2 flex max-h-32 min-h-0 flex-col overflow-hidden whitespace-pre-wrap px-3 pb-2.5 text-xs leading-relaxed text-muted-foreground">
					<Markdown>{content}</Markdown>
				</ScrollArea>
			</div>
		</div>
	);
}
