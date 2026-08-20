import { createContext, useContext } from "react";

export const SelectedContext = createContext<{ text: string }>({ text: "" });
export const useSelected = () => useContext(SelectedContext);
