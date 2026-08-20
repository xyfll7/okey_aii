
/**
 * Corresponds to the Rust struct PromptTag in src-tauri/src/states/app_config.rs
 */
export interface PromptTag {
	id: number;
	label?: string;
	content?: string;
	raw?: string;
}
