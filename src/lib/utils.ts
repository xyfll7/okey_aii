import { invoke } from "@tauri-apps/api/core";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}
export async function speak(text: string) {
	if ("speechSynthesis" in window) {
		const utterance = new SpeechSynthesisUtterance(text);
		utterance.rate = 1.0;
		utterance.pitch = 1.0;
		utterance.volume = 1.0;
		utterance.lang = await invoke<"en" | "zh-CN">("detect_language", { text });
		speechSynthesis.speak(utterance);
	} else {
		console.error("The browser does not support TTS.");
	}
}
