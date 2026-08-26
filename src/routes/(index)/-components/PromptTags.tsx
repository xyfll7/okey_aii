import { useState } from "react";
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
import type { PromptTag } from "#/lib/types";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";
import { useDrawerStack } from "#/routes/(index)/-components/DrawerStack";

function PromptTagsContent({
	prompts,
	onDelete,
	onAdd,
	onEdit,
}: {
	prompts: PromptTag[];
	onDelete?: (id: number) => void;
	onAdd?: (label: string, content: string) => void;
	onEdit?: (id: number, label: string, content: string) => void;
}) {
	const [newLabel, setNewLabel] = useState("");
	const [newContent, setNewContent] = useState("");
	const [editingId, setEditingId] = useState<number | null>(null);

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
		<>
			<div className="flex flex-col gap-2 px-2 pb-2">
				<Input
					placeholder={m.prompts_label_placeholder()}
					value={newLabel}
					onChange={(e) => setNewLabel(e.target.value)}
				/>
				<Input
					placeholder={m.prompts_content_placeholder()}
					value={newContent}
					onChange={(e) => setNewContent(e.target.value)}
				/>
				{editingId == null && (
					<div className="flex justify-end gap-2">
						<Button size={"xs"} variant={"outline"} onClick={handleAdd}>
							{m.prompts_add()}
						</Button>
					</div>
				)}
				{editingId !== null && (
					<div className="flex justify-end gap-2">
						<Button size={"xs"} variant={"ghost"} onClick={handleEdit}>
							{m.prompts_edit()}
						</Button>
						<Button size={"xs"} variant={"ghost"} onClick={resetForm}>
							{m.common_cancel()}
						</Button>
					</div>
				)}
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
		</>
	);
}

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
	const { push } = useDrawerStack();
	return (
		<Button
			size={"xs"}
			variant={"outline"}
			className={className}
			onClick={() => {
				push({
					title: () => m.prompts_title(),
					content: () => (
						<PromptTagsContent
							prompts={prompts}
							onDelete={onDelete}
							onAdd={onAdd}
							onEdit={onEdit}
						/>
					),
				});
			}}
		>
			<Icons.add />
		</Button>
	);
}
