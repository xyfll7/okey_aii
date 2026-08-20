import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import {
	Item,
	ItemActions,
	ItemContent,
	ItemDescription,
	ItemGroup,
	ItemTitle,
} from "#/components/ui/item";
import { ScrollArea } from "#/components/ui/scroll-area";
import {
	Drawer,
	DrawerContent,
	DrawerDescription,
	DrawerHeader,
	DrawerTitle,
	DrawerTrigger,
} from "@/components/ui/drawer";
import type { PromptTag } from "@/lib/types";
import { cn } from "@/lib/utils";
export function PromptTags({
	className,
	prompts,
	onDelete,
	onAdd,
	onEdit,
}: {
	className?: string;
	prompts: PromptTag[];
	onDelete?: (id: number) => void;
	onAdd?: (label: string, content: string) => void;
	onEdit?: (id: number, label: string, content: string) => void;
}) {
	const [isOpen, setIsOpen] = useState(false);
	const [newLabel, setNewLabel] = useState("");
	const [newContent, setNewContent] = useState("");
	const [editingId, setEditingId] = useState<number | null>(null);

	useEffect(() => {
		if (!isOpen) return;
		const overlay = document.querySelector('[data-slot="drawer-overlay"]');
		if (overlay) {
			(overlay as HTMLElement).setAttribute("data-tauri-drag-region", "true");
		}
	}, [isOpen]);

	const resetForm = () => {
		setNewLabel("");
		setNewContent("");
		setEditingId(null);
	};

	const handleAdd = () => {
		const label = newLabel.trim();
		if (!label) return;
		onAdd?.(label, newContent.trim() || label);
		resetForm();
	};

	const handleEdit = () => {
		if (editingId === null) return;
		const label = newLabel.trim();
		if (!label) return;
		onEdit?.(editingId, label, newContent.trim() || label);
		resetForm();
	};

	const startEdit = (id: number, label: string, content: string) => {
		setEditingId(id);
		setNewLabel(label);
		setNewContent(content);
	};

	return (
		<Drawer open={isOpen} onOpenChange={setIsOpen}>
			<DrawerTrigger
				onClick={async (e) => {
					(e.currentTarget as HTMLButtonElement).blur();
					setIsOpen(true);
				}}
			>
				<Button size={"xs"} variant={"outline"} className={className}>
					<Icons.add />
				</Button>
			</DrawerTrigger>
			<DrawerContent
				className={cn(
					"h-[80vh]  overflow-hidden",
					"pb-2 [&_.bg-muted.mx-auto.mt-4.hidden.h-1.w-[100px].shrink-0.rounded-full]:hidden",
				)}
			>
				<DrawerHeader className="" data-tauri-drag-region>
					<DrawerTitle
						className={cn("flex justify-between select-none", "")}
						data-tauri-drag-region
					>
						{"m.prompts_title()"}
					</DrawerTitle>
					<DrawerDescription className="sr-only" />
				</DrawerHeader>
				<div className="flex flex-col gap-2 px-2 pb-2">
					<Input
						placeholder={"m.prompts_label_placeholder()"}
						value={newLabel}
						onChange={(e) => setNewLabel(e.target.value)}
					/>
					<Input
						placeholder={"m.prompts_content_placeholder()"}
						value={newContent}
						onChange={(e) => setNewContent(e.target.value)}
					/>
					<div className="flex justify-end gap-2">
						<Button
							size={"xs"}
							variant={"ghost"}
							disabled={editingId === null}
							onClick={handleEdit}
						>
							{"m.prompts_edit()"}
						</Button>
						<Button
							size={"xs"}
							variant={"default"}
							onClick={editingId === null ? handleAdd : resetForm}
						>
							{editingId === null ? "m.prompts_add()" : "m.common_cancel()"}
						</Button>
					</div>
				</div>
				<ScrollArea className={cn("h-full", "overflow-hidden")}>
					<ItemGroup className="px-2">
						{[...prompts].map((e) => (
							<Item key={e.id} variant="outline" size="xs">
								<ItemContent>
									<ItemTitle>{e.label}</ItemTitle>
									<ItemDescription>{e.content}</ItemDescription>
								</ItemContent>
								<ItemActions>
									<Button
										size={"icon-xs"}
										variant={"ghost"}
										className="h-4 w-4"
										onClick={() =>
											startEdit(e.id, e.label ?? "", e.content ?? "")
										}
									>
										<Icons.pen />
									</Button>
									<Button
										size={"icon-xs"}
										variant={"ghost"}
										className="h-4 w-4"
										onClick={() => onDelete?.(e.id)}
									>
										<Icons.x />
									</Button>
								</ItemActions>
							</Item>
						))}
					</ItemGroup>
				</ScrollArea>
			</DrawerContent>
		</Drawer>
	);
}
