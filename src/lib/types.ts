export interface PromptTag {
	id: number;
	label?: string;
	content?: string;
}
export const AutoSpeakState = {
	Off: "off",
	Single: "single",
	All: "all",
} as const;

export type AutoSpeakState = typeof AutoSpeakState[keyof typeof AutoSpeakState];