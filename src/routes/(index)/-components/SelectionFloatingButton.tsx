import { computePosition, flip, offset, shift } from "@floating-ui/dom";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import {
	ButtonGroup,
	ButtonGroupSeparator,
} from "#/components/ui/button-group";
import { speak } from "#/lib/utils";
import { useSelected } from "#/store";
import { useDrawerStack } from "./DrawerStack";
import { SessionView } from "./SessionView";

function getSelectionRect(range: Range): DOMRect {
	const rects = Array.from(range.getClientRects()).filter(
		(r) => r.width > 0 && r.height > 0,
	);
	if (rects.length === 0) {
		return range.getBoundingClientRect();
	}
	const top = Math.min(...rects.map((r) => r.top));
	const left = Math.min(...rects.map((r) => r.left));
	const right = Math.max(...rects.map((r) => r.right));
	const bottom = Math.max(...rects.map((r) => r.bottom));
	return new DOMRect(left, top, right - left, bottom - top);
}

function useSelectionFloatingButton(
	containerRef?: React.RefObject<HTMLElement | null>,
) {
	const buttonRef = useRef<HTMLDivElement>(null);
	const [visible, setVisible] = useState(false);
	const [coords, setCoords] = useState({ x: 0, y: 0 });
	const pendingTextRef = useRef<string>("");

	const clear = useCallback(() => {
		setVisible(false);
		pendingTextRef.current = "";
	}, []);

	useEffect(() => {
		function handleMouseUp(e: MouseEvent) {
			if (buttonRef.current?.contains(e.target as Node)) return;

			const selection = window.getSelection();
			const text = selection?.toString().trim();
			if (!selection || !text || selection.rangeCount === 0) {
				clear();
				return;
			}

			const range = selection.getRangeAt(0);

			if (
				containerRef?.current &&
				!containerRef.current.contains(range.commonAncestorContainer)
			) {
				clear();
				return;
			}

			pendingTextRef.current = text;

			const virtualEl = {
				getBoundingClientRect: () => getSelectionRect(range),
			};

			if (!buttonRef.current) return;
			computePosition(virtualEl, buttonRef.current, {
				placement: "top",
				strategy: "fixed",
				middleware: [offset(8), flip({ padding: 48 }), shift({ padding: 8 })],
			}).then(({ x, y }) => {
				setCoords({ x, y });
				setVisible(true);
			});
		}

		function handleSelectionChange() {
			const selection = window.getSelection();
			if (!selection || !selection.toString().trim()) {
				clear();
			}
		}

		function handleScroll() {
			clear();
		}

		document.addEventListener("mouseup", handleMouseUp);
		document.addEventListener("selectionchange", handleSelectionChange);
		document.addEventListener("scroll", handleScroll, true);
		return () => {
			document.removeEventListener("mouseup", handleMouseUp);
			document.removeEventListener("selectionchange", handleSelectionChange);
			document.removeEventListener("scroll", handleScroll, true);
		};
	}, [clear, containerRef]);

	return { buttonRef, visible, coords, pendingTextRef, clear };
}

export function SelectionFloatingButton({
	containerRef,
}: {
	containerRef?: React.RefObject<HTMLElement | null>;
}) {
	const { buttonRef, visible, coords, pendingTextRef, clear } =
		useSelectionFloatingButton(containerRef);
	const { push } = useDrawerStack();
	const selected = useSelected();
	return (
		<ButtonGroup
			ref={buttonRef}
			className="fixed z-50 transition-none rounded-lg border bg-background shadow-md"
			style={{
				left: coords.x,
				top: coords.y,
				opacity: visible ? 1 : 0,
				pointerEvents: visible ? "auto" : "none",
			}}
		>
			<Button
				size="icon-sm"
				variant={"secondary"}
				onClick={() => {
					if (pendingTextRef.current) {
						speak(pendingTextRef.current);
					}
					clear();
				}}
			>
				<Icons.volumeHigh />
			</Button>
			<ButtonGroupSeparator />
			<Button
				size="icon-sm"
				variant={"secondary"}
				onClick={() => {
					if (pendingTextRef.current) {
						navigator.clipboard?.writeText(pendingTextRef.current);
					}
					clear();
				}}
			>
				<Icons.copy />
			</Button>
			<Button
				size="icon-sm"
				variant={"secondary"}
				onClick={async () => {
					const [new_session_id] = await invoke<[string]>("create_session");
					// Fetch the current translator instruction (depends on
					// "self-explaining" mode) so the new session's opening
					// message carries it as the tag.
					const translate_instruction = await invoke<string>(
						"translate_prompt",
					);
					push({
						id: new_session_id,
						content: () => (
							<SessionView
								session_id={new_session_id}
								onChatReady={(append) => {
									// Auto-send the selected text + tag as the new
									// session's opening message once its chat is ready.
									const text = selected.text.trim();
									if (!text) return;
									append({
										id: crypto.randomUUID(),
										role: "user",
										createdAt: new Date(),
										parts: [
											{ type: "text", content: text },
											...(translate_instruction
												? [
														{
															type: "text" as const,
															content: translate_instruction,
														},
													]
												: []),
										],
									});
								}}
							/>
						),
					});
				}}
			>
				<Icons.arrowUpRight01 />
			</Button>
		</ButtonGroup>
	);
}
