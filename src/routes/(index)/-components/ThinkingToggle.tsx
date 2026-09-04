import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Icons } from "#/components/icon";
import { InputGroupButton } from "#/components/ui/input-group";
import { cn } from "#/lib/utils";
import { m } from "#/paraglide/messages";
import type { Session } from "#/types";

/**
 * Toggles reasoning/thinking mode for a session. The current state is loaded
 * once per session from the backend; toggling persists it (and rebuilds the
 * session's agent, so later requests carry the provider's thinking params).
 */
export function ThinkingToggle({ session_id }: { session_id: string }) {
	const [thinking, setThinking] = useState(false);
	useEffect(() => {
		let cancelled = false;
		invoke<Session[]>("list_sessions")
			.then((sessions) => {
				const session = sessions.find((s) => s.session_id === session_id);
				if (!cancelled && session) setThinking(session.thinking);
			})
			.catch(console.error);
		return () => {
			cancelled = true;
		};
	}, [session_id]);

	const toggleThinking = async () => {
		const next = !thinking;
		try {
			await invoke("toggle_thinking", { session_id, thinking: next });
			setThinking(next);
		} catch (error) {
			console.error(error);
		}
	};

	return (
		<InputGroupButton
			variant="ghost"
			size="icon-xs"
			aria-pressed={thinking}
			title={thinking ? m.translate_thinking_on() : m.translate_thinking_off()}
			aria-label={thinking ? m.translate_thinking_on() : m.translate_thinking_off()}
			className={cn(
				"cursor-pointer",
				thinking
					? "bg-primary/10 text-primary hover:bg-primary/15"
					: "text-muted-foreground hover:bg-muted",
			)}
			onClick={toggleThinking}
		>
			<Icons.brain />
		</InputGroupButton>
	);
}
