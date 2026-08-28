import { invoke } from "@tauri-apps/api/core";
import { useSyncExternalStore } from "react";
import {
	overwriteGetLocale,
	setLocale as setParaglideLocale,
} from "@/paraglide/runtime.js";

export type AppLocale = "en" | "zh-CN";

type LocaleListener = () => void;

const listeners = new Set<LocaleListener>();

function subscribeLocale(listener: LocaleListener) {
	listeners.add(listener);
	return () => {
		listeners.delete(listener);
	};
}

let currentLocale: AppLocale = "en";

overwriteGetLocale(() => currentLocale);

export function setLocale(locale: AppLocale) {
	currentLocale = locale;
	setParaglideLocale(locale, { reload: false });
	for (const listener of listeners) {
		listener();
	}
}

export async function initLocale() {
	const stored = await invoke<string>("get_current_locale");
	setLocale(stored as AppLocale);
}

export function useLocale() {
	return useSyncExternalStore(subscribeLocale, () => currentLocale);
}
