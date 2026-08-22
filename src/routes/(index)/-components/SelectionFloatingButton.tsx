import { computePosition, flip, offset, shift } from "@floating-ui/dom";
import { useEffect, useRef, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { ButtonGroup, ButtonGroupSeparator } from "#/components/ui/button-group";
import { speak } from "#/lib/utils";


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


export function SelectionFloatingButton({
	containerRef,
}: {
	containerRef?: React.RefObject<HTMLElement | null>;
}) {
	const buttonRef = useRef<HTMLDivElement>(null);
	const [visible, setVisible] = useState(false);
	const [coords, setCoords] = useState({ x: 0, y: 0 });
	const pendingTextRef = useRef<string>("");

	useEffect(() => {
		function hide() {
			setVisible(false);
			pendingTextRef.current = "";
		}

		function handleMouseUp(e: MouseEvent) {
			
			if (buttonRef.current?.contains(e.target as Node)) return;

			const selection = window.getSelection();
			const text = selection?.toString().trim();
			if (!selection || !text || selection.rangeCount === 0) {
				hide();
				return;
			}

			const range = selection.getRangeAt(0);

			
			if (containerRef?.current && !containerRef.current.contains(range.commonAncestorContainer)) {
				hide();
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
				middleware: [
					offset(8),
					flip({ padding: 48 }),
					shift({ padding: 8 }),
				],
			}).then(({ x, y }) => {
				setCoords({ x, y });
				setVisible(true);
			});
		}

		function handleSelectionChange() {
			const selection = window.getSelection();
			if (!selection || !selection.toString().trim()) {
				hide();
			}
		}

		function handleScroll() {
			hide();
		}

		document.addEventListener("mouseup", handleMouseUp);
		document.addEventListener("selectionchange", handleSelectionChange);
		document.addEventListener("scroll", handleScroll, true);
		return () => {
			document.removeEventListener("mouseup", handleMouseUp);
			document.removeEventListener("selectionchange", handleSelectionChange);
			document.removeEventListener("scroll", handleScroll, true);
		};
	}, [containerRef]);

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
					setVisible(false);
					pendingTextRef.current = "";
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
					setVisible(false);
					pendingTextRef.current = "";
				}}
			>
				<Icons.copy />
			</Button>
		</ButtonGroup>
	);
}
