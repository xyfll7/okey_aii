import { Markdown } from "@tanstack/markdown/react";
import { useState } from "react";
import { Icons } from "#/components/icon";
import {  MarkerContent } from "#/components/ui/marker";
import { MessageHeader } from "#/components/ui/message";
import { ScrollArea } from "#/components/ui/scroll-area";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";

/**
 * Collapsible panel showing the model's reasoning/thinking process for an
 * assistant message. Rendered above the answer bubble; open by default.
 */
export function ThinkingBlock({
	content,
}: {
	content: string;
	className?: string;
}) {
	const [open, setOpen] = useState(false);
	if (!content?.trim()) return null;
	return (
		<>
			<MessageHeader
				role="banner"
				onClick={() => setOpen((prev) => !prev)}
				className={cn("cursor-pointer select-none")}
				aria-expanded={open}
			>
				<MarkerContent className="flex gap-1">
					<span className="text-start">{m.translate_thinking_section()}</span>
					<Icons.arrowRight01
						className={cn(
							"size-3.5",
							"transition-transform duration-200",
							open && "rotate-90",
						)}
					/>
				</MarkerContent>
			</MessageHeader>
			{open && (
				<ScrollArea className="flex max-h-64 min-h-0 flex-col overflow-hidden whitespace-pre-wrap px-3 pb-2.5 text-xs leading-relaxed text-muted-foreground">
					<Markdown>{content}</Markdown>
				</ScrollArea>
			)}
		</>
	);
}
