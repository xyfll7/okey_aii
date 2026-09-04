/**
 * A provider supported by the backend. Mirrors `Provider` in
 * `src-tauri/src/ai/config.rs`; `label` is resolved by the backend's own i18n
 * (rust_i18n) from the current locale, so the frontend never keeps its own
 * provider → label mapping.
 */
export interface ProviderInfo {
	id: string;
	label: string;
	api_key_url: string;
	base_url: string | null;
}

export interface Session {
	session_id: string;
	title: string;
	provider: ProviderInfo;
	model: string;
	preset_id: string;
	/** Whether reasoning/thinking mode is enabled for this session. */
	thinking: boolean;
	created_at: number;
	update_at: number;
}

/**
 * A model listed by a provider. Mirrors `ModelInfo` in
 * `src-tauri/src/ai/config.rs`; the data itself is fetched at runtime via the
 * `list_models` command.
 */
export interface ModelInfo {
	id: string;
	label: string;
}
