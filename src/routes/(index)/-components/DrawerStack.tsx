/**
 * DrawerStack.tsx
 *
 * A context-driven, infinitely-nestable Drawer stack built on top of
 * shadcn's Base UI Drawer (`#/components/ui/drawer`).
 *
 * Instead of hand-nesting <Drawer> N levels deep in JSX, you keep a
 * push/pop stack in React state and *recursively* render that stack as
 * a real nested JSX tree. This preserves Base UI's native nested-drawer
 * behavior (stacked transform, `data-nested-drawer-open`, only the
 * frontmost layer is interactive) while letting any component in your
 * app open "the next drawer" imperatively via `useDrawerStack()`.
 *
 * Usage:
 *   1. Wrap your app once with <DrawerStackProvider>.
 *   2. Anywhere inside, call:
 *        const { push, pop, closeTo, closeAll } = useDrawerStack()
 *        push({ title: "Profile", content: <ProfilePane /> })
 *   3. Inside `content`, call useDrawerStack() for push/pop/closeTo/closeAll,
 *      and useDrawerLayerId() to get this layer's own id (e.g. to close
 *      itself via closeTo(id)).
 */

import * as React from "react";
import {
	Drawer,
	DrawerContent,
	DrawerDescription,
	DrawerHeader,
	DrawerTitle,
} from "#/components/ui/drawer";

type SwipeDirection = "up" | "right" | "down" | "left";

export interface PushLayerInput {
	/** stable id; auto-generated if omitted */
	id?: string;
	title?: React.ReactNode;
	description?: React.ReactNode;
	content: React.ReactNode;
	swipeDirection?: SwipeDirection;
	showSwipeHandle?: boolean;
	/** applied to DrawerContent, e.g. "h-[50vh]" or "w-96" */
	contentClassName?: string;
}

type DrawerLayerState = PushLayerInput & { id: string };

interface DrawerStackContextValue {
	layers: DrawerLayerState[];
	push: (layer: PushLayerInput) => string;
	/** closes the topmost layer */
	pop: () => void;
	/** closes `id` and everything stacked above it, leaving layers below untouched */
	closeTo: (id: string) => void;
	/** closes every layer */
	closeAll: () => void;
}

const DrawerStackContext = React.createContext<DrawerStackContextValue | null>(
	null,
);

export function useDrawerStack() {
	const ctx = React.useContext(DrawerStackContext);
	if (!ctx) {
		throw new Error("useDrawerStack must be used inside <DrawerStackProvider>");
	}
	return ctx;
}

function makeId() {
	return typeof crypto !== "undefined" && "randomUUID" in crypto
		? crypto.randomUUID()
		: `drawer_${Date.now()}_${Math.random().toString(36).slice(2)}`;
}

export function DrawerStackProvider({
	children,
	onClose,
}: {
	children: React.ReactNode;
	/** called with the ids being closed whenever `closeMany` runs */
	onClose?: (ids: string[]) => void;
}) {
	const [layers, setLayers] = React.useState<DrawerLayerState[]>([]);

	// Always-fresh snapshot of onClose for imperative callbacks, so we never
	// call a stale version if the parent re-renders with a new handler.
	const onCloseRef = React.useRef(onClose);
	React.useEffect(() => {
		onCloseRef.current = onClose;
	}, [onClose]);
	// ids that are mid exit-animation and should be spliced out once
	// Base UI reports the transition as complete.
	const [closingIds, setClosingIds] = React.useState<Set<string>>(new Set());

	// Always-fresh snapshot of layers for imperative callbacks below, so pop/
	// closeTo/closeAll never act on a stale closure.
	const layersRef = React.useRef<DrawerLayerState[]>(layers);
	React.useEffect(() => {
		layersRef.current = layers;
	}, [layers]);

	const push = React.useCallback((input: PushLayerInput) => {
		const id = input.id ?? makeId();
		setLayers((prev) => [...prev, { ...input, id }]);
		return id;
	}, []);

	/**
	 * Closes the given ids (bottom-to-top order).
	 *
	 * Only the frontmost layer in the whole stack is ever actually visible
	 * mid-transition, so it's the only one guaranteed to fire
	 * `onOpenChangeComplete`. If we're closing exactly that one layer, wait
	 * for its real exit animation. If we're closing several at once (bulk
	 * close), the hidden ones underneath may never fire completion at all —
	 * so just drop everything synchronously instead of risking layers that
	 * get stuck in `closingIds` forever and block the stack from reopening.
	 */
	const closeMany = React.useCallback((ids: string[]) => {
		onCloseRef.current?.(ids);
		if (ids.length === 0) return;

		if (ids.length === 1) {
			const [id] = ids;
			setClosingIds((prev) => new Set(prev).add(id));
			return;
		}

		setLayers((prev) => prev.filter((l) => !ids.includes(l.id)));
		setClosingIds((prev) => {
			if (prev.size === 0) return prev;
			const next = new Set(prev);
			for (const id of ids) {
				next.delete(id);
			}
			return next;
		});
	}, []);

	const pop = React.useCallback(() => {
		const top = layersRef.current[layersRef.current.length - 1];
		if (top) closeMany([top.id]);
	}, [closeMany]);

	const closeTo = React.useCallback(
		(id: string) => {
			const idx = layersRef.current.findIndex((l) => l.id === id);
			if (idx === -1) return;
			const ids = layersRef.current.slice(idx).map((l) => l.id);
			closeMany(ids);
		},
		[closeMany],
	);

	const closeAll = React.useCallback(() => {
		closeMany(layersRef.current.map((l) => l.id));
	}, [closeMany]);

	// Called once a given layer's close transition has fully finished.
	const handleOpenChangeComplete = React.useCallback(
		(id: string, open: boolean) => {
			if (open) return;
			setClosingIds((prev) => {
				if (!prev.has(id)) return prev;
				const next = new Set(prev);
				next.delete(id);
				return next;
			});
			setLayers((prev) => prev.filter((l) => l.id !== id));
		},
		[],
	);

	const value = React.useMemo<DrawerStackContextValue>(
		() => ({ layers, push, pop, closeTo, closeAll }),
		[layers, push, pop, closeTo, closeAll],
	);

	return (
		<DrawerStackContext.Provider value={value}>
			{children}
			<DrawerStackOutlet
				layers={layers}
				closingIds={closingIds}
				closeTo={closeTo}
				onOpenChangeComplete={handleOpenChangeComplete}
			/>
		</DrawerStackContext.Provider>
	);
}

