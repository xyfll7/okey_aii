import { useSelector } from "@tanstack/react-store";
import { useState } from "react";
import { Icons } from "@/components/icon";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
	InputGroup,
	InputGroupAddon,
	InputGroupButton,
	InputGroupTextarea,
} from "@/components/ui/input-group";

import { cn } from "@/lib/utils";
import { s_Selected } from "@/store";

export function Inputer({
	className,
	isBusy,
}: { className?: string; isBusy?: boolean }) {
	const [value, setValue] = useState("");
	const selected = useSelector(s_Selected, (state) => state);

	return (
		<InputGroup
			className={cn(
				className,
				"rounded-xl",
				"has-[[data-slot=input-group-control]:focus-visible]:border-ring/70 has-[[data-slot=input-group-control]:focus-visible]:ring-ring/7",
			)}
		>
			{selected.text && (
				<InputGroupAddon align="block-start">
					<div>fadsf</div>
				</InputGroupAddon>
			)}
			<InputGroupTextarea
				placeholder={"m.translate_input_placeholder()"}
				value={value}
				onChange={(e) => setValue(e.target.value)}
				onKeyDown={async () => {}}
			/>
			<InputGroupAddon align="block-end">
				<DropdownMenu>
					<DropdownMenuTrigger
						render={
							<InputGroupButton variant="ghost" size="icon-xs" />
						}
					/>
					<DropdownMenuContent side="top" align="start">
						<DropdownMenuItem>123123</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>

				<InputGroupButton
					variant="default"
					className="rounded-full ml-auto cursor-pointer"
					size="icon-xs"
					onClick={async () => {}}
				>
					{isBusy ? <Icons.stop /> : <Icons.arrowUp />}
					<span className="sr-only">
						{isBusy ? "abort" : "send"}
					</span>
				</InputGroupButton>
			</InputGroupAddon>
		</InputGroup>
	);
}
