import {
	createContext,
	type ReactNode,
	useContext,
	useState,
} from "react";

type SelectedContextValue = {
	text: string;
	setText: (text: string) => void;
};

const SelectedContext = createContext<SelectedContextValue>({
	text: "",
	setText: () => {},
});

export function SelectedProvider({ children }: { children: ReactNode }) {
	const [text, setText] = useState("");
	return (
		<SelectedContext.Provider value={{ text, setText }}>
			{children}
		</SelectedContext.Provider>
	);
}

export const useSelected = () => useContext(SelectedContext);
