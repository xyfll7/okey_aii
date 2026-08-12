import { createRootRoute, Outlet } from "@tanstack/react-router";
import { DrawerStackProvider } from "#/components/drawer-stack";
import { ThemeProvider } from "#/components/theme-provider";
import "../styles.css";

export const Route = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	return (
		<ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
			<DrawerStackProvider>
				<Outlet />
			</DrawerStackProvider>
		</ThemeProvider>
	);
}
