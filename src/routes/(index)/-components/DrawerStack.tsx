import * as React from "react";
import {
	Drawer,
	DrawerContent,
	DrawerDescription,
	DrawerHeader,
	DrawerTitle,
} from "#/components/ui/drawer";
import { cn } from "#/lib/utils";

type SwipeDirection = "up" | "right" | "down" | "left";

/**
 * 抽屉层的 title/description/content 一律使用工厂函数 `() => ReactNode`：
 * 每次渲染时重新求值，语言切换（或其他外部状态变化）触发层重渲染后
 * 组件会重新执行，m.*() 消息文本自动同步更新，且 React 复用 fiber
 * 保留弹窗内部组件 state。
 */
type DrawerNode = () => React.ReactNode;

interface PushLayerInput {
	id?: string;
	title?: DrawerNode;
	description?: DrawerNode;
	content: DrawerNode;

	swipeDirection?: SwipeDirection;
	showSwipeHandle?: boolean;
	contentClassName?: string;
}

type DrawerLayerState = PushLayerInput & { id: string };

/** 解析抽屉层节点：调用工厂函数求值。 */
function resolveNode(node: DrawerNode): React.ReactNode {
	return node();
}

interface DrawerStackContextValue {
	layers: DrawerLayerState[];
	push: (layer: PushLayerInput) => string;

	pop: () => void;

	closeTo: (id: string) => void;

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
		: `drawer_$$${Date.now()}_$$${Math.random().toString(36).slice(2)}`;
}

export function DrawerStackProvider({
	children,
	onClose,
}: {
	children: React.ReactNode;

	onClose?: (ids: string[]) => void;
}) {
	const [layers, setLayers] = React.useState<DrawerLayerState[]>([]);

	const onCloseRef = React.useRef(onClose);
	React.useEffect(() => {
		onCloseRef.current = onClose;
	}, [onClose]);

	const [closingIds, setClosingIds] = React.useState<Set<string>>(new Set());

	const layersRef = React.useRef<DrawerLayerState[]>(layers);
	React.useEffect(() => {
		layersRef.current = layers;
	}, [layers]);

	const push = React.useCallback((input: PushLayerInput) => {
		const id = input.id ?? makeId();
		setLayers((prev) => [...prev, { ...input, showSwipeHandle: true, id }]);
		return id;
	}, []);

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
				<div className="flex-1 overflow-hidden">{resolveNode(layer.content)}</div>
				{!isLast ? renderLayer(index + 1) : null}
			</DrawerLayerNode>
		);
	};

	return renderLayer(0);
}

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

	const maskDragRef = React.useCallback((node: HTMLDivElement | null) => {
		node?.parentElement?.setAttribute("data-tauri-drag-region", "true");
	}, []);

	React.useEffect(() => {
		let raf2 = 0;
		const raf1 = requestAnimationFrame(() => {
			raf2 = requestAnimationFrame(() => setEntered(true));
		});
		return () => {
			cancelAnimationFrame(raf1);
			if (raf2) cancelAnimationFrame(raf2);
		};
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
			<DrawerContent
				ref={maskDragRef}
				className={cn(layer.contentClassName ?? "h-full")}
			>
				{(layer.title || layer.description) && (
					<DrawerHeader data-tauri-drag-region className="pb-2">
						{layer.title && (
							<DrawerTitle data-tauri-drag-region>{resolveNode(layer.title)}</DrawerTitle>
						)}
						{layer.description && (
							<DrawerDescription data-tauri-drag-region>
								{resolveNode(layer.description)}
							</DrawerDescription>
						)}
					</DrawerHeader>
				)}
				{children}
			</DrawerContent>
		</Drawer>
	);
}
