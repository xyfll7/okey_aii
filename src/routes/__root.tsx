import { createRootRoute, Outlet } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { DrawerStackProvider } from "#/components/drawer-stack";
import { ThemeProvider } from "#/components/theme-provider";
import "../styles.css";
import { TooltipProvider } from "#/components/ui/tooltip";

export const Route = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	return (
		<ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
			<DrawerStackProvider
				onClose={(ids) => {
					for (const session_id of ids) {
						invoke("close_session", { session_id }).catch(console.error);
					}
				}}
			>
				<TooltipProvider>
					<Outlet />
				</TooltipProvider>
			</DrawerStackProvider>
		</ThemeProvider>
	);
}
