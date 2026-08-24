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

/**
 * 当前语言的唯一数据源（模块级）。
 *
 * 不使用 paraglide 内置的 cookie/globalVariable 策略解析：
 * Tauri (WKWebView) 在 `tauri://` 自定义协议下 document.cookie 不可靠，
 * 且内置策略中 cookie 优先于 globalVariable，导致 setLocale 写入的
 * `_locale` 永远不会被 getLocale() 读到。
 */
let currentLocale: AppLocale = "en";

// 模块加载时立即接管 paraglide 的 locale 解析，确保所有 m.*() 消息
// 以及内部逻辑都读取我们的 currentLocale。
overwriteGetLocale(() => currentLocale);

/**
 * 切换语言并触发 React 树重新渲染。
 *
 * 注意：必须传 `{ reload: false }`，否则 paraglide 默认会执行
 * `window.location.reload()` 整页刷新。
 */
export function setLocale(locale: AppLocale) {
	currentLocale = locale;
	// 保持 paraglide 内部状态同步（cookie/globalVariable），不触发刷新
	setParaglideLocale(locale, { reload: false });
	for (const listener of listeners) {
		listener();
	}
}

/**
 * 应用启动时从 Tauri 后端读取持久化的语言并生效。
 */
export async function initLocale() {
	const stored = await invoke<string>("get_current_locale");
	setLocale(stored as AppLocale);
}

/**
 * 订阅当前语言，语言变化时所在组件（及其子树）会重新渲染，
 * 从而让渲染期读取的 `m.*()` 消息文本即时更新。
 */
export function useLocale() {
	return useSyncExternalStore(subscribeLocale, () => currentLocale);
}
