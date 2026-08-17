import { Store } from "@tanstack/react-store";

export const s_Selected = new Store<{ text: string }>({ text: "", });