function DrawerStackOutlet({
	layers,
	closingIds,
	closeTo,
	onOpenChangeComplete,
}: {
	layers: DrawerLayerState[];
	closingIds: Set<string>;
	closeTo: (id: string) => void;
	onOpenChangeComplete: (id: string, open: boolean) => void;
}) {
	if (layers.length === 0) return null;

	// Recursively render layers[index..] as a genuinely nested JSX tree so
	// Base UI's built-in nested-drawer stacking/animation kicks in.
	const renderLayer = (index: number): React.ReactNode => {
		const layer = layers[index];
		const isLast = index === layers.length - 1;

		return (
			<DrawerLayerNode
				key={layer.id}
				layer={layer}
				closing={closingIds.has(layer.id)}
				onDismiss={() => closeTo(layer.id)}
				onOpenChangeComplete={(open) => onOpenChangeComplete(layer.id, open)}
			>
				<div className="flex-1 overflow-hidden">{layer.content}</div>
				{!isLast ? renderLayer(index + 1) : null}
			</DrawerLayerNode>
		);
	};

	return renderLayer(0);
}

/**
 * Wraps a single stack layer's <Drawer>.
 *
 * Mounting a Base UI Drawer with `open` already `true` on its very first
 * render is treated as "was already open" — no enter transition plays.
 * Real trigger-driven usage avoids this because the <Drawer> is mounted
 * closed from page load and only *later* flips to open on click.
 *
 * We replicate that: this node always mounts with `entered = false`, then
 * flips to `true` a couple of frames later, so Base UI sees a genuine
 * false -> true transition and plays the enter animation. Closing is
 * driven separately by the `closing` prop and always takes priority.
 */
function DrawerLayerNode({
	layer,
	closing,
	onDismiss,
	onOpenChangeComplete,
	children,
}: {
	layer: DrawerLayerState;
	closing: boolean;
	onDismiss: () => void;
	onOpenChangeComplete: (open: boolean) => void;
	children: React.ReactNode;
}) {
	const [entered, setEntered] = React.useState(false);

	React.useEffect(() => {
		let raf2 = 0;
		const raf1 = requestAnimationFrame(() => {
			raf2 = requestAnimationFrame(() => setEntered(true));
		});
		return () => {
			cancelAnimationFrame(raf1);
			if (raf2) cancelAnimationFrame(raf2);
		};
		// Only ever run once, right after this layer mounts.
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);

	const open = !closing && entered;

	return (
		<Drawer
			open={open}
			onOpenChange={(next) => {
				if (!next) onDismiss();
			}}
			onOpenChangeComplete={onOpenChangeComplete}
			swipeDirection={layer.swipeDirection ?? "down"}
			showSwipeHandle={layer.showSwipeHandle}
		>
			<DrawerContent className={layer.contentClassName ?? "h-full"}>
				{(layer.title || layer.description) && (
					<DrawerHeader>
						{layer.title && <DrawerTitle>{layer.title}</DrawerTitle>}
						{layer.description && (
							<DrawerDescription>{layer.description}</DrawerDescription>
						)}
					</DrawerHeader>
				)}
				{children}
			</DrawerContent>
		</Drawer>
	);
}
